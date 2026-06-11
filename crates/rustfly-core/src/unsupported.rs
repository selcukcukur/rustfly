use async_trait::async_trait;
use bytes::Bytes;

use crate::{Metadata, Result, RustflyAdapter, RustflyError};

/// Adapter helper for driver crates whose public crate surface is ready before
/// their transport-specific implementation is linked in.
///
/// This is useful for feature-gated first-party drivers: enabling a feature can
/// register a named driver immediately while returning explicit unsupported
/// operation errors until the concrete adapter replaces it.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct UnsupportedAdapter {
    driver: &'static str,
}

impl UnsupportedAdapter {
    /// Create an unsupported adapter placeholder for a driver name.
    pub const fn new(driver: &'static str) -> Self {
        Self { driver }
    }

    /// Return the driver name associated with this placeholder.
    pub const fn driver(&self) -> &'static str {
        self.driver
    }

    fn unsupported(&self, operation: &'static str) -> RustflyError {
        let _ = self.driver;
        RustflyError::Unsupported(operation)
    }
}

#[async_trait]
impl RustflyAdapter for UnsupportedAdapter {
    fn read_sync(&self, _path: &str) -> Result<Bytes> {
        Err(self.unsupported("read_sync"))
    }

    async fn read(&self, _path: &str) -> Result<Bytes> {
        Err(self.unsupported("read"))
    }

    fn write_sync(&self, _path: &str, _contents: Bytes) -> Result<()> {
        Err(self.unsupported("write_sync"))
    }

    async fn write(&self, _path: &str, _contents: Bytes) -> Result<()> {
        Err(self.unsupported("write"))
    }

    fn delete_sync(&self, _path: &str) -> Result<()> {
        Err(self.unsupported("delete_sync"))
    }

    async fn delete(&self, _path: &str) -> Result<()> {
        Err(self.unsupported("delete"))
    }

    fn exists_sync(&self, _path: &str) -> Result<bool> {
        Err(self.unsupported("exists_sync"))
    }

    async fn exists(&self, _path: &str) -> Result<bool> {
        Err(self.unsupported("exists"))
    }

    fn create_dir_sync(&self, _path: &str) -> Result<()> {
        Err(self.unsupported("create_dir_sync"))
    }

    async fn create_dir(&self, _path: &str) -> Result<()> {
        Err(self.unsupported("create_dir"))
    }

    fn list_sync(&self, _path: &str) -> Result<Vec<Metadata>> {
        Err(self.unsupported("list_sync"))
    }

    async fn list(&self, _path: &str) -> Result<Vec<Metadata>> {
        Err(self.unsupported("list"))
    }

    fn metadata_sync(&self, _path: &str) -> Result<Metadata> {
        Err(self.unsupported("metadata_sync"))
    }

    async fn metadata(&self, _path: &str) -> Result<Metadata> {
        Err(self.unsupported("metadata"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn reports_unsupported_async_operations() {
        let adapter = UnsupportedAdapter::new("s3");

        assert!(matches!(
            adapter.read("file.txt").await,
            Err(RustflyError::Unsupported("read"))
        ));
    }

    #[test]
    fn reports_unsupported_sync_operations() {
        let adapter = UnsupportedAdapter::new("s3");

        assert_eq!(adapter.driver(), "s3");
        assert!(matches!(
            adapter.read_sync("file.txt"),
            Err(RustflyError::Unsupported("read_sync"))
        ));
    }
}
