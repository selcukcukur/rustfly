use rustfly_core::UnsupportedAdapter;

/// Canonical Rustfly driver name for WebDAV storage.
pub const DRIVER: &str = "webdav";

/// Create the current WebDAV adapter placeholder.
pub const fn adapter() -> UnsupportedAdapter {
    UnsupportedAdapter::new(DRIVER)
}
