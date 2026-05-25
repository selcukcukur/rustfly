use std::sync::Arc;
use bytes::Bytes;
use crate::adapter::adapter::RustflyAdapter;

pub struct RustflyOperator<A: RustflyAdapter> {
  adapter: Arc<A>,
}

impl<A> RustflyOperator<A>
where
  A: RustflyAdapter,
{
  pub fn new(adapter: A) -> Self {
    Self {
      adapter: Arc::new(adapter),
    }
  }

  pub async fn read(&self, path: &str) -> Result<Bytes, A::Error> {
    self.adapter.read(path).await
  }

  pub async fn write(&self, path: &str, data: Bytes) -> Result<(), A::Error> {
    self.adapter.write(path, data).await
  }

  pub async fn delete(&self, path: &str) -> Result<(), A::Error> {
    self.adapter.delete(path).await
  }

  pub async fn exists(&self, path: &str) -> Result<bool, A::Error> {
    self.adapter.exists(path).await
  }
}

