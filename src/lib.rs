//!  This crate contains feature `serde`, which enables serialization/deserialization
//!  support.

#[cfg(feature = "serde")]
mod serde;
mod strings;
mod strings_no_index;

pub use strings::*;
pub use strings_no_index::*;
