//! FTP storage adapter crate for Rustfly.
//!
//! The crate currently registers an unsupported placeholder so the facade can
//! expose the `ftp` driver behind a feature while the real implementation lands.

use rustfly_core::UnsupportedAdapter;

/// Canonical Rustfly driver name for FTP storage.
pub const DRIVER: &str = "ftp";

/// Create the current FTP adapter placeholder.
pub const fn adapter() -> UnsupportedAdapter {
    UnsupportedAdapter::new(DRIVER)
}
