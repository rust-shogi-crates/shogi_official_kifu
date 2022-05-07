#![cfg_attr(not(test), no_std)] // Forbids using std::*.

#[cfg(feature = "alloc")]
extern crate alloc;

use shogi_core::{Color, Hand, Move, PartialPosition, Piece, Square};

mod color;
mod common;
mod error;
mod hand;
mod mv;
mod piece;
mod position;
mod square;

// Equivalent to the `?` operator. Avoids `From` impl lookup.
#[macro_export]
#[doc(hidden)]
macro_rules! bind {
    ($e:expr) => {
        match $e {
            Ok(x) => x,
            Err(e) => return Err(e),
        }
    };
}

// `?` operator with shifting indices in `InvalidInput`.
#[macro_export]
#[doc(hidden)]
macro_rules! try_with_progress {
    ($e:expr, $shift:expr) => {
        match $e {
            Ok(x) => x,
            Err(Error::InvalidInput {
                from,
                to,
                description,
            }) => {
                return Err(Error::InvalidInput {
                    from: from + $shift,
                    to: to + $shift,
                    description,
                });
            }
            Err(e) => return Err(e),
        }
    };
}

/// `FromUsi` is a [sealed](https://rust-lang.github.io/api-guidelines/future-proofing.html) trait: other crates cannot implement `FromUsi` for types.
pub trait FromUsi: private::Sealed + Sized {
    /// Primitive parsing method. This crate handles implementing this method.
    #[doc(hidden)]
    fn parse_usi_slice(s: &[u8]) -> Result<(&[u8], Self)>;

    /// Parses USI representation.
    fn from_usi(s: &str) -> Result<Self> {
        let s = s.as_bytes();
        let (remaining, value) = bind!(Self::parse_usi_slice(s));
        if remaining.is_empty() {
            return Ok(value);
        }
        Err(Error::Extra {
            from: s.len() - remaining.len(),
        })
    }

    /// Parses USI representation.
    /// If an error occurs, this function will only notify that there is some error by returning `None`.
    #[inline]
    fn from_usi_lite(s: &str) -> Option<Self> {
        match Self::from_usi(s) {
            Ok(value) => Some(value),
            Err(_) => None,
        }
    }
}

#[doc(inline)]
pub use crate::error::{Error, Result};

mod private {
    use super::*;

    pub trait Sealed {}

    impl Sealed for Color {}
    impl Sealed for Square {}
    impl Sealed for Piece {}
    impl Sealed for Move {}
    impl Sealed for [Hand; 2] {}
    impl Sealed for PartialPosition {}
    #[cfg(feature = "alloc")]
    impl Sealed for shogi_core::Position {}
}
