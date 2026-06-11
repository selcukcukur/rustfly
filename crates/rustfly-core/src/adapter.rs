use async_trait::async_trait;
use bytes::Bytes;
use std::collections::VecDeque;

use crate::definition::{Metadata, Result, RustflyError};

/// Thread-safe filesystem adapter contract implemented by local, memory,
/// remote, cloud, archive, and database-backed Rustfly drivers.
///
/// Implementors provide a common surface for both async and sync consumers.
/// Async operations are the primary API; sync methods can be implemented
/// natively when the underlying transport supports blocking calls, or left as
/// explicit unsupported operations.
#[async_trait]
pub trait RustflyAdapter: Send + Sync {
    /// Read the entire file contents using a blocking call.
    fn read_sync(&self, path: &str) -> Result<Bytes> {
        let _ = path;
        Err(RustflyError::Unsupported("read_sync"))
    }

    /// Read the entire file contents asynchronously.
    async fn read(&self, path: &str) -> Result<Bytes>;

    /// Write file contents using a blocking call, replacing existing contents.
    fn write_sync(&self, path: &str, contents: Bytes) -> Result<()> {
        let _ = path;
        let _ = contents;
        Err(RustflyError::Unsupported("write_sync"))
    }

    /// Write file contents asynchronously, replacing existing contents.
    async fn write(&self, path: &str, contents: Bytes) -> Result<()>;

    /// Delete a file or directory using a blocking call.
    fn delete_sync(&self, path: &str) -> Result<()> {
        let _ = path;
        Err(RustflyError::Unsupported("delete_sync"))
    }

    /// Delete a file or directory asynchronously.
    async fn delete(&self, path: &str) -> Result<()>;

    /// Check whether a path exists using a blocking call.
    fn exists_sync(&self, path: &str) -> Result<bool> {
        let _ = path;
        Err(RustflyError::Unsupported("exists_sync"))
    }

    /// Check whether a path exists asynchronously.
    async fn exists(&self, path: &str) -> Result<bool>;

    /// Create a directory and any missing parents using a blocking call.
    fn create_dir_sync(&self, path: &str) -> Result<()> {
        let _ = path;
        Err(RustflyError::Unsupported("create_dir_sync"))
    }

    /// Create a directory and any missing parents asynchronously.
    async fn create_dir(&self, path: &str) -> Result<()> {
        let _ = path;
        Err(RustflyError::Unsupported("create_dir"))
    }

    /// List immediate children of a directory using a blocking call.
    fn list_sync(&self, path: &str) -> Result<Vec<Metadata>> {
        let _ = path;
        Err(RustflyError::Unsupported("list_sync"))
    }

    /// List immediate children of a directory asynchronously.
    async fn list(&self, path: &str) -> Result<Vec<Metadata>> {
        let _ = path;
        Err(RustflyError::Unsupported("list"))
    }

    /// Recursively list descendants using the blocking `list_sync` operation.
    fn list_recursive_sync(&self, path: &str) -> Result<Vec<Metadata>> {
        let mut entries = Vec::new();
        let mut pending = VecDeque::from([path.to_string()]);

        while let Some(current) = pending.pop_front() {
            for entry in self.list_sync(&current)? {
                if entry.is_directory() {
                    pending.push_back(entry.path().to_string());
                }

                entries.push(entry);
            }
        }

        Ok(entries)
    }

    /// Recursively list descendants using the async `list` operation.
    async fn list_recursive(&self, path: &str) -> Result<Vec<Metadata>> {
        let mut entries = Vec::new();
        let mut pending = VecDeque::from([path.to_string()]);

        while let Some(current) = pending.pop_front() {
            for entry in self.list(&current).await? {
                if entry.is_directory() {
                    pending.push_back(entry.path().to_string());
                }

                entries.push(entry);
            }
        }

        Ok(entries)
    }

    /// Read path metadata using a blocking call.
    fn metadata_sync(&self, path: &str) -> Result<Metadata> {
        let _ = path;
        Err(RustflyError::Unsupported("metadata_sync"))
    }

    /// Read path metadata asynchronously.
    async fn metadata(&self, path: &str) -> Result<Metadata> {
        let _ = path;
        Err(RustflyError::Unsupported("metadata"))
    }

    /// Copy a file using blocking calls.
    fn copy_sync(&self, from: &str, to: &str) -> Result<()> {
        let contents = self.read_sync(from)?;
        self.write_sync(to, contents)
    }

    /// Copy a file asynchronously.
    async fn copy(&self, from: &str, to: &str) -> Result<()> {
        let contents = self.read(from).await?;
        self.write(to, contents).await
    }

    /// Move a file using blocking calls.
    fn move_file_sync(&self, from: &str, to: &str) -> Result<()> {
        self.copy_sync(from, to)?;
        self.delete_sync(from)
    }

    /// Move a file asynchronously.
    async fn move_file(&self, from: &str, to: &str) -> Result<()> {
        self.copy(from, to).await?;
        self.delete(from).await
    }
}
