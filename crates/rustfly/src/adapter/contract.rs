use async_trait::async_trait;
use bytes::Bytes;

use crate::definition::{Metadata, Result, RustflyError};

#[async_trait]
pub trait RustflyAdapter: Send + Sync {
    fn read_sync(&self, path: &str) -> Result<Bytes> {
        let _ = path;
        Err(RustflyError::Unsupported("read_sync"))
    }

    async fn read(&self, path: &str) -> Result<Bytes>;

    fn write_sync(&self, path: &str, contents: Bytes) -> Result<()> {
        let _ = path;
        let _ = contents;
        Err(RustflyError::Unsupported("write_sync"))
    }

    async fn write(&self, path: &str, contents: Bytes) -> Result<()>;

    fn delete_sync(&self, path: &str) -> Result<()> {
        let _ = path;
        Err(RustflyError::Unsupported("delete_sync"))
    }

    async fn delete(&self, path: &str) -> Result<()>;

    fn exists_sync(&self, path: &str) -> Result<bool> {
        let _ = path;
        Err(RustflyError::Unsupported("exists_sync"))
    }

    async fn exists(&self, path: &str) -> Result<bool>;

    fn create_dir_sync(&self, path: &str) -> Result<()> {
        let _ = path;
        Err(RustflyError::Unsupported("create_dir_sync"))
    }

    async fn create_dir(&self, path: &str) -> Result<()> {
        let _ = path;
        Err(RustflyError::Unsupported("create_dir"))
    }

    fn list_sync(&self, path: &str) -> Result<Vec<Metadata>> {
        let _ = path;
        Err(RustflyError::Unsupported("list_sync"))
    }

    async fn list(&self, path: &str) -> Result<Vec<Metadata>> {
        let _ = path;
        Err(RustflyError::Unsupported("list"))
    }

    fn metadata_sync(&self, path: &str) -> Result<Metadata> {
        let _ = path;
        Err(RustflyError::Unsupported("metadata_sync"))
    }

    async fn metadata(&self, path: &str) -> Result<Metadata> {
        let _ = path;
        Err(RustflyError::Unsupported("metadata"))
    }

    fn copy_sync(&self, from: &str, to: &str) -> Result<()> {
        let contents = self.read_sync(from)?;
        self.write_sync(to, contents)
    }

    async fn copy(&self, from: &str, to: &str) -> Result<()> {
        let contents = self.read(from).await?;
        self.write(to, contents).await
    }

    fn move_file_sync(&self, from: &str, to: &str) -> Result<()> {
        self.copy_sync(from, to)?;
        self.delete_sync(from)
    }

    async fn move_file(&self, from: &str, to: &str) -> Result<()> {
        self.copy(from, to).await?;
        self.delete(from).await
    }
}
