use shogi_core::Square;

use crate::{Error, FromUsi, Result};

/// ```
/// # use shogi_core::Square;
/// use shogi_usi_parser::FromUsi;
/// assert_eq!(Square::from_usi_lite("7g"), Square::new(7, 7));
/// assert_eq!(Square::from_usi_lite("9j"), None); // `j` is not a valid rank
/// assert_eq!(Square::from_usi_lite("0g"), None); // `0` is not a valid file
/// ```
impl FromUsi for Square {
    fn parse_usi_slice(s: &[u8]) -> Result<(&[u8], Self)> {
        if s.len() < 2 {
            return Err(Error::InvalidInput {
                from: 0,
                to: s.len(),
                description: "A `Square` must have 2 letters in its representation",
            });
        }
        // Safety: s.len() >= 2
        let (&file, s) = unsafe { s.split_first().unwrap_unchecked() };
        let (&rank, s) = unsafe { s.split_first().unwrap_unchecked() };
        if !matches!(file, b'1'..=b'9') {
            return Err(Error::InvalidInput {
                from: 0,
                to: 2,
                description: "`Square`: the first letter must be among 1, 2, ..., 9",
            });
        }
        if !matches!(rank, b'a'..=b'i') {
            return Err(Error::InvalidInput {
                from: 0,
                to: 2,
                description: "`Square`: the second letter must be among a, b, ..., i",
            });
        }
        // Safety: file and rank are both in range `1..=9`
        let result = unsafe { Square::new(file - b'0', rank - b'a' + 1).unwrap_unchecked() };
        Ok((s, result))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_usi_positive() {
        for square in Square::all() {
            let file = b'0' + square.file();
            let rank = b'a' + square.rank() - 1;
            assert_eq!(
                Square::from_usi_lite(&String::from_utf8(vec![file, rank]).unwrap()),
                Some(square),
            );
        }
    }

    #[test]
    fn from_usi_slice_exhaustive() {
        for file in 0..=255 {
            for rank in 0..=255 {
                let _ = Square::parse_usi_slice(&[file, rank]);
            }
        }
    }
}
