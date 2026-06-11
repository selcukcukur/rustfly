use bytes::Bytes;
use std::sync::Arc;

use crate::adapter::contract::RustflyAdapter;
use crate::definition::{EntryKind, Metadata, Result};

#[derive(Clone)]
pub struct Filesystem {
    driver_name: String,
    adapter: Arc<dyn RustflyAdapter>,
}

pub type RustflyOperator = Filesystem;

impl Filesystem {
    pub fn new(driver_name: impl Into<String>, adapter: Arc<dyn RustflyAdapter>) -> Self {
        Self {
            driver_name: driver_name.into(),
            adapter,
        }
    }

    pub fn driver_name(&self) -> &str {
        &self.driver_name
    }

    pub async fn read(&self, path: &str) -> Result<Bytes> {
        self.adapter.read(path).await
    }

    pub fn read_sync(&self, path: &str) -> Result<Bytes> {
        self.adapter.read_sync(path)
    }

    pub async fn get(&self, path: &str) -> Result<Bytes> {
        self.read(path).await
    }

    pub fn get_sync(&self, path: &str) -> Result<Bytes> {
        self.read_sync(path)
    }

    pub async fn write(&self, path: &str, contents: impl Into<Bytes> + Send) -> Result<()> {
        self.adapter.write(path, contents.into()).await
    }

    pub fn write_sync(&self, path: &str, contents: impl Into<Bytes>) -> Result<()> {
        self.adapter.write_sync(path, contents.into())
    }

    pub async fn put(&self, path: &str, contents: impl Into<Bytes> + Send) -> Result<()> {
        self.write(path, contents).await
    }

    pub fn put_sync(&self, path: &str, contents: impl Into<Bytes>) -> Result<()> {
        self.write_sync(path, contents)
    }

    pub async fn delete(&self, path: &str) -> Result<()> {
        self.adapter.delete(path).await
    }

    pub fn delete_sync(&self, path: &str) -> Result<()> {
        self.adapter.delete_sync(path)
    }

    pub async fn exists(&self, path: &str) -> Result<bool> {
        self.adapter.exists(path).await
    }

    pub fn exists_sync(&self, path: &str) -> Result<bool> {
        self.adapter.exists_sync(path)
    }

    pub async fn create_dir(&self, path: &str) -> Result<()> {
        self.adapter.create_dir(path).await
    }

    pub fn create_dir_sync(&self, path: &str) -> Result<()> {
        self.adapter.create_dir_sync(path)
    }

    pub async fn list(&self, path: &str) -> Result<Vec<Metadata>> {
        self.adapter.list(path).await
    }

    pub fn list_sync(&self, path: &str) -> Result<Vec<Metadata>> {
        self.adapter.list_sync(path)
    }

    pub async fn files(&self, path: &str) -> Result<Vec<Metadata>> {
        filter_by_kind(self.list(path).await?, EntryKind::File)
    }

    pub fn files_sync(&self, path: &str) -> Result<Vec<Metadata>> {
        filter_by_kind(self.list_sync(path)?, EntryKind::File)
    }

    pub async fn directories(&self, path: &str) -> Result<Vec<Metadata>> {
        filter_by_kind(self.list(path).await?, EntryKind::Directory)
    }

    pub fn directories_sync(&self, path: &str) -> Result<Vec<Metadata>> {
        filter_by_kind(self.list_sync(path)?, EntryKind::Directory)
    }

    pub async fn metadata(&self, path: &str) -> Result<Metadata> {
        self.adapter.metadata(path).await
    }

    pub fn metadata_sync(&self, path: &str) -> Result<Metadata> {
        self.adapter.metadata_sync(path)
    }

    pub async fn copy(&self, from: &str, to: &str) -> Result<()> {
        self.adapter.copy(from, to).await
    }

    pub fn copy_sync(&self, from: &str, to: &str) -> Result<()> {
        self.adapter.copy_sync(from, to)
    }

    pub async fn move_file(&self, from: &str, to: &str) -> Result<()> {
        self.adapter.move_file(from, to).await
    }

    pub fn move_file_sync(&self, from: &str, to: &str) -> Result<()> {
        self.adapter.move_file_sync(from, to)
    }
}

fn filter_by_kind(entries: Vec<Metadata>, kind: EntryKind) -> Result<Vec<Metadata>> {
    Ok(entries
        .into_iter()
        .filter(|entry| entry.kind() == kind)
        .collect())
}
