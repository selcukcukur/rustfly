//! WebDAV storage adapter crate for Rustfly.
//!
//! The crate currently registers an unsupported placeholder so the facade can
//! expose the `webdav` driver behind a feature while the real implementation lands.

use rustfly_core::UnsupportedAdapter;

/// Canonical Rustfly driver name for WebDAV storage.
pub const DRIVER: &str = "webdav";

/// Create the current WebDAV adapter placeholder.
pub const fn adapter() -> UnsupportedAdapter {
    UnsupportedAdapter::new(DRIVER)
}
