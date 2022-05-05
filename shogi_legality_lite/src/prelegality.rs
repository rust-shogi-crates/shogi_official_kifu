use shogi_core::{Color, Move, PartialPosition, Piece, PieceKind, Square};

pub fn check(position: &PartialPosition, mv: Move) -> bool {
    let side = position.side_to_move();
    match mv {
        Move::Normal { from, to, promote } => {
            // Is `from` occupied by `side`'s piece?
            let from_piece = if let Some(x) = position.piece_at(from) {
                x
            } else {
                return false;
            };
            if from_piece.color() != side {
                return false;
            }
            // Is `to` occupied by `side`'s piece?
            let to_piece = position.piece_at(to);
            if let Some(x) = to_piece {
                if x.color() == side {
                    return false;
                }
            }
            // Stuck?
            let rel_rank = to.relative_rank(side);
            if rel_rank == 1
                && matches!(
                    from_piece.piece_kind(),
                    PieceKind::Pawn | PieceKind::Lance | PieceKind::Knight,
                )
                && !promote
            {
                return false;
            }
            if rel_rank == 2 && from_piece.piece_kind() == PieceKind::Knight && !promote {
                return false;
            }
            // Can promote?
            if promote && from.relative_rank(side) > 3 && to.relative_rank(side) > 3 {
                return false;
            }
            // Is the move valid?
            crate::normal::check(position, from_piece, from, to)
        }
        Move::Drop { piece, to } => {
            // Does `side` have a piece?
            if piece.color() != side {
                return false;
            }
            let remaining = if let Some(x) = position.hand(piece) {
                x
            } else {
                return false;
            };
            if remaining == 0 {
                return false;
            }
            // Is `to` vacant?
            if position.piece_at(to).is_some() {
                return false;
            }
            // Stuck?
            let rel_rank = to.relative_rank(side);
            if rel_rank == 1
                && matches!(
                    piece.piece_kind(),
                    PieceKind::Pawn | PieceKind::Lance | PieceKind::Knight,
                )
            {
                return false;
            }
            if rel_rank == 2 && piece.piece_kind() == PieceKind::Knight {
                return false;
            }
            // Does a drop-pawn-mate (`打ち歩詰め`, *uchifu-zume*) happen?
            if piece.piece_kind() == PieceKind::Pawn {
                let mut next = position.clone();
                let result = next.make_move(mv); // always Some(())
                debug_assert_eq!(result, Some(()));
                if is_mate(&next) != Some(false) {
                    return false;
                }
            }
            true
        }
    }
}

#[allow(unused)]
pub fn all_legal_moves(position: &PartialPosition) -> impl Iterator<Item = Move> + '_ {
    Square::all()
        .flat_map(|from| {
            Square::all().flat_map(move |to| {
                [false, true]
                    .into_iter()
                    .map(move |promote| Move::Normal { from, to, promote })
            })
        })
        .chain(
            Piece::all()
                .into_iter()
                .flat_map(|piece| Square::all().map(move |to| Move::Drop { piece, to })),
        )
        .filter(|&mv| check(position, mv))
}

// Can `side` play a move that captures the opponent's king?
pub fn will_king_be_captured(position: &PartialPosition) -> Option<bool> {
    let side = position.side_to_move();
    let king = king_position(position, side.flip())?;
    for from in Square::all() {
        let piece = if let Some(x) = position.piece_at(from) {
            x
        } else {
            continue;
        };
        if piece.color() != side {
            continue;
        }
        if crate::normal::check(position, piece, from, king) {
            return Some(true);
        }
    }
    Some(false)
}

// TODO: move to shogi_core (PartialPosition)
fn king_position(position: &PartialPosition, color: Color) -> Option<Square> {
    let king = Piece::new(PieceKind::King, color);
    for square in Square::all() {
        if position.piece_at(square) == Some(king) {
            return Some(square);
        }
    }
    None
}

// The king does not need to be in check.
fn is_mate(position: &PartialPosition) -> Option<bool> {
    let all = all_legal_moves(position);
    for mv in all {
        let mut next = position.clone();
        let result = next.make_move(mv);
        debug_assert_eq!(result, Some(()));
        if !will_king_be_captured(&next)? {
            return Some(false);
        }
    }
    Some(true)
}
