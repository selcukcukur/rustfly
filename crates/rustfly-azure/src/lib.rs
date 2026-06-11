use rustfly_core::UnsupportedAdapter;

/// Canonical Rustfly driver name for Azure Blob Storage.
pub const DRIVER: &str = "azure";

/// Create the current Azure adapter placeholder.
pub const fn adapter() -> UnsupportedAdapter {
    UnsupportedAdapter::new(DRIVER)
}
