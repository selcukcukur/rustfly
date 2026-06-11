//! Fluent, thread-safe storage facade for Rustfly.
//!
//! `rustfly` exposes a Laravel-inspired `Storage` API over native filesystems
//! and feature-enabled adapter crates. The default build stays local-first with
//! the native adapter, while optional features register additional drivers.

pub mod adapter;
pub mod definition;
pub mod operator;
pub mod path;
pub mod storage;
pub mod utility;

pub use adapter::contract::{AdapterCapabilities, RustflyAdapter};
pub use adapter::native::NativeAdapter;
pub use definition::{EntryKind, Metadata, Result, RustflyError};
pub use operator::{Filesystem, RustflyOperator};
#[cfg(feature = "azure")]
pub use rustfly_azure as azure;
#[cfg(feature = "cloudflare")]
pub use rustfly_cloudflare as cloudflare;
#[cfg(feature = "drive")]
pub use rustfly_drive as drive;
#[cfg(feature = "ftp")]
pub use rustfly_ftp as ftp;
#[cfg(feature = "gridfs")]
pub use rustfly_gridfs as gridfs;
#[cfg(feature = "inmemory")]
pub use rustfly_inmemory::InMemoryAdapter;
#[cfg(feature = "s3")]
pub use rustfly_s3 as s3;
#[cfg(feature = "sftp")]
pub use rustfly_sftp as sftp;
#[cfg(feature = "webdav")]
pub use rustfly_webdav as webdav;
#[cfg(feature = "zip")]
pub use rustfly_zip as zip;
pub use storage::{Storage, StorageConfig};
