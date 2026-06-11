use rustfly_core::UnsupportedAdapter;

/// Canonical Rustfly driver name for ZIP archive-backed storage.
pub const DRIVER: &str = "zip";

/// Create the current ZIP adapter placeholder.
pub const fn adapter() -> UnsupportedAdapter {
    UnsupportedAdapter::new(DRIVER)
}
