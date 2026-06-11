//! Shared Rustfly adapter contracts and storage primitives.
//!
//! Driver crates depend on `rustfly-core` to implement the same object-safe
//! sync and async adapter API without depending on the facade crate.

pub mod adapter;
pub mod definition;
pub mod path;
pub mod unsupported;

pub use adapter::RustflyAdapter;
pub use definition::{EntryKind, Metadata, Result, RustflyError};
pub use path::RustflyPath;
pub use unsupported::UnsupportedAdapter;
