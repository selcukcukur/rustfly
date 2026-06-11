use std::path::{Component, Path, PathBuf};

use crate::definition::{Result, RustflyError};

#[derive(Debug, Clone)]
pub struct RustflyPath {
    inner: PathBuf,
}

impl RustflyPath {
    pub fn new(root: &Path, input: &str) -> Result<Self> {
        let relative = normalize_relative(input)?;
        Ok(Self {
            inner: root.join(relative),
        })
    }

    pub fn new_allow_root(root: &Path, input: &str) -> Result<Self> {
        let relative = normalize_relative_allow_root(input)?;
        Ok(Self {
            inner: root.join(relative),
        })
    }

    pub fn normalize(input: &str) -> Result<PathBuf> {
        normalize_relative(input)
    }

    pub fn normalize_allow_root(input: &str) -> Result<PathBuf> {
        normalize_relative_allow_root(input)
    }

    pub fn storage_key(input: &str) -> Result<String> {
        let path = normalize_relative_allow_root(input)?;
        let key = path
            .components()
            .map(|component| component.as_os_str().to_string_lossy())
            .collect::<Vec<_>>()
            .join("/");

        Ok(key)
    }

    pub fn as_path(&self) -> &Path {
        &self.inner
    }

    pub fn into_path_buf(self) -> PathBuf {
        self.inner
    }
}

fn normalize_relative(input: &str) -> Result<PathBuf> {
    let normalized = normalize_relative_allow_root(input)?;

    if normalized.as_os_str().is_empty() {
        return Err(RustflyError::InvalidPath(input.to_string()));
    }

    Ok(normalized)
}

fn normalize_relative_allow_root(input: &str) -> Result<PathBuf> {
    let mut normalized = PathBuf::new();

    for component in Path::new(input).components() {
        match component {
            Component::Normal(value) => normalized.push(value),
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(RustflyError::InvalidPath(input.to_string()));
            }
        }
    }

    Ok(normalized)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_traversal() {
        assert!(RustflyPath::normalize("../secret.txt").is_err());
    }

    #[test]
    fn normalizes_storage_keys_to_forward_slashes() {
        assert_eq!(
            RustflyPath::storage_key("folder/file.txt").unwrap(),
            "folder/file.txt"
        );
    }

    #[test]
    fn allows_root_for_root_safe_operations() {
        assert_eq!(RustflyPath::storage_key("").unwrap(), "");
        assert!(RustflyPath::normalize("").is_err());
        assert!(
            RustflyPath::normalize_allow_root("")
                .unwrap()
                .as_os_str()
                .is_empty()
        );
    }
}
