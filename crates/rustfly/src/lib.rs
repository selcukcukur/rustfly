pub mod adapter;
pub mod definition;
pub mod operator;
pub mod path;
pub mod storage;
pub mod utility;

pub use adapter::contract::RustflyAdapter;
pub use adapter::native::NativeAdapter;
pub use definition::{EntryKind, Metadata, Result, RustflyError};
pub use operator::{Filesystem, RustflyOperator};
#[cfg(feature = "inmemory")]
pub use rustfly_inmemory::InMemoryAdapter;
pub use storage::{Storage, StorageConfig};
