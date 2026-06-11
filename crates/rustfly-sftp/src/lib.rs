//! SFTP storage adapter crate for Rustfly.
//!
//! The crate currently registers an unsupported placeholder so the facade can
//! expose the `sftp` driver behind a feature while the real implementation lands.

use rustfly_core::UnsupportedAdapter;

/// Canonical Rustfly driver name for SFTP storage.
pub const DRIVER: &str = "sftp";

/// Create the current SFTP adapter placeholder.
pub const fn adapter() -> UnsupportedAdapter {
    UnsupportedAdapter::new(DRIVER)
}
