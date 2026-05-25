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
}