use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, OnceLock, RwLock};

use crate::adapter::contract::RustflyAdapter;
#[cfg(feature = "native")]
use crate::adapter::native::NativeAdapter;
use crate::definition::{Result, RustflyError};
use crate::operator::Filesystem;

pub type DriverFactory = dyn Fn(&StorageConfig) -> Result<Arc<dyn RustflyAdapter>> + Send + Sync;

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct StorageConfig {
    values: HashMap<String, String>,
}

impl StorageConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.values.insert(key.into(), value.into());
        self
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.values.get(key).map(String::as_str)
    }

    pub fn path(&self, key: &str) -> Option<PathBuf> {
        self.get(key).map(PathBuf::from)
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

        if registry.drivers.contains_key("local") {
            return;
        }

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
        registry.default_driver = Some("local".to_string());
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
}
