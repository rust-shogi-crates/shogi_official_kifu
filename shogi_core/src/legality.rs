#[cfg(feature = "alloc")]
use crate::Position;
use crate::{Move, PartialPosition};

/// A trait that handles legality checking.
///
/// This crate does not provide any implementors of `LegalityChecker`:
/// users of this crate should depend on a crate that has an implementor of `LegalityChecker`.
pub trait LegalityChecker {
    #[cfg(feature = "alloc")]
    fn is_valid_position(&self, position: &Position) -> bool;
    fn is_valid_position_partial(&self, position: &PartialPosition) -> bool;
    #[cfg(feature = "alloc")]
    fn is_legal(&self, position: &Position, mv: Move) -> bool;
    fn is_legal_partial(&self, position: &PartialPosition, mv: Move) -> bool;
    #[cfg(feature = "alloc")]
    fn all_legal_moves(&self, position: &PartialPosition) -> alloc::vec::Vec<Move>;
    #[cfg(feature = "alloc")]
    fn make_move(&self, position: &mut Position, mv: Move) {
        if self.is_legal(position, mv) {
            position.make_move(mv);
        }
    }
}
