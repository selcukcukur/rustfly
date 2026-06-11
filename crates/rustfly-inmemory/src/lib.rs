use std::collections::{HashMap, HashSet};
use std::sync::RwLock;

use async_trait::async_trait;
use bytes::Bytes;
use rustfly_core::{EntryKind, Metadata, Result, RustflyAdapter, RustflyError, RustflyPath};

#[derive(Debug, Default)]
/// In-memory adapter for tests, ephemeral storage, and local development.
///
/// Files and directory markers are kept behind `RwLock`s so the adapter can be
/// shared safely across threads while preserving the same sync and async
/// contract as remote adapters.
pub struct InMemoryAdapter {
    files: RwLock<HashMap<String, Bytes>>,
    directories: RwLock<HashSet<String>>,
}

impl InMemoryAdapter {
    /// Create an empty in-memory adapter.
    pub fn new() -> Self {
        Self::default()
    }

    fn key(path: &str) -> Result<String> {
        RustflyPath::storage_key(path)
    }

    fn key_required(path: &str) -> Result<String> {
        let key = Self::key(path)?;

        if key.is_empty() {
            return Err(RustflyError::InvalidPath(path.to_string()));
        }

        Ok(key)
    }

    fn remember_parent_directories(&self, key: &str) -> Result<()> {
        let mut directories = self
            .directories
            .write()
            .map_err(|_| RustflyError::LockPoisoned)?;
        let mut parts = key.split('/').collect::<Vec<_>>();
        parts.pop();

        let mut current = String::new();
        for part in parts {
            if !current.is_empty() {
                current.push('/');
            }

            current.push_str(part);
            directories.insert(current.clone());
        }

        Ok(())
    }

    fn child_of(parent: &str, child: &str) -> bool {
        if parent.is_empty() {
            !child.is_empty() && !child.contains('/')
        } else {
            Self::strip_parent(parent, child).is_some_and(|rest| !rest.contains('/'))
        }
    }

    fn is_descendant(parent: &str, child: &str) -> bool {
        Self::strip_parent(parent, child).is_some()
    }

    fn strip_parent<'a>(parent: &str, child: &'a str) -> Option<&'a str> {
        child
            .strip_prefix(parent)?
            .strip_prefix('/')
            .filter(|rest| !rest.is_empty())
    }
}

#[async_trait]
impl RustflyAdapter for InMemoryAdapter {
    fn read_sync(&self, path: &str) -> Result<Bytes> {
        let key = Self::key_required(path)?;
        let files = self.files.read().map_err(|_| RustflyError::LockPoisoned)?;
        files
            .get(&key)
            .cloned()
            .ok_or_else(|| RustflyError::InvalidPath(path.to_string()))
    }

    async fn read(&self, path: &str) -> Result<Bytes> {
        self.read_sync(path)
    }

    fn write_sync(&self, path: &str, contents: Bytes) -> Result<()> {
        let key = Self::key_required(path)?;
        self.remember_parent_directories(&key)?;

        let mut files = self.files.write().map_err(|_| RustflyError::LockPoisoned)?;
        files.insert(key, contents);
        Ok(())
    }

    async fn write(&self, path: &str, contents: Bytes) -> Result<()> {
        self.write_sync(path, contents)
    }

    fn delete_sync(&self, path: &str) -> Result<()> {
        let key = Self::key_required(path)?;
        let mut files = self.files.write().map_err(|_| RustflyError::LockPoisoned)?;
        let mut directories = self
            .directories
            .write()
            .map_err(|_| RustflyError::LockPoisoned)?;

        files.remove(&key);
        files.retain(|file, _| !Self::is_descendant(&key, file));
        directories.remove(&key);
        directories.retain(|directory| !Self::is_descendant(&key, directory));
        Ok(())
    }

    async fn delete(&self, path: &str) -> Result<()> {
        self.delete_sync(path)
    }

    fn exists_sync(&self, path: &str) -> Result<bool> {
        let key = Self::key(path)?;

        if key.is_empty() {
            return Ok(true);
        }

        let files = self.files.read().map_err(|_| RustflyError::LockPoisoned)?;
        let directories = self
            .directories
            .read()
            .map_err(|_| RustflyError::LockPoisoned)?;

        Ok(files.contains_key(&key) || directories.contains(&key))
    }

    async fn exists(&self, path: &str) -> Result<bool> {
        self.exists_sync(path)
    }

    fn create_dir_sync(&self, path: &str) -> Result<()> {
        let key = Self::key(path)?;

        if key.is_empty() {
            return Ok(());
        }

        self.remember_parent_directories(&format!("{key}/placeholder"))?;
        let mut directories = self
            .directories
            .write()
            .map_err(|_| RustflyError::LockPoisoned)?;
        directories.insert(key);
        Ok(())
    }

    async fn create_dir(&self, path: &str) -> Result<()> {
        self.create_dir_sync(path)
    }

    fn list_sync(&self, path: &str) -> Result<Vec<Metadata>> {
        let parent = Self::key(path)?;
        let files = self.files.read().map_err(|_| RustflyError::LockPoisoned)?;
        let directories = self
            .directories
            .read()
            .map_err(|_| RustflyError::LockPoisoned)?;
        let mut entries = Vec::new();

        for (file, contents) in files.iter() {
            if Self::child_of(&parent, file) {
                entries.push(Metadata::new(
                    file.clone(),
                    EntryKind::File,
                    contents.len() as u64,
                    None,
                ));
            }
        }

        for directory in directories.iter() {
            if Self::child_of(&parent, directory) {
                entries.push(Metadata::new(
                    directory.clone(),
                    EntryKind::Directory,
                    0,
                    None,
                ));
            }
        }

        entries.sort_by(|left, right| left.path().cmp(right.path()));
        Ok(entries)
    }

    async fn list(&self, path: &str) -> Result<Vec<Metadata>> {
        self.list_sync(path)
    }

    fn metadata_sync(&self, path: &str) -> Result<Metadata> {
        let key = Self::key(path)?;

        if key.is_empty() {
            return Ok(Metadata::new("", EntryKind::Directory, 0, None));
        }

        let files = self.files.read().map_err(|_| RustflyError::LockPoisoned)?;
        if let Some(contents) = files.get(&key) {
            return Ok(Metadata::new(
                key,
                EntryKind::File,
                contents.len() as u64,
                None,
            ));
        }

        let directories = self
            .directories
            .read()
            .map_err(|_| RustflyError::LockPoisoned)?;
        if directories.contains(&key) {
            return Ok(Metadata::new(key, EntryKind::Directory, 0, None));
        }

        Err(RustflyError::InvalidPath(path.to_string()))
    }

    async fn metadata(&self, path: &str) -> Result<Metadata> {
        self.metadata_sync(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn supports_async_and_sync_flows() {
        let adapter = InMemoryAdapter::new();

        adapter
            .write("docs/readme.md", "hello".into())
            .await
            .unwrap();
        adapter
            .write_sync("docs/sync.md", Bytes::from_static(b"sync"))
            .unwrap();

        assert_eq!(adapter.read("docs/readme.md").await.unwrap(), "hello");
        assert_eq!(
            adapter.read_sync("docs/sync.md").unwrap(),
            Bytes::from_static(b"sync")
        );
        assert!(adapter.exists("docs").await.unwrap());
        assert_eq!(adapter.metadata("docs/readme.md").await.unwrap().len(), 5);
    }

    #[test]
    fn lists_root_and_direct_children() {
        let adapter = InMemoryAdapter::new();

        adapter
            .write_sync("a.txt", Bytes::from_static(b"a"))
            .unwrap();
        adapter
            .write_sync("docs/readme.md", Bytes::from_static(b"hello"))
            .unwrap();

        let root = adapter.list_sync("").unwrap();
        let docs = adapter.list_sync("docs").unwrap();

        assert!(root.iter().any(|entry| entry.path() == "a.txt"));
        assert!(root.iter().any(|entry| entry.path() == "docs"));
        assert!(docs.iter().any(|entry| entry.path() == "docs/readme.md"));
    }
}
