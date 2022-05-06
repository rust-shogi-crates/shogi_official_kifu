use shogi_core::Color;

use crate::{Error, FromUsi, Result};

/// ```
/// # use shogi_core::Color;
/// use shogi_usi_parser::FromUsi;
/// assert_eq!(Color::from_usi_lite("b"), Some(Color::Black));
/// assert_eq!(Color::from_usi_lite("w"), Some(Color::White));
/// assert_eq!(Color::from_usi_lite("B"), None); // uppercase letters not allowed
/// ```
impl FromUsi for Color {
    fn parse_usi_slice(s: &[u8]) -> Result<(&[u8], Self)> {
        if s.is_empty() {
            return Err(Error::InvalidInput {
                from: 0,
                to: 0,
                description: "A `Color` expected, but nothing found",
            });
        }
        if s[0] == b'b' {
            return Ok((&s[1..], Color::Black));
        }
        if s[0] == b'w' {
            return Ok((&s[1..], Color::White));
        }
        Err(Error::InvalidInput {
            from: 0,
            to: 1,
            description: "A `Color` (`b` or `w`) expected, but invalid byte found",
        })
    }
}
