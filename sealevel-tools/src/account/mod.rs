//! Account serialization and deserialization utilities.

#[cfg(feature = "borsh")]
mod borsh;
mod write;

#[cfg(feature = "borsh")]
pub use borsh::*;
pub use write::*;
