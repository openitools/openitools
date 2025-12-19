use std::{collections::HashMap, path::PathBuf, str::FromStr};

use futures_util::{future::BoxFuture, lock::Mutex, FutureExt};
use openitools_idevice::afc::{get_afc_client, AfcClient, AfcFopenMode, FSTree, FileType};
use rfuse3::{raw::Session, MountOptions};
use tokio::{
    io::{AsyncReadExt, AsyncSeekExt as _, AsyncWriteExt as _},
    sync::RwLock,
};

use crate::device_management::afc::fuse::AfcFS;

// mod fuse;
mod fuse;

#[derive(Debug)]
pub enum FuseCommand {
    Shutdown,
    AddFile(String),
}

#[derive(Debug, Default)]
pub struct FuseChannel {
    sender: RwLock<Option<tokio::sync::mpsc::UnboundedSender<FuseCommand>>>,
}

/// makes sure that folders are up top
fn sort_fs_tree(node: &mut FSTree) -> &mut FSTree {
    if node.children.is_empty() {
        return node;
    }

    node.children.sort_by(|first, second| {
        let first_is_folder = matches!(first.info.file_type, FileType::Directory);
        let second_is_folder = matches!(second.info.file_type, FileType::Directory);

        match (first_is_folder, second_is_folder) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            (false, false) | (true, true) => std::cmp::Ordering::Equal,
        }
    });

    for child in &mut node.children {
        if child.children.is_empty() {
            continue;
        }

        sort_fs_tree(child);
    }

    node
}

#[tauri::command]
pub async fn dump_fs_tree() -> FSTree {
    let mut afc = get_afc_client().await;

    let mut root = FSTree::new("/");

    fn fill_tree<'a>(
        afc: &'a mut AfcClient,
        current_path: &'a str,
        tree: &'a mut FSTree,
    ) -> BoxFuture<'a, ()> {
        async move {
            let info = afc.get_file_info(current_path).await.unwrap();

            // fill the current path to it's info
            tree.info = info.into();

            if tree.info.is_dir() {
                let entries = afc.list_dir(current_path).await.unwrap();
                for entry in entries {
                    // skip the dots
                    if entry == "." || entry == ".." || entry == "DCIM" {
                        continue;
                    }

                    let mut child_tree = FSTree::new(&entry);

                    let child_path = format!("{}/{}", current_path.trim_end_matches('/'), entry);

                    fill_tree(afc, &child_path, &mut child_tree).await;

                    tree.add_child(child_tree);
                }
            }
        }
        .boxed()
    }

    fill_tree(&mut afc, "/", &mut root).await;
    sort_fs_tree(&mut root);
    root
}

#[tauri::command]
pub async fn mount_fuse(files_path: Vec<String>) -> Vec<String> {
    #[cfg(not(target_family = "unix"))]
    return vec![];

    let afc = get_afc_client().await;

    let mut tempdir = tempfile::tempdir().unwrap();
    tempdir.disable_cleanup(true);

    let tempdir = tempdir.path().to_path_buf();
    let tempdir1 = tempdir.clone();

    let files_path1 = files_path.clone();

    tokio::spawn(async move {
        let (tx, mut rx) = tokio::sync::watch::channel(());
        let fs = AfcFS::new(files_path1, afc, tx);
        let mut mount_options = MountOptions::default();
        unsafe {
            mount_options.uid(libc::getuid()).gid(libc::getgid());
        }
        mount_options
            .fs_name("OpeniTools-Fuse")
            .read_only(true)
            .nonempty(true);

        let mut mount_handle = Session::new(mount_options)
            .mount_with_unprivileged(fs, tempdir1.to_string_lossy().to_string())
            .await
            .unwrap();

        tokio::select! {
            res = &mut mount_handle => {
                match res {
                    Ok(_) => println!("Filesystem exited normally"),
                    Err(e) => {
                        println!("Filesystem runtime error: {}", e);
                    }
                }
            },
            _ = rx.changed() => {
                println!("Received shutdown signal, unmounting filesystem...");

                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

                match mount_handle.unmount().await {
                    Ok(()) => {
                        tokio::fs::remove_dir_all(tempdir1).await;
                    }
                    Err(e) => {
                        eprintln!("Failed to unmount fuse: {e}");
                    }
                }

            }
        }
    });

    files_path
        .into_iter()
        .map(|s| tempdir.join(s).to_string_lossy().to_string())
        .collect()
}

#[tauri::command]
pub async fn download_afc_files(files_path: Vec<String>) -> Vec<String> {
    let mut afc = get_afc_client().await;

    let mut output_paths = Vec::<String>::new();

    let mut tempdir = tempfile::tempdir().unwrap();
    tempdir.disable_cleanup(true);

    for file in files_path {
        let mut remote_file =
            tokio::io::BufReader::new(afc.open(&file, AfcFopenMode::RdOnly).await.unwrap());

        let local_file_path = tempdir.path().join(&file);

        let mut local_file = tokio::io::BufWriter::new(
            tokio::fs::File::options()
                .write(true)
                .truncate(true)
                .create(true)
                .open(&local_file_path)
                .await
                .unwrap(),
        );

        let mut buf = vec![0u8; 1024 * 1024];

        loop {
            let n = remote_file.read(&mut buf).await.unwrap();

            if n == 0 {
                break;
            }

            local_file.write_all(&buf[..n]).await.unwrap();
        }

        local_file.flush().await.unwrap();
        remote_file.into_inner().close().await.unwrap();

        output_paths.push(local_file_path.to_string_lossy().to_string());
    }
    output_paths
}
