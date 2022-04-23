use core::num::NonZeroU8;

use crate::{Color, PieceKind};

/// A piece + who owns it.
///
/// `Piece` and `Option<Piece>` are both 1-byte data types.
/// Because they are cheap to copy, they implement [`Copy`](https://doc.rust-lang.org/core/marker/trait.Copy.html).
///
/// Examples:
/// ```
/// use shogi_core::Piece;
/// assert_eq!(core::mem::size_of::<Piece>(), 1);
/// ```
#[repr(C)]
#[derive(Eq, PartialEq, Clone, Copy, Debug)]
#[cfg_attr(feature = "ord", derive(PartialOrd, Ord))]
#[cfg_attr(feature = "hash", derive(Hash))]
// Internal representation: 1..=14: black, 16..=30: white
pub struct Piece(NonZeroU8);

impl Piece {
    /// Creates a new `Piece` from `PieceKind` and `Color`.
    #[must_use]
    #[export_name = "Piece_new"]
    pub extern "C" fn new(piece_kind: PieceKind, color: Color) -> Self {
        let disc = piece_kind as u8;
        let value = disc
            + match color {
                Color::Black => 0,
                Color::White => 16,
            };
        // Safety: disc > 0 always holds
        Piece(unsafe { NonZeroU8::new_unchecked(value) })
    }
    /// An inverse of `new`. Finds `PieceKind` and `Color` from a `Piece`.
    #[must_use]
    pub fn to_parts(self) -> (PieceKind, Color) {
        let data = self.0.get();
        let disc = data & 15;
        (
            // Safety: 1 <= disc <= 14
            unsafe { PieceKind::from_u8(disc) },
            if data >= 16 {
                Color::White
            } else {
                Color::Black
            },
        )
    }
    /// Finds the `PieceKind` of this piece.
    #[must_use]
    #[export_name = "Piece_piece_kind"]
    pub extern "C" fn piece_kind(self) -> PieceKind {
        self.to_parts().0
    }
    /// Finds the `Color` of this piece.
    #[must_use]
    #[export_name = "Piece_color"]
    pub extern "C" fn color(self) -> Color {
        self.to_parts().1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_parts_works() {
        let piece_kinds = crate::PieceKind::all();
        let colors = crate::Color::all();
        for &piece_kind in &piece_kinds {
            for &color in &colors {
                let piece = Piece::new(piece_kind, color);
                let (piece_kind0, color0) = piece.to_parts();
                assert_eq!(piece_kind0, piece_kind);
                assert_eq!(color0, color);
            }
        }
    }
}
