use shogi_core::{PartialPosition, Piece, Square};

/// Checks if the normal move is legal.
///
/// `piece` is given as a hint and `position.piece_at(from) == Some(piece)` must hold.
#[allow(unused)]
pub fn check(position: &PartialPosition, piece: Piece, from: Square, to: Square) -> bool {
    todo!();
}
