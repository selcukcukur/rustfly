use rustfly_core::UnsupportedAdapter;

/// Canonical Rustfly driver name for FTP storage.
pub const DRIVER: &str = "ftp";

/// Create the current FTP adapter placeholder.
pub const fn adapter() -> UnsupportedAdapter {
    UnsupportedAdapter::new(DRIVER)
}
