use async_trait::async_trait;
use bytes::Bytes;
use std::fs as sync_fs;
use std::io::ErrorKind;
use std::path::PathBuf;
use tokio::fs;

use crate::adapter::contract::RustflyAdapter;
use crate::definition::{EntryKind, Metadata, Result, RustflyError};
use crate::path::RustflyPath;

#[derive(Debug, Clone)]
pub struct NativeAdapter {
    root: PathBuf,
}

impl NativeAdapter {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    fn full_path(&self, path: &str) -> Result<PathBuf> {
        Ok(RustflyPath::new(&self.root, path)?.into_path_buf())
    }

    fn full_path_allow_root(&self, path: &str) -> Result<PathBuf> {
        Ok(RustflyPath::new_allow_root(&self.root, path)?.into_path_buf())
    }

    async fn metadata_for(&self, storage_path: &str, full_path: PathBuf) -> Result<Metadata> {
        let metadata = fs::metadata(full_path).await?;
        let kind = if metadata.is_dir() {
            EntryKind::Directory
        } else {
            EntryKind::File
        };

        Ok(Metadata::new(
            storage_path,
            kind,
            metadata.len(),
            metadata.modified().ok(),
        ))
    }

    fn metadata_for_sync(&self, storage_path: &str, full_path: PathBuf) -> Result<Metadata> {
        let metadata = sync_fs::metadata(full_path)?;
        let kind = if metadata.is_dir() {
            EntryKind::Directory
        } else {
            EntryKind::File
        };

        Ok(Metadata::new(
            storage_path,
            kind,
            metadata.len(),
            metadata.modified().ok(),
        ))
    }
}

#[async_trait]
impl RustflyAdapter for NativeAdapter {
    fn read_sync(&self, path: &str) -> Result<Bytes> {
        let full_path = self.full_path(path)?;
        let data = sync_fs::read(full_path)?;
        Ok(Bytes::from(data))
    }

    async fn read(&self, path: &str) -> Result<Bytes> {
        let full_path = self.full_path(path)?;
        let data = fs::read(full_path).await?;
        Ok(Bytes::from(data))
    }

    fn write_sync(&self, path: &str, contents: Bytes) -> Result<()> {
        let full_path = self.full_path(path)?;

        if let Some(parent) = full_path.parent() {
            sync_fs::create_dir_all(parent)?;
        }

        sync_fs::write(full_path, &contents)?;
        Ok(())
    }

    async fn write(&self, path: &str, contents: Bytes) -> Result<()> {
        let full_path = self.full_path(path)?;

        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        fs::write(full_path, &contents).await?;
        Ok(())
    }

    fn delete_sync(&self, path: &str) -> Result<()> {
        let full_path = self.full_path(path)?;

        if sync_fs::metadata(&full_path)
            .map(|m| m.is_dir())
            .unwrap_or(false)
        {
            sync_fs::remove_dir_all(full_path)?;
        } else {
            sync_fs::remove_file(full_path)?;
        }

        Ok(())
    }

    async fn delete(&self, path: &str) -> Result<()> {
        let full_path = self.full_path(path)?;

        if fs::metadata(&full_path)
            .await
            .map(|m| m.is_dir())
            .unwrap_or(false)
        {
            fs::remove_dir_all(full_path).await?;
        } else {
            fs::remove_file(full_path).await?;
        }

        Ok(())
    }

    fn exists_sync(&self, path: &str) -> Result<bool> {
        let full_path = self.full_path_allow_root(path)?;
        Ok(sync_fs::metadata(full_path).is_ok())
    }

    async fn exists(&self, path: &str) -> Result<bool> {
        let full_path = self.full_path_allow_root(path)?;
        Ok(fs::metadata(full_path).await.is_ok())
    }

    fn create_dir_sync(&self, path: &str) -> Result<()> {
        let full_path = self.full_path_allow_root(path)?;
        sync_fs::create_dir_all(full_path)?;
        Ok(())
    }

    async fn create_dir(&self, path: &str) -> Result<()> {
        let full_path = self.full_path_allow_root(path)?;
        fs::create_dir_all(full_path).await?;
        Ok(())
    }

    fn list_sync(&self, path: &str) -> Result<Vec<Metadata>> {
        let full_path = self.full_path_allow_root(path)?;
        let mut entries = Vec::new();

        for entry in sync_fs::read_dir(full_path)? {
            let entry = entry?;
            let file_name = entry.file_name().to_string_lossy().to_string();
            let parent = RustflyPath::storage_key(path)?;
            let storage_path = join_storage_key(&parent, &file_name);
            entries.push(self.metadata_for_sync(&storage_path, entry.path())?);
        }

        Ok(entries)
    }

    async fn list(&self, path: &str) -> Result<Vec<Metadata>> {
        let full_path = self.full_path_allow_root(path)?;
        let mut reader = fs::read_dir(full_path).await?;
        let mut entries = Vec::new();

        while let Some(entry) = reader.next_entry().await? {
            let file_name = entry.file_name().to_string_lossy().to_string();
            let parent = RustflyPath::storage_key(path)?;
            let storage_path = join_storage_key(&parent, &file_name);
            entries.push(self.metadata_for(&storage_path, entry.path()).await?);
        }

        Ok(entries)
    }

    fn metadata_sync(&self, path: &str) -> Result<Metadata> {
        let storage_path = RustflyPath::storage_key(path)?;
        let full_path = self.full_path_allow_root(path)?;
        self.metadata_for_sync(&storage_path, full_path)
    }

    async fn metadata(&self, path: &str) -> Result<Metadata> {
        let storage_path = RustflyPath::storage_key(path)?;
        let full_path = self.full_path_allow_root(path)?;
        self.metadata_for(&storage_path, full_path).await
    }

    fn copy_sync(&self, from: &str, to: &str) -> Result<()> {
        let from = self.full_path(from)?;
        let to = self.full_path(to)?;

        if let Some(parent) = to.parent() {
            sync_fs::create_dir_all(parent)?;
        }

        sync_fs::copy(from, to)?;
        Ok(())
    }

    async fn copy(&self, from: &str, to: &str) -> Result<()> {
        let from = self.full_path(from)?;
        let to = self.full_path(to)?;

        if let Some(parent) = to.parent() {
            fs::create_dir_all(parent).await?;
        }

        fs::copy(from, to).await?;
        Ok(())
    }

    fn move_file_sync(&self, from: &str, to: &str) -> Result<()> {
        let from = self.full_path(from)?;
        let to = self.full_path(to)?;

        if let Some(parent) = to.parent() {
            sync_fs::create_dir_all(parent)?;
        }

        match sync_fs::rename(&from, &to) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == ErrorKind::CrossesDevices => {
                sync_fs::copy(&from, &to)?;
                sync_fs::remove_file(from)?;
                Ok(())
            }
            Err(error) => Err(RustflyError::Io(error)),
        }
    }

    async fn move_file(&self, from: &str, to: &str) -> Result<()> {
        let from = self.full_path(from)?;
        let to = self.full_path(to)?;

        if let Some(parent) = to.parent() {
            fs::create_dir_all(parent).await?;
        }

        match fs::rename(&from, &to).await {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == ErrorKind::CrossesDevices => {
                fs::copy(&from, &to).await?;
                fs::remove_file(from).await?;
                Ok(())
            }
            Err(error) => Err(RustflyError::Io(error)),
        }
    }
}

fn join_storage_key(parent: &str, child: &str) -> String {
    if parent.is_empty() {
        child.to_string()
    } else {
        format!("{parent}/{child}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn make_adapter() -> (NativeAdapter, tempfile::TempDir) {
        let dir = tempdir().unwrap();
        let adapter = NativeAdapter::new(dir.path().to_path_buf());
        (adapter, dir)
    }

    #[tokio::test]
    async fn test_write_and_read() {
        let (adapter, _dir) = make_adapter();

        let path = "test.txt";
        let data = Bytes::from("hello world");

        adapter.write(path, data.clone()).await.unwrap();
        let result = adapter.read(path).await.unwrap();

        assert_eq!(result, data);
    }

    #[tokio::test]
    async fn test_write_creates_parent_directories() {
        let (adapter, _dir) = make_adapter();

        adapter
            .write("nested/file.txt", Bytes::from("data"))
            .await
            .unwrap();

        assert!(adapter.exists("nested/file.txt").await.unwrap());
    }

    #[tokio::test]
    async fn test_exists_true() {
        let (adapter, _dir) = make_adapter();

        let path = "exists.txt";
        adapter.write(path, Bytes::from("data")).await.unwrap();

        assert!(adapter.exists(path).await.unwrap());
    }

    #[tokio::test]
    async fn test_exists_false() {
        let (adapter, _dir) = make_adapter();

        assert!(!adapter.exists("missing.txt").await.unwrap());
    }

    #[tokio::test]
    async fn test_delete_file() {
        let (adapter, _dir) = make_adapter();

        let path = "delete.txt";
        adapter.write(path, Bytes::from("data")).await.unwrap();

        adapter.delete(path).await.unwrap();

        assert!(!adapter.exists(path).await.unwrap());
    }

    #[tokio::test]
    async fn test_delete_directory() {
        let (adapter, dir) = make_adapter();

        let dir_path = dir.path().join("folder");
        fs::create_dir_all(&dir_path).unwrap();

        let file_path = dir_path.join("file.txt");
        fs::write(&file_path, b"content").unwrap();

        adapter.delete("folder").await.unwrap();

        assert!(!dir_path.exists());
    }

    #[tokio::test]
    async fn test_rejects_path_traversal() {
        let (adapter, _dir) = make_adapter();

        assert!(
            adapter
                .write("../escape.txt", Bytes::from("data"))
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn test_copy_move_list_and_metadata() {
        let (adapter, _dir) = make_adapter();

        adapter
            .write("folder/source.txt", Bytes::from("data"))
            .await
            .unwrap();
        adapter
            .copy("folder/source.txt", "folder/copy.txt")
            .await
            .unwrap();
        adapter
            .move_file("folder/copy.txt", "folder/moved.txt")
            .await
            .unwrap();

        let metadata = adapter.metadata("folder/moved.txt").await.unwrap();
        let entries = adapter.list("folder").await.unwrap();

        assert_eq!(metadata.kind(), EntryKind::File);
        assert!(
            entries
                .iter()
                .any(|entry| entry.path() == "folder/source.txt")
        );
        assert!(
            entries
                .iter()
                .any(|entry| entry.path() == "folder/moved.txt")
        );
    }

    #[tokio::test]
    async fn test_lists_root_without_leading_separator() {
        let (adapter, _dir) = make_adapter();

        adapter
            .write("root.txt", Bytes::from("data"))
            .await
            .unwrap();

        let metadata = adapter.metadata("").await.unwrap();
        let entries = adapter.list("").await.unwrap();

        assert_eq!(metadata.kind(), EntryKind::Directory);
        assert!(entries.iter().any(|entry| entry.path() == "root.txt"));
    }

    #[test]
    fn test_sync_api_write_read_and_delete() {
        let (adapter, _dir) = make_adapter();

        adapter
            .write_sync("sync/file.txt", Bytes::from("data"))
            .unwrap();
        assert_eq!(
            adapter.read_sync("sync/file.txt").unwrap(),
            Bytes::from("data")
        );
        assert!(adapter.exists_sync("sync/file.txt").unwrap());

        adapter.delete_sync("sync/file.txt").unwrap();
        assert!(!adapter.exists_sync("sync/file.txt").unwrap());
    }
}
