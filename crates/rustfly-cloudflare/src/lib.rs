use rustfly_core::UnsupportedAdapter;

/// Canonical Rustfly driver name for Cloudflare storage services.
pub const DRIVER: &str = "cloudflare";

/// Create the current Cloudflare adapter placeholder.
pub const fn adapter() -> UnsupportedAdapter {
    UnsupportedAdapter::new(DRIVER)
}
