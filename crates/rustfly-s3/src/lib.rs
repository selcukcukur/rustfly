use rustfly_core::UnsupportedAdapter;

/// Canonical Rustfly driver name for Amazon S3 compatible object storage.
pub const DRIVER: &str = "s3";

/// Create the current S3 adapter placeholder.
pub const fn adapter() -> UnsupportedAdapter {
    UnsupportedAdapter::new(DRIVER)
}
