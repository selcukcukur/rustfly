use rustfly_core::UnsupportedAdapter;

/// Canonical Rustfly driver name for MongoDB GridFS storage.
pub const DRIVER: &str = "gridfs";

/// Create the current GridFS adapter placeholder.
pub const fn adapter() -> UnsupportedAdapter {
    UnsupportedAdapter::new(DRIVER)
}
