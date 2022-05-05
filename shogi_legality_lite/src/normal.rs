use shogi_core::{Bitboard, Color, PartialPosition, Piece, PieceKind, Square};

/// Checks if the normal move is legal.
///
/// `piece` is given as a hint and `position.piece_at(from) == Some(piece)` must hold.
#[allow(unused)]
#[export_name = "legality_normal_check"]
pub extern "C" fn check(
    position: &PartialPosition,
    piece: Piece,
    from: Square,
    to: Square,
) -> bool {
    let attacking = attacking(position, piece, from);
    attacking.contains(to)
}

pub fn attacking(position: &PartialPosition, piece: Piece, from: Square) -> Bitboard {
    debug_assert_eq!(position.side_to_move(), piece.color());
    debug_assert_eq!(position.piece_at(from), Some(piece));
    // Is `piece` long-range?
    if matches!(
        piece.piece_kind(),
        PieceKind::Lance
            | PieceKind::Bishop
            | PieceKind::Rook
            | PieceKind::ProBishop
            | PieceKind::ProRook
    ) {
        let occupied = !position.vacant_bitboard();
        let range = match piece.piece_kind() {
            PieceKind::Lance => lance_range(piece.color(), from, occupied),
            PieceKind::Bishop => bishop_range(from, occupied),
            PieceKind::Rook => rook_range(from, occupied),
            PieceKind::ProBishop => bishop_range(from, occupied) | king(from),
            PieceKind::ProRook => rook_range(from, occupied) | king(from),
            _ => unreachable!(),
        };
        return range & !position.player_bitboard(piece.color());
    }
    // `piece` is short-range, i.e., no blocking is possible
    // no need to consider the possibility of blockading by pieces
    let range = unsafe { short_range(piece, from) };
    range & !position.player_bitboard(piece.color())
}

// Safety: `piece` must be short-range, i.e., `piece`'s move cannot be blockaded
unsafe fn short_range(piece: Piece, from: Square) -> Bitboard {
    match piece.piece_kind() {
        PieceKind::Pawn => pawn(piece.color(), from),
        PieceKind::Knight => knight(piece.color(), from),
        PieceKind::Silver => silver(piece.color(), from),
        PieceKind::Gold
        | PieceKind::ProPawn
        | PieceKind::ProLance
        | PieceKind::ProKnight
        | PieceKind::ProSilver => gold(piece.color(), from),
        PieceKind::King => king(from),
        PieceKind::Lance
        | PieceKind::Bishop
        | PieceKind::Rook
        | PieceKind::ProBishop
        | PieceKind::ProRook => core::hint::unreachable_unchecked(),
    }
}

// If `from` is on the 9th row (i.e., a pawn cannot move), the result is unspecified.
// That being said, since this function is not marked unsafe, this cannot cause UB.
fn pawn(color: Color, from: Square) -> Bitboard {
    let index = from.index();
    match color {
        Color::Black => {
            if index > 1 {
                Bitboard::single(unsafe { Square::from_u8_unchecked(index - 1) })
            } else {
                Bitboard::empty()
            }
        }
        Color::White => {
            if index < 81 {
                Bitboard::single(unsafe { Square::from_u8_unchecked(index + 1) })
            } else {
                Bitboard::empty()
            }
        }
    }
}

fn knight(color: Color, from: Square) -> Bitboard {
    let rank = from.relative_rank(color);
    if rank <= 2 {
        return Bitboard::empty();
    }
    let file = from.relative_file(color);
    let mut result = Bitboard::empty();
    if file >= 2 {
        // Safety: file - 1 >= 1, rank - 2 >= 1
        result |= unsafe { Square::new_relative(file - 1, rank - 2, color).unwrap_unchecked() };
    }
    if file <= 8 {
        // Safety: file + 1 <= 9, rank - 2 >= 1
        result |= unsafe { Square::new_relative(file + 1, rank - 2, color).unwrap_unchecked() };
    }
    result
}

fn silver(color: Color, from: Square) -> Bitboard {
    use core::cmp::{max, min};

    let file = from.relative_file(color);
    let rank = from.relative_rank(color);
    let mut result = Bitboard::empty();
    if rank >= 2 {
        for to_file in max(1, file - 1)..=min(9, file + 1) {
            // Safety: `to_file` and `rank - 1` are both in `1..=9`.
            result |= unsafe { Square::new_relative(to_file, rank - 1, color).unwrap_unchecked() };
        }
    }
    if rank <= 8 {
        if file <= 8 {
            // Safety: `file + 1` and `rank + 1` are both in `1..=9`.
            result |= unsafe { Square::new_relative(file + 1, rank + 1, color).unwrap_unchecked() };
        }
        if file >= 2 {
            // Safety: `file - 1` and `rank + 1` are both in `1..=9`.
            result |= unsafe { Square::new_relative(file - 1, rank + 1, color).unwrap_unchecked() };
        }
    }
    result
}

fn gold(color: Color, from: Square) -> Bitboard {
    use core::cmp::{max, min};

    let file = from.relative_file(color);
    let rank = from.relative_rank(color);
    let mut result = Bitboard::empty();
    for to_file in max(1, file - 1)..=min(9, file + 1) {
        for to_rank in max(1, rank - 1)..=rank {
            // Safety: `to_file` and `to_rank` are both in `1..=9`.
            result |= unsafe { Square::new_relative(to_file, to_rank, color).unwrap_unchecked() };
        }
    }
    if rank <= 8 {
        // Safety: `file` and `rank + 1` are both in `1..=9`.
        result |= unsafe { Square::new_relative(file, rank + 1, color).unwrap_unchecked() };
    }
    result ^= from; // Cannot move to the original square
    result
}

fn king(from: Square) -> Bitboard {
    use core::cmp::{max, min};

    let file = from.file();
    let rank = from.rank();
    let mut result = Bitboard::empty();
    for to_file in max(1, file - 1)..=min(9, file + 1) {
        for to_rank in max(1, rank - 1)..=min(9, rank + 1) {
            // Safety: `to_file` and `to_rank` are both in 1..=9.
            result |= unsafe { Square::new(to_file, to_rank).unwrap_unchecked() };
        }
    }
    result ^= from; // Cannot move to the original square
    result
}

fn lance_range(color: Color, from: Square, occupied: Bitboard) -> Bitboard {
    let direction = match color {
        Color::Black => (0, -1),
        Color::White => (0, 1),
    };
    long_range(from, occupied, direction)
}

fn bishop_range(from: Square, occupied: Bitboard) -> Bitboard {
    let directions = [(1, 1), (1, -1), (-1, 1), (-1, -1)];
    let mut result = Bitboard::empty();
    for direction in directions {
        result |= long_range(from, occupied, direction);
    }
    result
}

fn rook_range(from: Square, occupied: Bitboard) -> Bitboard {
    let directions = [(0, 1), (0, -1), (1, 0), (-1, 0)];
    let mut result = Bitboard::empty();
    for direction in directions {
        result |= long_range(from, occupied, direction);
    }
    result
}

fn long_range(from: Square, occupied: Bitboard, (file_delta, rank_delta): (i8, i8)) -> Bitboard {
    let mut result = Bitboard::empty();
    let mut current = from;
    while let Some(next) = current.shift(file_delta, rank_delta) {
        // `result` includes the blocking piece's square.
        result |= next;
        if occupied.contains(next) {
            break;
        }
        current = next;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    // Utility function. If the arguments are out of range, this function panics.
    fn single(file: u8, rank: u8) -> Bitboard {
        Bitboard::single(Square::new(file, rank).unwrap())
    }

    #[test]
    fn pawn_moves_are_correct() {
        let position = PartialPosition::startpos();
        let pawn = Piece::new(PieceKind::Pawn, Color::Black);
        let pawn_square = Square::new(7, 7).unwrap();
        let attacking = attacking(&position, pawn, pawn_square);
        assert_eq!(attacking, single(7, 6));

        // Exhaustive checking: `super::pawn` cannot panic or cause UB
        for color in Color::all() {
            for square in Square::all() {
                if square.relative_rank(color) == 1 {
                    let _ = super::pawn(color, square);
                    continue;
                }
                let result = super::pawn(color, square);
                assert_eq!(result.count(), 1);
            }
        }
        // Compatibility with `flip`
        for square in Square::all() {
            let result_black = super::pawn(Color::Black, square);
            let result_white = super::pawn(Color::White, square.flip());
            assert_eq!(result_white.flip(), result_black);
        }
    }

    #[test]
    fn knight_moves_are_correct() {
        use shogi_core::Move;

        let mut position = PartialPosition::startpos();
        let moves = [
            Move::Normal {
                from: Square::new(7, 7).unwrap(),
                to: Square::new(7, 6).unwrap(),
                promote: false,
            },
            Move::Normal {
                from: Square::new(3, 3).unwrap(),
                to: Square::new(3, 4).unwrap(),
                promote: false,
            },
        ];
        for mv in moves {
            position.make_move(mv).unwrap();
        }
        let knight = Piece::new(PieceKind::Knight, Color::Black);
        let knight_square = Square::new(8, 9).unwrap();
        let attacking = attacking(&position, knight, knight_square);
        assert_eq!(attacking, single(7, 7));
    }

    #[test]
    fn silver_moves_are_correct() {
        let position = PartialPosition::startpos();
        let silver = Piece::new(PieceKind::Silver, Color::Black);
        let silver_square = Square::new(3, 9).unwrap();
        let attacking = attacking(&position, silver, silver_square);
        let expected = single(3, 8) | single(4, 8);
        assert_eq!(attacking, expected);

        let square = Square::new(8, 1).unwrap();
        let expected = single(7, 2) | single(9, 2);
        assert_eq!(super::silver(Color::Black, square), expected);

        let square = Square::new(8, 1).unwrap();
        let expected = single(7, 2) | single(8, 2) | single(9, 2);
        assert_eq!(super::silver(Color::White, square), expected);

        // Exhaustive checking: `super::silver` cannot panic or cause UB
        for color in Color::all() {
            for square in Square::all() {
                let result = super::silver(color, square);
                assert!(result.count() <= 5);
            }
        }
        // Compatibility with `flip`
        for square in Square::all() {
            let result_black = super::silver(Color::Black, square);
            let result_white = super::silver(Color::White, square.flip());
            assert_eq!(result_white.flip(), result_black);
        }
    }

    #[test]
    fn gold_moves_are_correct() {
        let position = PartialPosition::startpos();
        let gold = Piece::new(PieceKind::Gold, Color::Black);
        let gold_square = Square::new(4, 9).unwrap();
        let attacking = attacking(&position, gold, gold_square);
        let expected = single(3, 8) | single(4, 8) | single(5, 8);
        assert_eq!(attacking, expected);

        let square = Square::new(8, 1).unwrap();
        let expected = single(7, 1) | single(8, 2) | single(9, 1);
        assert_eq!(super::gold(Color::Black, square), expected);

        let square = Square::new(8, 1).unwrap();
        let expected = single(7, 1) | single(7, 2) | single(8, 2) | single(9, 1) | single(9, 2);
        assert_eq!(super::gold(Color::White, square), expected);

        // Exhaustive checking: `super::gold` cannot panic or cause UB
        for color in Color::all() {
            for square in Square::all() {
                let result = super::gold(color, square);
                assert!(result.count() <= 6);
            }
        }
        // Compatibility with `flip`
        for square in Square::all() {
            let result_black = super::gold(Color::Black, square);
            let result_white = super::gold(Color::White, square.flip());
            assert_eq!(result_white.flip(), result_black);
        }
    }

    #[test]
    fn king_moves_are_correct() {
        let position = PartialPosition::startpos();
        let king = Piece::new(PieceKind::King, Color::Black);
        let king_square = Square::new(5, 9).unwrap();
        let attacking = attacking(&position, king, king_square);
        let expected = single(4, 8) | single(5, 8) | single(6, 8);
        assert_eq!(attacking, expected);
    }

    #[test]
    fn bishop_moves_are_correct() {
        use shogi_core::Move;

        let mut position = PartialPosition::startpos();
        let moves = [
            Move::Normal {
                from: Square::new(7, 7).unwrap(),
                to: Square::new(7, 6).unwrap(),
                promote: false,
            },
            Move::Normal {
                from: Square::new(3, 3).unwrap(),
                to: Square::new(3, 4).unwrap(),
                promote: false,
            },
        ];
        for mv in moves {
            position.make_move(mv).unwrap();
        }
        let bishop = Piece::new(PieceKind::Bishop, Color::Black);
        let bishop_square = Square::new(8, 8).unwrap();
        let attacking = attacking(&position, bishop, bishop_square);
        let expected =
            single(2, 2) | single(3, 3) | single(4, 4) | single(5, 5) | single(6, 6) | single(7, 7);
        assert_eq!(attacking, expected);
    }

    #[test]
    fn rook_moves_are_correct() {
        let position = PartialPosition::startpos();
        let rook = Piece::new(PieceKind::Rook, Color::Black);
        let rook_square = Square::new(2, 8).unwrap();
        let attacking = attacking(&position, rook, rook_square);
        let expected =
            single(1, 8) | single(3, 8) | single(4, 8) | single(5, 8) | single(6, 8) | single(7, 8);
        assert_eq!(attacking, expected);
    }
}
