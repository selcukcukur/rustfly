use async_trait::async_trait;
use bytes::Bytes;

#[async_trait]
pub trait RustflyAdapter: Send + Sync {
  type Error;

  async fn read(&self, path: &str) -> Result<Bytes, Self::Error>;

  async fn write(&self, path: &str, contents: Bytes) -> Result<(), Self::Error>;

  async fn delete(&self, path: &str) -> Result<(), Self::Error>;

  async fn exists(&self, path: &str) -> Result<bool, Self::Error>;
}