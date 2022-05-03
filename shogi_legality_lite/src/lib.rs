#![cfg_attr(
    all(feature = "alloc", not(feature = "std")),
    feature(alloc_error_handler)
)]
#![cfg_attr(not(feature = "std"), no_std)] // Forbids using std::*.

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(not(feature = "std"))]
mod no_std;

use shogi_core::{GameStatus, LegalityChecker, Move, PartialPosition};

pub struct LiteLegalityChecker;

impl LegalityChecker for LiteLegalityChecker {
    #[allow(unused)]
    #[cfg(feature = "alloc")]
    fn status(&self, position: &shogi_core::Position) -> GameStatus {
        todo!()
    }

    #[allow(unused)]
    fn status_partial(&self, position: &PartialPosition) -> GameStatus {
        todo!()
    }

    #[allow(unused)]
    fn is_legal_partial(&self, position: &PartialPosition, mv: Move) -> bool {
        todo!()
    }

    #[cfg(feature = "alloc")]
    fn all_legal_moves_partial(&self, position: &PartialPosition) -> alloc::vec::Vec<Move> {
        use shogi_core::Square;

        let mut result = alloc::vec::Vec::new();
        for from in Square::all() {
            for to in Square::all() {
                for promote in [true, false] {
                    let mv = Move::Normal { from, to, promote };
                    if self.is_legal_partial(position, mv) {
                        result.push(mv);
                    }
                }
            }
        }
        for piece in shogi_core::Piece::all() {
            for to in Square::all() {
                let mv = Move::Drop { piece, to };
                if self.is_legal_partial(position, mv) {
                    result.push(mv);
                }
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
