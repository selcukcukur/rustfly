//! ZIP archive storage adapter crate for Rustfly.
//!
//! The crate currently registers an unsupported placeholder so the facade can
//! expose the `zip` driver behind a feature while the real implementation lands.

use rustfly_core::UnsupportedAdapter;

/// Canonical Rustfly driver name for ZIP archive-backed storage.
pub const DRIVER: &str = "zip";

/// Create the current ZIP adapter placeholder.
pub const fn adapter() -> UnsupportedAdapter {
    UnsupportedAdapter::new(DRIVER)
}
