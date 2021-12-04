//!  This crate contains feature `serde`, which enables serialization/deserialization
//!  support.

#[cfg(feature = "serde")]
mod serde;
mod strings;

pub use strings::*;
