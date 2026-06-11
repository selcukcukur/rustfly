//! Google Drive storage adapter crate for Rustfly.
//!
//! The crate currently registers an unsupported placeholder so the facade can
//! expose the `drive` driver behind a feature while the real implementation lands.

use rustfly_core::UnsupportedAdapter;

/// Canonical Rustfly driver name for Google Drive style storage.
pub const DRIVER: &str = "drive";

/// Create the current Drive adapter placeholder.
pub const fn adapter() -> UnsupportedAdapter {
    UnsupportedAdapter::new(DRIVER)
}
