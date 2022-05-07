use shogi_core::{Color, Move, Piece, Square};

use crate::{bind, try_with_progress, Error, FromUsi, Result};

/// Drop moves are assumed to be black's move.
/// In order to figure out whose move it is,
/// one must check which side is to play at the starting position and count how many moves are played.
///
/// Examples:
/// ```
/// # use shogi_core::{Color, Move, Piece, PieceKind, Square};
/// use shogi_usi_parser::FromUsi;
/// assert_eq!(Move::from_usi_lite("7g7f").unwrap(), Move::Normal { from: Square::new(7, 7).unwrap(), to: Square::new(7, 6).unwrap(), promote: false });
/// assert_eq!(Move::from_usi_lite("8h2b+").unwrap(), Move::Normal { from: Square::new(8, 8).unwrap(), to: Square::new(2, 2).unwrap(), promote: true });
/// assert_eq!(Move::from_usi_lite("P*3d").unwrap(), Move::Drop { piece: Piece::new(PieceKind::Pawn, Color::Black), to: Square::new(3, 4).unwrap() });
/// assert_eq!(Move::from_usi_lite("p*3d"), None); // piece must be an uppercase letter
/// ```
impl FromUsi for Move {
    fn parse_usi_slice(s: &[u8]) -> Result<(&[u8], Self)> {
        if s.len() < 4 {
            return Err(Error::InvalidInput {
                from: 0,
                to: s.len(),
                description: "A `Move` expected, but less than 4 bytes found",
            });
        }
        if s.get(1).copied() == Some(b'*') {
            // A drop move: s[0] is an uppercase letter designating the `PieceKind`
            let (remaining, piece) = bind!(Piece::parse_usi_slice(&s[..1]));
            debug_assert!(remaining.is_empty());
            if piece.color() == Color::White {
                return Err(Error::InvalidInput {
                    from: 0,
                    to: 1,
                    description: "piece must be an uppercase letter",
                });
            }
            let (s, square) = try_with_progress!(Square::parse_usi_slice(&s[2..]), 2);
            return Ok((s, Move::Drop { piece, to: square }));
        }
        // A normal move: `s[..2]` is `from` and `s[2..4]` is `to`
        let (s, from) = bind!(Square::parse_usi_slice(s));
        let (mut s, to) = try_with_progress!(Square::parse_usi_slice(s), 2);
        // promote?
        let mut promote = false;
        if let Some((&b'+', rest)) = s.split_first() {
            promote = true;
            s = rest;
        }
        Ok((s, Move::Normal { from, to, promote }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn negative() {
        let result = Move::from_usi("P*9j"); // wrong coordinates
        assert!(matches!(
            result,
            Err(Error::InvalidInput { from: 2, to: 4, .. }),
        ));

        let result = Move::from_usi("P+3d"); // wrong drop letter
        assert!(matches!(
            result,
            Err(Error::InvalidInput { from: 0, to: 2, .. }),
        ));
    }
}
