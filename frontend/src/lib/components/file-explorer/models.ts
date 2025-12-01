export interface FSTree {
  path: string,
  info: PathInfo,
  children: FSTree[]
}

export interface PathInfo {
  size: number,
  blocks: number,
  nlink: string,
  creation: Date,
  modified: Date,
  file_type: FileType,
}


export enum FileType {
  Directory,
  File,
  Symlink,

  CharDevice,
  BlockDevice,
  NamedPipe,
  Socket,
  Unknown,
}

