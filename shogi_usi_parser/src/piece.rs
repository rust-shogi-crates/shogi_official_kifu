use shogi_core::{Color, Piece, PieceKind};

use crate::{Error, FromUsi, Result};

const BLACK_PIECES: &[u8] = b"PLNSGBRK";
const WHITE_PIECES: &[u8] = b"plnsgbrk";

/// ```
/// # use shogi_core::{Color, Piece, PieceKind};
/// use shogi_usi_parser::FromUsi;
/// assert_eq!(Piece::from_usi_lite("+B"), Some(Piece::new(PieceKind::ProBishop, Color::Black)));
/// assert_eq!(Piece::from_usi_lite("l"), Some(Piece::new(PieceKind::Lance, Color::White)));
/// assert_eq!(Piece::from_usi_lite("Q"), None); // shogi doesn't have queens
/// assert_eq!(Piece::from_usi_lite("+K"), None); // king cannot promote
/// assert_eq!(Piece::from_usi_lite("+"), None); // promote what?
/// assert_eq!(Piece::from_usi_lite(""), None); // at least give something, will you?
/// ```
impl FromUsi for Piece {
    fn parse_usi_slice(s: &[u8]) -> Result<(&[u8], Self)> {
        if s.is_empty() {
            return Err(Error::InvalidInput {
                from: 0,
                to: 0,
                description: "A `Piece` expected, but nothing found",
            });
        }
        // Safety: s.len() >= 1
        let (&first, rest) = unsafe { s.split_first().unwrap_unchecked() };
        if first == b'+' {
            // a promoted piece
            let (&piece_byte, rest) = if let Some(x) = rest.split_first() {
                x
            } else {
                return Err(Error::InvalidInput {
                    from: 0,
                    to: 1,
                    description: "A promoted `Piece` expected, but nothing found",
                });
            };
            return if let Some(piece) = byte_to_piece(piece_byte) {
                if let Some(piece) = piece.promote() {
                    Ok((rest, piece))
                } else {
                    Err(Error::InvalidInput {
                        from: 0,
                        to: 2,
                        description: "Cannot promote",
                    })
                }
            } else {
                Err(Error::InvalidInput {
                    from: 0,
                    to: 2,
                    description: "Unrecognized piece type (promoted)",
                })
            };
        }
        if let Some(piece) = byte_to_piece(first) {
            Ok((rest, piece))
        } else {
            Err(Error::InvalidInput {
                from: 0,
                to: 1,
                description: "Unrecognized piece type",
            })
        }
    }
}

fn byte_to_piece(c: u8) -> Option<Piece> {
    // TODO: optimize
    for (index, piece_char) in BLACK_PIECES.iter().copied().enumerate() {
        if c == piece_char {
            let piece_kind = PieceKind::from_u8(index as u8 + 1)?;
            return Some(Piece::new(piece_kind, Color::Black));
        }
    }
    for (index, piece_char) in WHITE_PIECES.iter().copied().enumerate() {
        if c == piece_char {
            let piece_kind = PieceKind::from_u8(index as u8 + 1)?;
            return Some(Piece::new(piece_kind, Color::White));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn positive() {
        let tests: [(&[u8], _); 28] = [
            // black
            (b"P", Piece::new(PieceKind::Pawn, Color::Black)),
            (b"L", Piece::new(PieceKind::Lance, Color::Black)),
            (b"N", Piece::new(PieceKind::Knight, Color::Black)),
            (b"S", Piece::new(PieceKind::Silver, Color::Black)),
            (b"G", Piece::new(PieceKind::Gold, Color::Black)),
            (b"B", Piece::new(PieceKind::Bishop, Color::Black)),
            (b"R", Piece::new(PieceKind::Rook, Color::Black)),
            (b"K", Piece::new(PieceKind::King, Color::Black)),
            (b"+P", Piece::new(PieceKind::ProPawn, Color::Black)),
            (b"+L", Piece::new(PieceKind::ProLance, Color::Black)),
            (b"+N", Piece::new(PieceKind::ProKnight, Color::Black)),
            (b"+S", Piece::new(PieceKind::ProSilver, Color::Black)),
            (b"+B", Piece::new(PieceKind::ProBishop, Color::Black)),
            (b"+R", Piece::new(PieceKind::ProRook, Color::Black)),
            // white
            (b"p", Piece::new(PieceKind::Pawn, Color::White)),
            (b"l", Piece::new(PieceKind::Lance, Color::White)),
            (b"n", Piece::new(PieceKind::Knight, Color::White)),
            (b"s", Piece::new(PieceKind::Silver, Color::White)),
            (b"g", Piece::new(PieceKind::Gold, Color::White)),
            (b"b", Piece::new(PieceKind::Bishop, Color::White)),
            (b"r", Piece::new(PieceKind::Rook, Color::White)),
            (b"k", Piece::new(PieceKind::King, Color::White)),
            (b"+p", Piece::new(PieceKind::ProPawn, Color::White)),
            (b"+l", Piece::new(PieceKind::ProLance, Color::White)),
            (b"+n", Piece::new(PieceKind::ProKnight, Color::White)),
            (b"+s", Piece::new(PieceKind::ProSilver, Color::White)),
            (b"+b", Piece::new(PieceKind::ProBishop, Color::White)),
            (b"+r", Piece::new(PieceKind::ProRook, Color::White)),
        ];
        for (slice, piece) in tests {
            assert_eq!(
                Piece::parse_usi_slice(slice).unwrap(),
                (&[] as &[u8], piece),
            );
        }
    }
}
