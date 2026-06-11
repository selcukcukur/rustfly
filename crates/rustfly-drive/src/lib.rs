use rustfly_core::UnsupportedAdapter;

/// Canonical Rustfly driver name for Google Drive style storage.
pub const DRIVER: &str = "drive";

/// Create the current Drive adapter placeholder.
pub const fn adapter() -> UnsupportedAdapter {
    UnsupportedAdapter::new(DRIVER)
}
