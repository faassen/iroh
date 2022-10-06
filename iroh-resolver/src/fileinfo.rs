use anyhow::Result;
use std::path::{Path, PathBuf};

use async_recursion::async_recursion;
use async_stream::try_stream;
use futures::stream::StreamExt;
use futures::stream::TryStreamExt;
use futures::Stream;

#[derive(Debug, PartialEq)]
enum FileInfo {
    File(PathBuf, u64),
    Dir(PathBuf),
}

// #[derive(Debug, PartialEq)]
// struct FileInfo {
//     path: PathBuf,
//     size: u64,
// }

fn get_file_info_stream<'a>(path: &'a Path) -> impl Stream<Item = FileInfo> + 'a {
    async_stream::stream! {
        if path.is_file() {
            yield FileInfo::File(path.to_path_buf(), 0);
        } else if path.is_dir() {
            let directory_reader = tokio::fs::read_dir(path).await.unwrap();
            let read_dir_stream = tokio_stream::wrappers::ReadDirStream::new(directory_reader);

            for await dir_entry in read_dir_stream {
                let dir_entry = dir_entry.unwrap();
                let path = dir_entry.path();
                if path.is_file() {
                    let size = dir_entry.metadata().await.unwrap().len();
                    yield FileInfo::File(path, size);
                } else {
                    continue
                }
            }
        } else {
            panic!("not a file or directory");
        }

    }
}

// fn get_file_info<'a>(path: &'a Path) -> Box<dyn Stream<Item = FileInfo> + 'a> {
//     if path.is_file() {
//         get_file(path)
//     } else {
//         get_dir(path)
//     }

// } else {
//     anyhow::bail!("not a file or directory");
// }

// }
// while let Some(dir_entry) = read_dir_stream.next().await {}

// async_stream::stream! {
//     yield FileInfo { path: path.to_path_buf(), size: 0 };
// }
// let mut dir = DirectoryBuilder::new();
// dir.name(
//     path.file_name()
//         .and_then(|s| s.to_str())
//         .unwrap_or_default(),
// );
// if recursive {
//     dir.recursive();
// }
// let mut directory_reader = tokio::fs::read_dir(path.clone()).await?;
// while let Some(entry) = directory_reader.next_entry().await? {
//     let path = entry.path();
//     if path.is_file() {
//         let f = FileBuilder::new().path(path).build().await?;
//         dir.add_file(f);
//     } else if path.is_dir() {
//         let d = make_dir_from_path(path, recursive).await?;
//         dir.add_dir(d)?;
//     } else {
//         anyhow::bail!("directory entry is neither file nor directory")
//     }
// }
// dir.build()
// }

#[cfg(test)]
mod tests {
    use super::*;
    use futures::StreamExt;

    #[tokio::test]
    async fn test_get_file_info_stream() {
        let path = Path::new("fixtures/root");
        let stream = get_file_info_stream(path);
        let data = stream.collect::<Vec<FileInfo>>().await;

        assert_eq!(
            data,
            vec![FileInfo::File(PathBuf::from("fixtures/root/a.txt"), 1)]
        );
    }
}
