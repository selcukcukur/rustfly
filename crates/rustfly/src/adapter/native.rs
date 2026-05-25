use async_trait::async_trait;
use bytes::Bytes;
use std::path::PathBuf;
use tokio::fs;
use crate::adapter::adapter::RustflyAdapter;

pub struct NativeAdapter {
  root: PathBuf,
}

impl NativeAdapter {
  pub fn new(root: impl Into<PathBuf>) -> Self {
    Self {
      root: root.into(),
    }
  }

  fn full_path(&self, path: &str) -> PathBuf {
    self.root.join(path)
  }
}

#[async_trait]
impl RustflyAdapter for NativeAdapter {
  type Error = std::io::Error;

  async fn read(&self, path: &str) -> Result<Bytes, Self::Error> {
    let full_path = self.full_path(path);
    let data = fs::read(full_path).await?;
    Ok(Bytes::from(data))
  }

  async fn write(&self, path: &str, contents: Bytes) -> Result<(), Self::Error> {
    let full_path = self.full_path(path);
    fs::write(full_path, &contents).await?;
    Ok(())
  }

  async fn delete(&self, path: &str) -> Result<(), Self::Error> {
    let full_path = self.full_path(path);

    if fs::metadata(&full_path).await.map(|m| m.is_dir()).unwrap_or(false) {
      fs::remove_dir_all(full_path).await?;
    } else {
      fs::remove_file(full_path).await?;
    }

    Ok(())
  }

  async fn exists(&self, path: &str) -> Result<bool, Self::Error> {
    let full_path = self.full_path(path);
    Ok(fs::metadata(full_path).await.is_ok())
  }
}