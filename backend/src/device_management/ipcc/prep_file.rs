use async_zip::{tokio::write::ZipFileWriter, Compression, ZipEntryBuilder};
use futures_util::StreamExt;
use tokio::{
    fs::File,
    io::{duplex, AsyncReadExt, AsyncWriteExt},
};

use tokio_tar::{Archive, EntryType};

fn adjust_path(path: &str) -> String {
    if !path.contains(".bundle") {
        return format!("Payload/{}", path);
    }

    let mut out = String::with_capacity(8 + path.len()); // "Payload/" + rest
    out.push_str("Payload/");

    for (i, part) in path.split('/').enumerate() {
        if i > 0 {
            out.push('/');
        }

        if part.ends_with(".bundle") {
            out.push_str("default.bundle");
        } else {
            out.push_str(part);
        }
    }

    out
}

pub async fn repack_tar_to_zip(tar_data: &[u8]) -> Vec<u8> {
    let (mut tar_writer, tar_reader) = duplex(1024 * 1024);
    tar_writer.write_all(tar_data).await.unwrap();

    let mut archive = Archive::new(tar_reader);

    let (mut zip_writer, mut zip_reader) = duplex(1024 * 1024);
    let mut writer = ZipFileWriter::with_tokio(&mut zip_writer);

    let mut entries = archive.entries().expect("Invalid TAR archive");

    while let Some(Ok(mut entry)) = entries.next().await {
        let path = entry.path().unwrap_or_default();
        let path_str = path.to_string_lossy();
        let adjusted = adjust_path(&path_str);

        let mut data = Vec::new();
        entry.read_to_end(&mut data).await.unwrap();

        let builder = ZipEntryBuilder::new(adjusted.into(), Compression::Deflate);
        writer.write_entry_whole(builder, &data).await.unwrap();
        //
        // let mut buffer = [0u8; 8192];
        // loop {
        //     let n = entry.read(&mut buffer).await.unwrap();
        //     if n == 0 {
        //         break;
        //     }
        //     entry_writer
        //         .compat_write()
        //         .write_all(&buffer[..n])
        //         .await
        //         .unwrap();
        // }
        //
        // entry_writer.close().await.unwrap();
    }

    writer.close().await.unwrap();
    drop(zip_writer); // signals EOF to zip_reader

    let mut buf = Vec::new();
    zip_reader.read_to_end(&mut buf).await.unwrap();
    buf
}
