use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct RustflyPath {
  inner: PathBuf,
}

impl RustflyPath {
  /// Create a safe path inside a given root directory.
  pub fn new(root: &Path, input: &str) -> Result<Self, String> {
    let joined = root.join(input);

    let canonical = joined
      .canonicalize()
      .map_err(|_| "invalid path".to_string())?;

    let root_canonical = root
      .canonicalize()
      .map_err(|_| "invalid root path".to_string())?;

    if !canonical.starts_with(&root_canonical) {
      return Err("path traversal detected".to_string());
    }

    Ok(Self { inner: canonical })
  }

  /// Borrow as Path
  pub fn as_path(&self) -> &Path {
    &self.inner
  }

  /// Consume and return PathBuf
  pub fn into_path_buf(self) -> PathBuf {
    self.inner
  }

  /// String representation (lossy)
  pub fn to_string(&self) -> String {
    self.inner.to_string_lossy().to_string()
  }
}