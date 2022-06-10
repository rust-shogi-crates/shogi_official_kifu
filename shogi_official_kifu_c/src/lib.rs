#![cfg_attr(not(feature = "std"), feature(alloc_error_handler))]
#![cfg_attr(not(feature = "std"), no_std)] // Forbids using std::*.

extern crate alloc;

#[cfg(not(feature = "std"))]
mod no_std;

#[doc(hidden)]
pub use shogi_official_kifu::*;
