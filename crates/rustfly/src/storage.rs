use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, OnceLock, RwLock};

use bytes::Bytes;

use crate::adapter::contract::RustflyAdapter;
#[cfg(feature = "native")]
use crate::adapter::native::NativeAdapter;
use crate::definition::{Metadata, Result, RustflyError};
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

    pub async fn get(path: &str) -> Result<Bytes> {
        Self::read(path).await
    }

    pub fn get_sync(path: &str) -> Result<Bytes> {
        Self::read_sync(path)
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

    pub async fn metadata(path: &str) -> Result<Metadata> {
        Self::default_driver()?.metadata(path).await
    }

    pub fn metadata_sync(path: &str) -> Result<Metadata> {
        Self::default_driver()?.metadata_sync(path)
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

    #[cfg(feature = "inmemory")]
    #[tokio::test]
    async fn enabled_inmemory_feature_registers_memory_driver() {
        let storage = Storage::driver("memory").unwrap();

        storage.write("feature.txt", "enabled").await.unwrap();

        assert_eq!(storage.read("feature.txt").await.unwrap(), "enabled");
    }
}
