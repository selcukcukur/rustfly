use rustfly_core::UnsupportedAdapter;

/// Canonical Rustfly driver name for SFTP storage.
pub const DRIVER: &str = "sftp";

/// Create the current SFTP adapter placeholder.
pub const fn adapter() -> UnsupportedAdapter {
    UnsupportedAdapter::new(DRIVER)
}
