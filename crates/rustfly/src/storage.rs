use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, OnceLock, RwLock};
use std::time::SystemTime;

use bytes::Bytes;

use crate::adapter::contract::RustflyAdapter;
#[cfg(feature = "native")]
use crate::adapter::native::NativeAdapter;
use crate::definition::{EntryKind, Metadata, Result, RustflyError};
use crate::operator::Filesystem;
#[cfg(feature = "inmemory")]
use rustfly_inmemory::InMemoryAdapter;

pub type DriverFactory = dyn Fn(&StorageConfig) -> Result<Arc<dyn RustflyAdapter>> + Send + Sync;

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct StorageConfig {
    values: HashMap<String, String>,
}

impl StorageConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_pairs(
        pairs: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>,
    ) -> Self {
        let mut config = Self::new();

        for (key, value) in pairs {
            config.insert(key, value);
        }

        config
    }

    pub fn insert(&mut self, key: impl Into<String>, value: impl Into<String>) -> Option<String> {
        self.values.insert(key.into(), value.into())
    }

    pub fn with(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.insert(key, value);
        self
    }

    pub fn with_path(self, key: impl Into<String>, value: impl Into<PathBuf>) -> Self {
        self.with(key, value.into().to_string_lossy())
    }

    pub fn with_bool(self, key: impl Into<String>, value: bool) -> Self {
        self.with(key, value.to_string())
    }

    pub fn with_u64(self, key: impl Into<String>, value: u64) -> Self {
        self.with(key, value.to_string())
    }

    pub fn merge(mut self, other: Self) -> Self {
        self.values.extend(other.values);
        self
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.values.get(key).map(String::as_str)
    }

    pub fn contains(&self, key: &str) -> bool {
        self.values.contains_key(key)
    }

    pub fn path(&self, key: &str) -> Option<PathBuf> {
        self.get(key).map(PathBuf::from)
    }

    pub fn bool(&self, key: &str) -> Option<bool> {
        self.get(key).and_then(|value| value.parse().ok())
    }

    pub fn u64(&self, key: &str) -> Option<u64> {
        self.get(key).and_then(|value| value.parse().ok())
    }

    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.values.keys().map(String::as_str)
    }
}

#[derive(Clone)]
struct DriverDefinition {
    factory: Arc<DriverFactory>,
    config: StorageConfig,
}

#[derive(Default)]
struct Registry {
    default_driver: Option<String>,
    drivers: HashMap<String, DriverDefinition>,
}

static REGISTRY: OnceLock<RwLock<Registry>> = OnceLock::new();

fn registry() -> &'static RwLock<Registry> {
    let lock = REGISTRY.get_or_init(|| RwLock::new(Registry::default()));
    init_builtin_drivers(lock);
    lock
}

fn init_builtin_drivers(lock: &'static RwLock<Registry>) {
    #[cfg(feature = "native")]
    {
        let mut registry = lock.write().expect("rustfly registry poisoned");

        if !registry.drivers.contains_key("local") {
            let config = StorageConfig::new().with("root", ".");
            registry.drivers.insert(
                "local".to_string(),
                DriverDefinition {
                    factory: Arc::new(|config| {
                        let root = config.path("root").unwrap_or_else(|| PathBuf::from("."));
                        Ok(Arc::new(NativeAdapter::new(root)))
                    }),
                    config,
                },
            );
        }

        if registry.default_driver.is_none() {
            registry.default_driver = Some("local".to_string());
        }
    }

    #[cfg(feature = "inmemory")]
    {
        let mut registry = lock.write().expect("rustfly registry poisoned");

        if !registry.drivers.contains_key("memory") {
            registry.drivers.insert(
                "memory".to_string(),
                DriverDefinition {
                    factory: Arc::new(|_| Ok(Arc::new(InMemoryAdapter::new()))),
                    config: StorageConfig::new(),
                },
            );
        }

        if !registry.drivers.contains_key("inmemory") {
            let memory = registry
                .drivers
                .get("memory")
                .expect("memory driver must exist")
                .clone();
            registry.drivers.insert("inmemory".to_string(), memory);
        }

        if registry.default_driver.is_none() {
            registry.default_driver = Some("memory".to_string());
        }
    }

    #[cfg(any(
        feature = "s3",
        feature = "drive",
        feature = "ftp",
        feature = "azure",
        feature = "gridfs",
        feature = "webdav",
        feature = "zip",
        feature = "sftp",
        feature = "cloudflare"
    ))]
    {
        let mut registry = lock.write().expect("rustfly registry poisoned");

        #[cfg(feature = "s3")]
        register_unsupported_driver(&mut registry, rustfly_s3::DRIVER, rustfly_s3::adapter);
        #[cfg(feature = "drive")]
        register_unsupported_driver(&mut registry, rustfly_drive::DRIVER, rustfly_drive::adapter);
        #[cfg(feature = "ftp")]
        register_unsupported_driver(&mut registry, rustfly_ftp::DRIVER, rustfly_ftp::adapter);
        #[cfg(feature = "azure")]
        register_unsupported_driver(&mut registry, rustfly_azure::DRIVER, rustfly_azure::adapter);
        #[cfg(feature = "gridfs")]
        register_unsupported_driver(
            &mut registry,
            rustfly_gridfs::DRIVER,
            rustfly_gridfs::adapter,
        );
        #[cfg(feature = "webdav")]
        register_unsupported_driver(
            &mut registry,
            rustfly_webdav::DRIVER,
            rustfly_webdav::adapter,
        );
        #[cfg(feature = "zip")]
        register_unsupported_driver(&mut registry, rustfly_zip::DRIVER, rustfly_zip::adapter);
        #[cfg(feature = "sftp")]
        register_unsupported_driver(&mut registry, rustfly_sftp::DRIVER, rustfly_sftp::adapter);
        #[cfg(feature = "cloudflare")]
        register_unsupported_driver(
            &mut registry,
            rustfly_cloudflare::DRIVER,
            rustfly_cloudflare::adapter,
        );
    }
}

#[cfg(any(
    feature = "s3",
    feature = "drive",
    feature = "ftp",
    feature = "azure",
    feature = "gridfs",
    feature = "webdav",
    feature = "zip",
    feature = "sftp",
    feature = "cloudflare"
))]
fn register_unsupported_driver<A>(registry: &mut Registry, name: &'static str, adapter: fn() -> A)
where
    A: RustflyAdapter + 'static,
{
    if registry.drivers.contains_key(name) {
        return;
    }

    registry.drivers.insert(
        name.to_string(),
        DriverDefinition {
            factory: Arc::new(move |_| Ok(Arc::new(adapter()))),
            config: StorageConfig::new(),
        },
    );
}

pub struct Storage;

impl Storage {
    pub fn extend(
        name: impl Into<String>,
        factory: impl Fn(&StorageConfig) -> Result<Arc<dyn RustflyAdapter>> + Send + Sync + 'static,
    ) -> Result<()> {
        Self::extend_with_config(name, StorageConfig::new(), factory)
    }

    pub fn extend_with_config(
        name: impl Into<String>,
        config: StorageConfig,
        factory: impl Fn(&StorageConfig) -> Result<Arc<dyn RustflyAdapter>> + Send + Sync + 'static,
    ) -> Result<()> {
        let name = normalize_driver_name(name.into())?;
        let mut registry = registry()
            .write()
            .map_err(|_| RustflyError::RegistryPoisoned)?;

        if registry.drivers.contains_key(&name) {
            return Err(RustflyError::DriverAlreadyRegistered(name));
        }

        registry.drivers.insert(
            name.clone(),
            DriverDefinition {
                factory: Arc::new(factory),
                config,
            },
        );

        if registry.default_driver.is_none() {
            registry.default_driver = Some(name);
        }

        Ok(())
    }

    pub fn extend_or_replace(
        name: impl Into<String>,
        factory: impl Fn(&StorageConfig) -> Result<Arc<dyn RustflyAdapter>> + Send + Sync + 'static,
    ) -> Result<()> {
        Self::extend_or_replace_with_config(name, StorageConfig::new(), factory)
    }

    pub fn extend_or_replace_with_config(
        name: impl Into<String>,
        config: StorageConfig,
        factory: impl Fn(&StorageConfig) -> Result<Arc<dyn RustflyAdapter>> + Send + Sync + 'static,
    ) -> Result<()> {
        let name = normalize_driver_name(name.into())?;
        let mut registry = registry()
            .write()
            .map_err(|_| RustflyError::RegistryPoisoned)?;

        registry.drivers.insert(
            name.clone(),
            DriverDefinition {
                factory: Arc::new(factory),
                config,
            },
        );

        if registry.default_driver.is_none() {
            registry.default_driver = Some(name);
        }

        Ok(())
    }

    pub fn has_driver(name: impl AsRef<str>) -> Result<bool> {
        let name = normalize_driver_name(name.as_ref().to_string())?;
        let registry = registry()
            .read()
            .map_err(|_| RustflyError::RegistryPoisoned)?;

        Ok(registry.drivers.contains_key(&name))
    }

    pub fn driver_names() -> Result<Vec<String>> {
        let registry = registry()
            .read()
            .map_err(|_| RustflyError::RegistryPoisoned)?;
        let mut names = registry.drivers.keys().cloned().collect::<Vec<_>>();
        names.sort();
        Ok(names)
    }

    pub fn configure(name: impl Into<String>, config: StorageConfig) -> Result<()> {
        let name = normalize_driver_name(name.into())?;
        let mut registry = registry()
            .write()
            .map_err(|_| RustflyError::RegistryPoisoned)?;
        let driver = registry
            .drivers
            .get_mut(&name)
            .ok_or_else(|| RustflyError::DriverNotFound(name.clone()))?;

        driver.config = config;
        Ok(())
    }

    pub fn set_default_driver(name: impl Into<String>) -> Result<()> {
        let name = normalize_driver_name(name.into())?;
        let mut registry = registry()
            .write()
            .map_err(|_| RustflyError::RegistryPoisoned)?;

        if !registry.drivers.contains_key(&name) {
            return Err(RustflyError::DriverNotFound(name));
        }

        registry.default_driver = Some(name);
        Ok(())
    }

    pub fn set_default_disk(name: impl Into<String>) -> Result<()> {
        Self::set_default_driver(name)
    }

    pub fn default_driver() -> Result<Filesystem> {
        let name = {
            let registry = registry()
                .read()
                .map_err(|_| RustflyError::RegistryPoisoned)?;
            registry
                .default_driver
                .clone()
                .ok_or(RustflyError::DefaultDriverMissing)?
        };

        Self::driver(name)
    }

    pub fn default_disk() -> Result<Filesystem> {
        Self::default_driver()
    }

    pub fn driver(name: impl AsRef<str>) -> Result<Filesystem> {
        let requested = name.as_ref().trim();

        if requested.is_empty() {
            return Self::default_driver();
        }

        let definition = {
            let registry = registry()
                .read()
                .map_err(|_| RustflyError::RegistryPoisoned)?;
            registry
                .drivers
                .get(requested)
                .cloned()
                .ok_or_else(|| RustflyError::DriverNotFound(requested.to_string()))?
        };

        let adapter = (definition.factory)(&definition.config)?;
        Ok(Filesystem::new(requested.to_string(), adapter))
    }

    pub fn disk(name: impl AsRef<str>) -> Result<Filesystem> {
        Self::driver(name)
    }

    pub async fn read(path: &str) -> Result<Bytes> {
        Self::default_driver()?.read(path).await
    }

    pub fn read_sync(path: &str) -> Result<Bytes> {
        Self::default_driver()?.read_sync(path)
    }

    pub async fn read_string(path: &str) -> Result<String> {
        Self::default_driver()?.read_string(path).await
    }

    pub fn read_string_sync(path: &str) -> Result<String> {
        Self::default_driver()?.read_string_sync(path)
    }

    pub async fn get(path: &str) -> Result<Bytes> {
        Self::read(path).await
    }

    pub fn get_sync(path: &str) -> Result<Bytes> {
        Self::read_sync(path)
    }

    pub async fn get_string(path: &str) -> Result<String> {
        Self::read_string(path).await
    }

    pub fn get_string_sync(path: &str) -> Result<String> {
        Self::read_string_sync(path)
    }

    pub async fn write(path: &str, contents: impl Into<Bytes> + Send) -> Result<()> {
        Self::default_driver()?.write(path, contents).await
    }

    pub fn write_sync(path: &str, contents: impl Into<Bytes>) -> Result<()> {
        Self::default_driver()?.write_sync(path, contents)
    }

    pub async fn put(path: &str, contents: impl Into<Bytes> + Send) -> Result<()> {
        Self::write(path, contents).await
    }

    pub fn put_sync(path: &str, contents: impl Into<Bytes>) -> Result<()> {
        Self::write_sync(path, contents)
    }

    pub async fn append(path: &str, contents: impl Into<Bytes> + Send) -> Result<()> {
        Self::default_driver()?.append(path, contents).await
    }

    pub fn append_sync(path: &str, contents: impl Into<Bytes>) -> Result<()> {
        Self::default_driver()?.append_sync(path, contents)
    }

    pub async fn prepend(path: &str, contents: impl Into<Bytes> + Send) -> Result<()> {
        Self::default_driver()?.prepend(path, contents).await
    }

    pub fn prepend_sync(path: &str, contents: impl Into<Bytes>) -> Result<()> {
        Self::default_driver()?.prepend_sync(path, contents)
    }

    pub async fn delete(path: &str) -> Result<()> {
        Self::default_driver()?.delete(path).await
    }

    pub fn delete_sync(path: &str) -> Result<()> {
        Self::default_driver()?.delete_sync(path)
    }

    pub async fn exists(path: &str) -> Result<bool> {
        Self::default_driver()?.exists(path).await
    }

    pub fn exists_sync(path: &str) -> Result<bool> {
        Self::default_driver()?.exists_sync(path)
    }

    pub async fn missing(path: &str) -> Result<bool> {
        Self::default_driver()?.missing(path).await
    }

    pub fn missing_sync(path: &str) -> Result<bool> {
        Self::default_driver()?.missing_sync(path)
    }

    pub async fn create_dir(path: &str) -> Result<()> {
        Self::default_driver()?.create_dir(path).await
    }

    pub fn create_dir_sync(path: &str) -> Result<()> {
        Self::default_driver()?.create_dir_sync(path)
    }

    pub async fn list(path: &str) -> Result<Vec<Metadata>> {
        Self::default_driver()?.list(path).await
    }

    pub fn list_sync(path: &str) -> Result<Vec<Metadata>> {
        Self::default_driver()?.list_sync(path)
    }

    pub async fn list_recursive(path: &str) -> Result<Vec<Metadata>> {
        Self::default_driver()?.list_recursive(path).await
    }

    pub fn list_recursive_sync(path: &str) -> Result<Vec<Metadata>> {
        Self::default_driver()?.list_recursive_sync(path)
    }

    pub async fn files(path: &str) -> Result<Vec<Metadata>> {
        filter_by_kind(Self::list(path).await?, EntryKind::File)
    }

    pub fn files_sync(path: &str) -> Result<Vec<Metadata>> {
        filter_by_kind(Self::list_sync(path)?, EntryKind::File)
    }

    pub async fn directories(path: &str) -> Result<Vec<Metadata>> {
        filter_by_kind(Self::list(path).await?, EntryKind::Directory)
    }

    pub fn directories_sync(path: &str) -> Result<Vec<Metadata>> {
        filter_by_kind(Self::list_sync(path)?, EntryKind::Directory)
    }

    pub async fn all_files(path: &str) -> Result<Vec<Metadata>> {
        filter_by_kind(Self::list_recursive(path).await?, EntryKind::File)
    }

    pub fn all_files_sync(path: &str) -> Result<Vec<Metadata>> {
        filter_by_kind(Self::list_recursive_sync(path)?, EntryKind::File)
    }

    pub async fn all_directories(path: &str) -> Result<Vec<Metadata>> {
        filter_by_kind(Self::list_recursive(path).await?, EntryKind::Directory)
    }

    pub fn all_directories_sync(path: &str) -> Result<Vec<Metadata>> {
        filter_by_kind(Self::list_recursive_sync(path)?, EntryKind::Directory)
    }

    pub async fn metadata(path: &str) -> Result<Metadata> {
        Self::default_driver()?.metadata(path).await
    }

    pub fn metadata_sync(path: &str) -> Result<Metadata> {
        Self::default_driver()?.metadata_sync(path)
    }

    pub async fn size(path: &str) -> Result<u64> {
        Self::default_driver()?.size(path).await
    }

    pub fn size_sync(path: &str) -> Result<u64> {
        Self::default_driver()?.size_sync(path)
    }

    pub async fn last_modified(path: &str) -> Result<Option<SystemTime>> {
        Self::default_driver()?.last_modified(path).await
    }

    pub fn last_modified_sync(path: &str) -> Result<Option<SystemTime>> {
        Self::default_driver()?.last_modified_sync(path)
    }

    pub async fn copy(from: &str, to: &str) -> Result<()> {
        Self::default_driver()?.copy(from, to).await
    }

    pub fn copy_sync(from: &str, to: &str) -> Result<()> {
        Self::default_driver()?.copy_sync(from, to)
    }

    pub async fn move_file(from: &str, to: &str) -> Result<()> {
        Self::default_driver()?.move_file(from, to).await
    }

    pub fn move_file_sync(from: &str, to: &str) -> Result<()> {
        Self::default_driver()?.move_file_sync(from, to)
    }
}

fn normalize_driver_name(name: String) -> Result<String> {
    let name = name.trim().to_string();

    if name.is_empty() {
        return Err(RustflyError::InvalidDriverName);
    }

    Ok(name)
}

fn filter_by_kind(entries: Vec<Metadata>, kind: EntryKind) -> Result<Vec<Metadata>> {
    Ok(entries
        .into_iter()
        .filter(|entry| entry.kind() == kind)
        .collect())
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::*;
    use async_trait::async_trait;
    use bytes::Bytes;

    #[derive(Default)]
    struct MemoryAdapter {
        writes: AtomicUsize,
    }

    #[async_trait]
    impl RustflyAdapter for MemoryAdapter {
        fn read_sync(&self, _path: &str) -> Result<Bytes> {
            Ok(Bytes::from_static(b"memory-sync"))
        }

        async fn read(&self, _path: &str) -> Result<Bytes> {
            Ok(Bytes::from_static(b"memory"))
        }

        fn write_sync(&self, _path: &str, _contents: Bytes) -> Result<()> {
            self.writes.fetch_add(1, Ordering::Relaxed);
            Ok(())
        }

        async fn write(&self, _path: &str, _contents: Bytes) -> Result<()> {
            self.writes.fetch_add(1, Ordering::Relaxed);
            Ok(())
        }

        fn delete_sync(&self, _path: &str) -> Result<()> {
            Ok(())
        }

        async fn delete(&self, _path: &str) -> Result<()> {
            Ok(())
        }

        fn exists_sync(&self, _path: &str) -> Result<bool> {
            Ok(true)
        }

        async fn exists(&self, _path: &str) -> Result<bool> {
            Ok(true)
        }
    }

    #[tokio::test]
    async fn empty_driver_name_uses_default_driver() {
        let storage = Storage::driver("").unwrap();
        assert_eq!(storage.driver_name(), "local");
    }

    #[tokio::test]
    async fn disk_alias_resolves_named_driver() {
        let storage = Storage::disk("local").unwrap();
        assert_eq!(storage.driver_name(), "local");
    }

    #[test]
    fn storage_config_supports_typed_builders_and_getters() {
        let config = StorageConfig::from_pairs([("region", "eu")])
            .with_path("root", PathBuf::from("storage"))
            .with_bool("visibility", true)
            .with_u64("max_retries", 3)
            .merge(StorageConfig::new().with("bucket", "assets"));

        assert_eq!(config.get("region"), Some("eu"));
        assert_eq!(config.path("root"), Some(PathBuf::from("storage")));
        assert_eq!(config.bool("visibility"), Some(true));
        assert_eq!(config.u64("max_retries"), Some(3));
        assert!(config.contains("bucket"));
        assert!(config.keys().any(|key| key == "bucket"));
    }

    #[tokio::test]
    async fn custom_drivers_can_be_registered_and_resolved() {
        let name = format!("memory-{}", std::process::id());

        Storage::extend(&name, |_| Ok(Arc::new(MemoryAdapter::default()))).unwrap();

        let storage = Storage::driver(&name).unwrap();
        assert_eq!(
            storage.read("anything").await.unwrap(),
            Bytes::from_static(b"memory")
        );
        assert_eq!(
            storage.read_sync("anything").unwrap(),
            Bytes::from_static(b"memory-sync")
        );
    }

    #[tokio::test]
    async fn drivers_can_be_inspected_and_replaced() {
        let name = format!("replaceable-{}", std::process::id());

        Storage::extend(&name, |_| Ok(Arc::new(MemoryAdapter::default()))).unwrap();
        assert!(Storage::has_driver(&name).unwrap());
        assert!(
            Storage::driver_names()
                .unwrap()
                .iter()
                .any(|driver| driver == &name)
        );

        Storage::extend_or_replace(&name, |_| Ok(Arc::new(MemoryAdapter::default()))).unwrap();

        let storage = Storage::driver(&name).unwrap();
        storage.write("file.txt", "data").await.unwrap();
        assert!(storage.exists("file.txt").await.unwrap());
    }

    #[tokio::test]
    async fn default_driver_shortcuts_support_async_and_sync_operations() {
        let dir = tempfile::tempdir().unwrap();
        Storage::configure(
            "local",
            StorageConfig::new().with("root", dir.path().to_string_lossy()),
        )
        .unwrap();

        Storage::put("facade/async.txt", "async").await.unwrap();
        Storage::put_sync("facade/sync.txt", Bytes::from_static(b"sync")).unwrap();
        Storage::copy("facade/async.txt", "facade/copy.txt")
            .await
            .unwrap();
        Storage::move_file_sync("facade/sync.txt", "facade/moved.txt").unwrap();

        assert_eq!(Storage::get("facade/async.txt").await.unwrap(), "async");
        assert_eq!(Storage::get_sync("facade/moved.txt").unwrap(), "sync");
        assert!(Storage::exists("facade/copy.txt").await.unwrap());
        assert_eq!(Storage::metadata_sync("facade/moved.txt").unwrap().len(), 4);
    }

    #[tokio::test]
    async fn driver_shortcuts_filter_files_and_directories() {
        let dir = tempfile::tempdir().unwrap();
        let name = format!("listing-{}", std::process::id());
        Storage::extend_or_replace_with_config(
            &name,
            StorageConfig::new().with_path("root", dir.path()),
            |config| {
                let root = config.path("root").unwrap();
                Ok(Arc::new(NativeAdapter::new(root)))
            },
        )
        .unwrap();

        let storage = Storage::disk(&name).unwrap();
        storage.put("listing/file.txt", "file").await.unwrap();
        storage.create_dir("listing/empty").await.unwrap();

        let files = storage.files("listing").await.unwrap();
        let directories = storage.directories_sync("listing").unwrap();

        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path(), "listing/file.txt");
        assert!(files[0].is_file());
        assert_eq!(directories.len(), 1);
        assert_eq!(directories[0].path(), "listing/empty");
        assert!(directories[0].is_directory());
    }

    #[tokio::test]
    async fn driver_shortcuts_support_recursive_listing() {
        let dir = tempfile::tempdir().unwrap();
        let name = format!("recursive-{}", std::process::id());
        Storage::extend_or_replace_with_config(
            &name,
            StorageConfig::new().with_path("root", dir.path()),
            |config| {
                let root = config.path("root").unwrap();
                Ok(Arc::new(NativeAdapter::new(root)))
            },
        )
        .unwrap();

        let storage = Storage::disk(&name).unwrap();
        storage.put("tree/root.txt", "root").await.unwrap();
        storage.put("tree/nested/file.txt", "nested").await.unwrap();
        storage.create_dir("tree/empty").await.unwrap();

        let files = storage.all_files("tree").await.unwrap();
        let directories = storage.all_directories_sync("tree").unwrap();

        assert!(files.iter().any(|entry| entry.path() == "tree/root.txt"));
        assert!(
            files
                .iter()
                .any(|entry| entry.path() == "tree/nested/file.txt")
        );
        assert!(
            directories
                .iter()
                .any(|entry| entry.path() == "tree/nested")
        );
        assert!(directories.iter().any(|entry| entry.path() == "tree/empty"));
    }

    #[tokio::test]
    async fn driver_shortcuts_read_size_and_last_modified() {
        let dir = tempfile::tempdir().unwrap();
        let name = format!("metadata-{}", std::process::id());
        Storage::extend_or_replace_with_config(
            &name,
            StorageConfig::new().with_path("root", dir.path()),
            |config| {
                let root = config.path("root").unwrap();
                Ok(Arc::new(NativeAdapter::new(root)))
            },
        )
        .unwrap();

        let storage = Storage::disk(&name).unwrap();
        storage.put("meta/file.txt", "hello").await.unwrap();

        assert_eq!(storage.size("meta/file.txt").await.unwrap(), 5);
        assert_eq!(storage.size_sync("meta/file.txt").unwrap(), 5);
        assert!(
            storage
                .last_modified("meta/file.txt")
                .await
                .unwrap()
                .is_some()
        );
        assert!(
            storage
                .last_modified_sync("meta/file.txt")
                .unwrap()
                .is_some()
        );
    }

    #[tokio::test]
    async fn driver_shortcuts_read_utf8_strings() {
        let dir = tempfile::tempdir().unwrap();
        let name = format!("strings-{}", std::process::id());
        Storage::extend_or_replace_with_config(
            &name,
            StorageConfig::new().with_path("root", dir.path()),
            |config| {
                let root = config.path("root").unwrap();
                Ok(Arc::new(NativeAdapter::new(root)))
            },
        )
        .unwrap();

        let storage = Storage::disk(&name).unwrap();
        storage.put("text/file.txt", "hello").await.unwrap();

        assert_eq!(storage.get_string("text/file.txt").await.unwrap(), "hello");
        assert_eq!(storage.get_string_sync("text/file.txt").unwrap(), "hello");
    }

    #[tokio::test]
    async fn driver_shortcuts_append_and_prepend_contents() {
        let dir = tempfile::tempdir().unwrap();
        let name = format!("append-{}", std::process::id());
        Storage::extend_or_replace_with_config(
            &name,
            StorageConfig::new().with_path("root", dir.path()),
            |config| {
                let root = config.path("root").unwrap();
                Ok(Arc::new(NativeAdapter::new(root)))
            },
        )
        .unwrap();

        let storage = Storage::disk(&name).unwrap();
        storage.append("log.txt", "world").await.unwrap();
        storage
            .prepend_sync("log.txt", Bytes::from_static(b"hello "))
            .unwrap();
        storage
            .append_sync("log.txt", Bytes::from_static(b"!"))
            .unwrap();

        assert_eq!(storage.get_string("log.txt").await.unwrap(), "hello world!");
    }

    #[tokio::test]
    async fn driver_shortcuts_check_missing_paths() {
        let dir = tempfile::tempdir().unwrap();
        let name = format!("missing-{}", std::process::id());
        Storage::extend_or_replace_with_config(
            &name,
            StorageConfig::new().with_path("root", dir.path()),
            |config| {
                let root = config.path("root").unwrap();
                Ok(Arc::new(NativeAdapter::new(root)))
            },
        )
        .unwrap();

        let storage = Storage::disk(&name).unwrap();
        assert!(storage.missing("missing.txt").await.unwrap());
        storage.put("missing.txt", "present").await.unwrap();
        assert!(!storage.missing_sync("missing.txt").unwrap());
    }

    #[cfg(feature = "inmemory")]
    #[tokio::test]
    async fn enabled_inmemory_feature_registers_memory_driver() {
        let storage = Storage::driver("memory").unwrap();

        storage.write("feature.txt", "enabled").await.unwrap();

        assert_eq!(storage.read("feature.txt").await.unwrap(), "enabled");
    }

    #[cfg(all(
        feature = "s3",
        feature = "drive",
        feature = "ftp",
        feature = "azure",
        feature = "gridfs",
        feature = "webdav",
        feature = "zip",
        feature = "sftp",
        feature = "cloudflare"
    ))]
    #[tokio::test]
    async fn enabled_adapter_features_register_placeholder_drivers() {
        let expected = [
            "azure",
            "cloudflare",
            "drive",
            "ftp",
            "gridfs",
            "s3",
            "sftp",
            "webdav",
            "zip",
        ];
        let names = Storage::driver_names().unwrap();

        for driver in expected {
            assert!(names.iter().any(|name| name == driver), "{driver} missing");
        }

        let error = Storage::driver("s3")
            .unwrap()
            .read("object.txt")
            .await
            .unwrap_err();

        assert!(matches!(error, RustflyError::Unsupported("read")));
    }
}
