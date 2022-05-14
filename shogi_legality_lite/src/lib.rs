#![cfg_attr(not(test), no_std)] // Forbids using std::*.

#[cfg(feature = "alloc")]
extern crate alloc;

use shogi_core::{
    Bitboard, IllegalMoveKind, LegalityChecker, Move, PartialPosition, Piece, PositionStatus,
    Square,
};

mod normal;
mod prelegality;

pub struct LiteLegalityChecker;

impl LegalityChecker for LiteLegalityChecker {
    #[allow(unused)]
    #[cfg(feature = "alloc")]
    fn status(&self, position: &shogi_core::Position) -> PositionStatus {
        todo!()
    }

    #[allow(unused)]
    fn status_partial(&self, position: &PartialPosition) -> PositionStatus {
        todo!()
    }

    fn is_legal_partial(
        &self,
        position: &PartialPosition,
        mv: Move,
    ) -> Result<(), IllegalMoveKind> {
        if !prelegality::check(position, mv) {
            return Err(IllegalMoveKind::IncorrectMove);
        }
        let mut next = position.clone();
        if next.make_move(mv).is_none() {}
        if prelegality::will_king_be_captured(&next) != Some(false) {
            return Err(IllegalMoveKind::IncorrectMove);
        }
        Ok(())
    }

    fn is_legal_partial_lite(&self, position: &PartialPosition, mv: Move) -> bool {
        if !prelegality::check(position, mv) {
            return false;
        }
        let mut next = position.clone();
        if next.make_move(mv).is_none() {
            return false;
        }
        if prelegality::will_king_be_captured(&next) != Some(false) {
            return false;
        }
        true
    }

    #[cfg(feature = "alloc")]
    fn all_legal_moves_partial(&self, position: &PartialPosition) -> alloc::vec::Vec<Move> {
        let mut result = alloc::vec::Vec::new();
        for from in Square::all() {
            for to in Square::all() {
                for promote in [true, false] {
                    let mv = Move::Normal { from, to, promote };
                    if self.is_legal_partial_lite(position, mv) {
                        result.push(mv);
                    }
                }
            }
        }
        for piece in shogi_core::Piece::all() {
            for to in Square::all() {
                let mv = Move::Drop { piece, to };
                if self.is_legal_partial_lite(position, mv) {
                    result.push(mv);
                }
            }
        }
        result
    }

    fn normal_from_candidates(&self, position: &PartialPosition, from: Square) -> Bitboard {
        let mut result = Bitboard::empty();
        for to in Square::all() {
            for promote in [true, false] {
                let mv = Move::Normal { from, to, promote };
                if self.is_legal_partial_lite(position, mv) {
                    result |= to;
                }
            }
        }
        result
    }

    fn normal_to_candidates(
        &self,
        position: &PartialPosition,
        to: Square,
        piece: Piece,
    ) -> Bitboard {
        let mut result = Bitboard::empty();
        for from in Square::all() {
            for promote in [true, false] {
                let mv = Move::Normal { from, to, promote };
                if self.is_legal_partial_lite(position, mv)
                    && position.piece_at(from) == Some(piece)
                {
                    result |= from;
                }
            }
        }
        result
    }

    fn drop_candidates(&self, position: &PartialPosition, piece: Piece) -> Bitboard {
        let mut result = Bitboard::empty();
        for to in Square::all() {
            let mv = Move::Drop { piece, to };
            if self.is_legal_partial_lite(position, mv) {
                result |= to;
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_legal_moves_partial_works() {
        let position = PartialPosition::startpos();
        let first_moves = LiteLegalityChecker.all_legal_moves_partial(&position);
        assert_eq!(first_moves.len(), 30);
    }
}
