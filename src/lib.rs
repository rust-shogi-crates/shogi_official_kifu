use core::fmt::Write;
use shogi_core::{
    Bitboard, Color, LegalityChecker, Move, PartialPosition, Piece, PieceKind, Square,
};
use shogi_legality_lite::LiteLegalityChecker;

/// Disambiguation of normal moves.
mod disambiguation;

const SANYOU_SUJI: [char; 9] = ['１', '２', '３', '４', '５', '６', '７', '８', '９'];
#[cfg(feature = "kansuji")]
const KANSUJI: [char; 9] = ['一', '二', '三', '四', '五', '六', '七', '八', '九'];

/// https://www.shogi.or.jp/faq/kihuhyouki.html
#[cfg(feature = "std")]
pub fn display_single_move(position: &PartialPosition, mv: Move) -> Option<String> {
    let mut ret = "".to_string();
    display_single_move_write(position, mv, &mut ret)
        .expect("fmt::Write for String cannot return an error")?;
    Some(ret)
}

/// https://www.shogi.or.jp/faq/kihuhyouki.html
#[cfg(all(feature = "std", feature = "kansuji"))]
pub fn display_single_move_kansuji(position: &PartialPosition, mv: Move) -> Option<String> {
    let mut ret = "".to_string();
    display_single_move_write_kansuji(position, mv, &mut ret)
        .expect("fmt::Write for String cannot return an error")?;
    Some(ret)
}

/// https://www.shogi.or.jp/faq/kihuhyouki.html
pub fn display_single_move_write<W: Write>(
    position: &PartialPosition,
    mv: Move,
    w: &mut W,
) -> Result<Option<()>, std::fmt::Error> {
    if let Some(to) = write_side_and_find_to(position, mv, w)? {
        w.write_char(SANYOU_SUJI[to.file() as usize - 1])?;
        w.write_char(SANYOU_SUJI[to.rank() as usize - 1])?;
    }
    disambiguate(position, mv, w)
}

/// Traditional move notation, usually found in books, magazines, articles.
#[cfg(feature = "kansuji")]
pub fn display_single_move_write_kansuji<W: Write>(
    position: &PartialPosition,
    mv: Move,
    w: &mut W,
) -> Result<Option<()>, std::fmt::Error> {
    if let Some(to) = write_side_and_find_to(position, mv, w)? {
        w.write_char(SANYOU_SUJI[to.file() as usize - 1])?;
        w.write_char(KANSUJI[to.rank() as usize - 1])?;
    }
    disambiguate(position, mv, w)
}

/// Returns Ok(Some((to, should_continue))) when the call was successful.
/// If unsuccessful, this functions tries not to write to w, but it is in a best-effort basis.
fn write_side_and_find_to<W: Write>(
    position: &PartialPosition,
    mv: Move,
    w: &mut W,
) -> Result<Option<Square>, std::fmt::Error> {
    let side = position.side_to_move();
    let side_color = if side == Color::Black { '▲' } else { '△' };
    let to = match mv {
        Move::Normal { from, to, .. } => {
            let last_move = position.last_move();
            if let Some(Move::Normal { to: last_to, .. }) = last_move {
                if last_to == to {
                    if position.piece_at(from).is_none() {
                        return Ok(None);
                    }
                    w.write_char(side_color)?;
                    w.write_char('同')?;
                    return Ok(None);
                }
            }
            to
        }
        Move::Drop { to, .. } => to,
    };
    w.write_char(side_color)?;
    Ok(Some(to))
}

fn disambiguate<W: Write>(
    position: &PartialPosition,
    mv: Move,
    w: &mut W,
) -> Result<Option<()>, std::fmt::Error> {
    let all_moves = LiteLegalityChecker.all_legal_moves_partial(position);
    match mv {
        Move::Normal { from, to, promote } => {
            let p = if let Some(p) = position.piece_at(from) {
                p
            } else {
                return Ok(None);
            };
            w.write_str(piece_kind_to_kanji(p.piece_kind()))?;
            let mut candidates = Bitboard::empty();
            for mv in all_moves {
                if let Move::Normal {
                    from, to: mv_to, ..
                } = mv
                {
                    if mv_to != to {
                        continue;
                    }
                    if position.piece_at(from) != Some(p) {
                        continue;
                    }
                    candidates |= from;
                }
            }
            if disambiguation::run(position, from, to, candidates, w)?.is_none() {
                return Ok(None);
            }
            let side = position.side_to_move();
            let could_promote = is_promotable_piece(p.piece_kind())
                && (from.relative_rank(side) <= 3 || to.relative_rank(side) <= 3);
            if promote {
                w.write_char('成')?;
            } else if could_promote {
                w.write_str("不成")?;
            }
        }
        Move::Drop { to, piece } => {
            let piece_kind = piece.piece_kind();
            let side = position.side_to_move();
            w.write_str(piece_kind_to_kanji(piece_kind))?;
            let mut normal_possible = false;
            let p = Piece::new(piece_kind, side);
            for mv in all_moves {
                if let Move::Normal {
                    from, to: mv_to, ..
                } = mv
                {
                    if mv_to != to {
                        continue;
                    }
                    if position.piece_at(from) != Some(p) {
                        continue;
                    }
                    normal_possible = true;
                    break;
                }
            }
            if normal_possible {
                w.write_str("打")?
            }
        }
    }
    Ok(Some(()))
}

fn piece_kind_to_kanji(piece_kind: PieceKind) -> &'static str {
    match piece_kind {
        PieceKind::King => "玉",
        PieceKind::Rook => "飛",
        PieceKind::Bishop => "角",
        PieceKind::Gold => "金",
        PieceKind::Silver => "銀",
        PieceKind::Knight => "桂",
        PieceKind::Lance => "香",
        PieceKind::Pawn => "歩",
        PieceKind::ProRook => "竜",
        PieceKind::ProBishop => "馬",
        PieceKind::ProSilver => "成銀",
        PieceKind::ProKnight => "成桂",
        PieceKind::ProLance => "成香",
        PieceKind::ProPawn => "と",
    }
}

fn is_promotable_piece(piece_kind: PieceKind) -> bool {
    piece_kind.promote().is_some()
}

#[cfg(test)]
mod tests {
    use super::*;
    use shogi_usi_parser::FromUsi;

    #[test]
    fn normal_works_0() {
        let pos = PartialPosition::from_usi("sfen 4k4/9/9/8P/9/9/9/4G4/4K4 b G 1").unwrap();
        let mv = Move::Normal {
            from: Square::new(5, 8).unwrap(),
            to: Square::new(4, 8).unwrap(),
            promote: false,
        };
        let result = display_single_move(&pos, mv);
        assert_eq!(result, Some("▲４８金".to_string()));

        let mv = Move::Normal {
            from: Square::new(1, 4).unwrap(),
            to: Square::new(1, 3).unwrap(),
            promote: false,
        };
        let result = display_single_move(&pos, mv);
        assert_eq!(result, Some("▲１３歩不成".to_string()));

        let mv = Move::Normal {
            from: Square::new(1, 4).unwrap(),
            to: Square::new(1, 3).unwrap(),
            promote: true,
        };
        let result = display_single_move(&pos, mv);
        assert_eq!(result, Some("▲１３歩成".to_string()));
    }

    #[test]
    fn normal_works_1() {
        use shogi_core::Position;

        let pos = Position::from_usi("sfen 4k4/9/9/9/9/9/4g4/9/4KG3 w - 2 moves 5g5h").unwrap();
        let mv = Move::Normal {
            from: Square::new(4, 9).unwrap(),
            to: Square::new(5, 8).unwrap(),
            promote: false,
        };
        let result = display_single_move(pos.inner(), mv);
        assert_eq!(result, Some("▲同金".to_string()));

        let pos = Position::from_usi("sfen 4k4/9/9/9/9/9/3gG4/9/4KG3 w - 2 moves 6g5h").unwrap();
        let mv = Move::Normal {
            from: Square::new(4, 9).unwrap(),
            to: Square::new(5, 8).unwrap(),
            promote: false,
        };
        let result = display_single_move(pos.inner(), mv);
        assert_eq!(result, Some("▲同金上".to_string()));
        let mv = Move::Normal {
            from: Square::new(5, 7).unwrap(),
            to: Square::new(5, 8).unwrap(),
            promote: false,
        };
        let result = display_single_move(pos.inner(), mv);
        assert_eq!(result, Some("▲同金引".to_string()));
    }

    #[test]
    fn normal_works_2() {
        // Examples found in https://www.shogi.or.jp/faq/kihuhyouki.html.
        let pos = PartialPosition::from_usi("sfen 4k4/2G6/G8/9/9/9/9/9/4K4 b - 1").unwrap(); // A

        // A
        let mv = Move::Normal {
            from: Square::new(7, 2).unwrap(),
            to: Square::new(8, 2).unwrap(),
            promote: false,
        };
        let result = display_single_move(&pos, mv);
        assert_eq!(result, Some("▲８２金寄".to_string()));
        let mv = Move::Normal {
            from: Square::new(9, 3).unwrap(),
            to: Square::new(8, 2).unwrap(),
            promote: false,
        };
        let result = display_single_move(&pos, mv);
        assert_eq!(result, Some("▲８２金上".to_string()));

        let pos = PartialPosition::from_usi("sfen 4k1G2/9/5G3/9/9/9/9/9/4K4 b - 1").unwrap(); // B

        // B
        let mv = Move::Normal {
            from: Square::new(4, 3).unwrap(),
            to: Square::new(3, 2).unwrap(),
            promote: false,
        };
        let result = display_single_move(&pos, mv);
        assert_eq!(result, Some("▲３２金上".to_string()));
        let mv = Move::Normal {
            from: Square::new(3, 1).unwrap(),
            to: Square::new(3, 2).unwrap(),
            promote: false,
        };
        let result = display_single_move(&pos, mv);
        assert_eq!(result, Some("▲３２金引".to_string()));

        let pos = PartialPosition::from_usi("sfen 4k4/9/9/9/5G3/4G4/2S4S1/9/1S2KS3 b - 1").unwrap(); // C, D, E

        // C
        let mv = Move::Normal {
            from: Square::new(5, 6).unwrap(),
            to: Square::new(5, 5).unwrap(),
            promote: false,
        };
        let result = display_single_move(&pos, mv);
        assert_eq!(result, Some("▲５５金上".to_string()));
        let mv = Move::Normal {
            from: Square::new(4, 5).unwrap(),
            to: Square::new(5, 5).unwrap(),
            promote: false,
        };
        let result = display_single_move(&pos, mv);
        assert_eq!(result, Some("▲５５金寄".to_string()));
        let mv = Move::Normal {
            from: Square::new(8, 9).unwrap(),
            to: Square::new(8, 8).unwrap(),
            promote: false,
        };
        let result = display_single_move(&pos, mv);
        // D
        assert_eq!(result, Some("▲８８銀上".to_string()));
        let mv = Move::Normal {
            from: Square::new(7, 7).unwrap(),
            to: Square::new(8, 8).unwrap(),
            promote: false,
        };
        let result = display_single_move(&pos, mv);
        assert_eq!(result, Some("▲８８銀引".to_string()));
        let mv = Move::Normal {
            from: Square::new(4, 9).unwrap(),
            to: Square::new(3, 8).unwrap(),
            promote: false,
        };
        // E
        let result = display_single_move(&pos, mv);
        assert_eq!(result, Some("▲３８銀上".to_string()));
        let mv = Move::Normal {
            from: Square::new(2, 7).unwrap(),
            to: Square::new(3, 8).unwrap(),
            promote: false,
        };
        let result = display_single_move(&pos, mv);
        assert_eq!(result, Some("▲３８銀引".to_string()));
    }

    #[test]
    fn normal_works_3() {
        // Examples found in https://www.shogi.or.jp/faq/kihuhyouki.html.
        let pos = PartialPosition::from_usi("sfen 4k4/G1G3G1G/9/9/3S1S3/9/9/9/4K4 b - 1").unwrap(); // A, B, C

        // A
        let mv = Move::Normal {
            from: Square::new(9, 2).unwrap(),
            to: Square::new(8, 1).unwrap(),
            promote: false,
        };
        let result = display_single_move(&pos, mv);
        assert_eq!(result, Some("▲８１金左".to_string()));
        let mv = Move::Normal {
            from: Square::new(7, 2).unwrap(),
            to: Square::new(8, 1).unwrap(),
            promote: false,
        };
        let result = display_single_move(&pos, mv);
        assert_eq!(result, Some("▲８１金右".to_string()));

        // B
        let mv = Move::Normal {
            from: Square::new(3, 2).unwrap(),
            to: Square::new(2, 2).unwrap(),
            promote: false,
        };
        let result = display_single_move(&pos, mv);
        assert_eq!(result, Some("▲２２金左".to_string()));
        let mv = Move::Normal {
            from: Square::new(1, 2).unwrap(),
            to: Square::new(2, 2).unwrap(),
            promote: false,
        };
        let result = display_single_move(&pos, mv);
        assert_eq!(result, Some("▲２２金右".to_string()));

        // C
        let mv = Move::Normal {
            from: Square::new(6, 5).unwrap(),
            to: Square::new(5, 6).unwrap(),
            promote: false,
        };
        let result = display_single_move(&pos, mv);
        assert_eq!(result, Some("▲５６銀左".to_string()));
        let mv = Move::Normal {
            from: Square::new(4, 5).unwrap(),
            to: Square::new(5, 6).unwrap(),
            promote: false,
        };
        let result = display_single_move(&pos, mv);
        assert_eq!(result, Some("▲５６銀右".to_string()));

        let pos = PartialPosition::from_usi("sfen 4k4/9/9/9/9/9/9/9/1GG1K1SS1 b - 1").unwrap(); // D, E

        // D
        let mv = Move::Normal {
            from: Square::new(8, 9).unwrap(),
            to: Square::new(7, 8).unwrap(),
            promote: false,
        };
        let result = display_single_move(&pos, mv);
        assert_eq!(result, Some("▲７８金左".to_string()));
        let mv = Move::Normal {
            from: Square::new(7, 9).unwrap(),
            to: Square::new(7, 8).unwrap(),
            promote: false,
        };
        let result = display_single_move(&pos, mv);
        assert_eq!(result, Some("▲７８金直".to_string()));

        // E
        let mv = Move::Normal {
            from: Square::new(3, 9).unwrap(),
            to: Square::new(3, 8).unwrap(),
            promote: false,
        };
        let result = display_single_move(&pos, mv);
        assert_eq!(result, Some("▲３８銀直".to_string()));
        let mv = Move::Normal {
            from: Square::new(2, 9).unwrap(),
            to: Square::new(3, 8).unwrap(),
            promote: false,
        };
        let result = display_single_move(&pos, mv);
        assert_eq!(result, Some("▲３８銀右".to_string()));
    }

    #[test]
    fn normal_works_4() {
        // Examples found in https://www.shogi.or.jp/faq/kihuhyouki.html.
        let pos =
            PartialPosition::from_usi("sfen 4k4/9/3GGG3/9/9/9/1+P4S1S/+P8/+P+P+P1K1SS1 b - 1")
                .unwrap(); // A, B, C

        // A
        let mv = Move::Normal {
            from: Square::new(6, 3).unwrap(),
            to: Square::new(5, 2).unwrap(),
            promote: false,
        };
        let result = display_single_move(&pos, mv);
        assert_eq!(result, Some("▲５２金左".to_string()));
        let mv = Move::Normal {
            from: Square::new(5, 3).unwrap(),
            to: Square::new(5, 2).unwrap(),
            promote: false,
        };
        let result = display_single_move(&pos, mv);
        assert_eq!(result, Some("▲５２金直".to_string()));
        let mv = Move::Normal {
            from: Square::new(4, 3).unwrap(),
            to: Square::new(5, 2).unwrap(),
            promote: false,
        };
        let result = display_single_move(&pos, mv);
        assert_eq!(result, Some("▲５２金右".to_string()));

        // B
        let mv = Move::Normal {
            from: Square::new(7, 9).unwrap(),
            to: Square::new(8, 8).unwrap(),
            promote: false,
        };
        let result = display_single_move(&pos, mv);
        assert_eq!(result, Some("▲８８と右".to_string()));
        let mv = Move::Normal {
            from: Square::new(8, 9).unwrap(),
            to: Square::new(8, 8).unwrap(),
            promote: false,
        };
        let result = display_single_move(&pos, mv);
        assert_eq!(result, Some("▲８８と直".to_string()));
        let mv = Move::Normal {
            from: Square::new(9, 9).unwrap(),
            to: Square::new(8, 8).unwrap(),
            promote: false,
        };
        let result = display_single_move(&pos, mv);
        assert_eq!(result, Some("▲８８と左上".to_string()));
        let mv = Move::Normal {
            from: Square::new(9, 8).unwrap(),
            to: Square::new(8, 8).unwrap(),
            promote: false,
        };
        let result = display_single_move(&pos, mv);
        assert_eq!(result, Some("▲８８と寄".to_string()));
        let mv = Move::Normal {
            from: Square::new(8, 7).unwrap(),
            to: Square::new(8, 8).unwrap(),
            promote: false,
        };
        let result = display_single_move(&pos, mv);
        assert_eq!(result, Some("▲８８と引".to_string()));

        // C
        let mv = Move::Normal {
            from: Square::new(2, 9).unwrap(),
            to: Square::new(2, 8).unwrap(),
            promote: false,
        };
        let result = display_single_move(&pos, mv);
        assert_eq!(result, Some("▲２８銀直".to_string()));
        let mv = Move::Normal {
            from: Square::new(1, 7).unwrap(),
            to: Square::new(2, 8).unwrap(),
            promote: false,
        };
        let result = display_single_move(&pos, mv);
        assert_eq!(result, Some("▲２８銀右".to_string()));
        let mv = Move::Normal {
            from: Square::new(3, 9).unwrap(),
            to: Square::new(2, 8).unwrap(),
            promote: false,
        };
        let result = display_single_move(&pos, mv);
        assert_eq!(result, Some("▲２８銀左上".to_string()));
        let mv = Move::Normal {
            from: Square::new(3, 7).unwrap(),
            to: Square::new(2, 8).unwrap(),
            promote: false,
        };
        let result = display_single_move(&pos, mv);
        assert_eq!(result, Some("▲２８銀左引".to_string()));
    }

    #[test]
    fn normal_works_5() {
        // Examples found in https://www.shogi.or.jp/faq/kihuhyouki.html.
        // A
        let pos = PartialPosition::from_usi("sfen +R8/9/9/1+R7/9/9/9/9/4K1k2 b - 1").unwrap();
        let mv = Move::Normal {
            from: Square::new(9, 1).unwrap(),
            to: Square::new(8, 2).unwrap(),
            promote: false,
        };
        let result = display_single_move(&pos, mv);
        assert_eq!(result, Some("▲８２竜引".to_string()));
        let mv = Move::Normal {
            from: Square::new(8, 4).unwrap(),
            to: Square::new(8, 2).unwrap(),
            promote: false,
        };
        let result = display_single_move(&pos, mv);
        assert_eq!(result, Some("▲８２竜上".to_string()));

        // B
        let pos = PartialPosition::from_usi("sfen 9/4+R4/7+R1/9/9/9/9/9/2k1K4 b - 1").unwrap();
        let mv = Move::Normal {
            from: Square::new(2, 3).unwrap(),
            to: Square::new(4, 3).unwrap(),
            promote: false,
        };
        let result = display_single_move(&pos, mv);
        assert_eq!(result, Some("▲４３竜寄".to_string()));
        let mv = Move::Normal {
            from: Square::new(5, 2).unwrap(),
            to: Square::new(4, 3).unwrap(),
            promote: false,
        };
        let result = display_single_move(&pos, mv);
        assert_eq!(result, Some("▲４３竜引".to_string()));

        // C
        let pos = PartialPosition::from_usi("sfen 9/9/9/9/4+R3+R/9/9/9/2k1K4 b - 1").unwrap();
        let mv = Move::Normal {
            from: Square::new(5, 5).unwrap(),
            to: Square::new(3, 5).unwrap(),
            promote: false,
        };
        let result = display_single_move(&pos, mv);
        assert_eq!(result, Some("▲３５竜左".to_string()));
        let mv = Move::Normal {
            from: Square::new(1, 5).unwrap(),
            to: Square::new(3, 5).unwrap(),
            promote: false,
        };
        let result = display_single_move(&pos, mv);
        assert_eq!(result, Some("▲３５竜右".to_string()));

        // D
        let pos = PartialPosition::from_usi("sfen 9/9/9/9/9/9/9/9/+R+R2K1k2 b - 1").unwrap();
        let mv = Move::Normal {
            from: Square::new(9, 9).unwrap(),
            to: Square::new(8, 8).unwrap(),
            promote: false,
        };
        let result = display_single_move(&pos, mv);
        assert_eq!(result, Some("▲８８竜左".to_string()));
        let mv = Move::Normal {
            from: Square::new(8, 9).unwrap(),
            to: Square::new(8, 8).unwrap(),
            promote: false,
        };
        let result = display_single_move(&pos, mv);
        assert_eq!(result, Some("▲８８竜右".to_string()));

        // E
        let pos = PartialPosition::from_usi("sfen 9/9/9/9/9/9/9/7+R1/2k1K3+R b - 1").unwrap();
        let mv = Move::Normal {
            from: Square::new(2, 8).unwrap(),
            to: Square::new(1, 7).unwrap(),
            promote: false,
        };
        let result = display_single_move(&pos, mv);
        assert_eq!(result, Some("▲１７竜左".to_string()));
        let mv = Move::Normal {
            from: Square::new(1, 9).unwrap(),
            to: Square::new(1, 7).unwrap(),
            promote: false,
        };
        let result = display_single_move(&pos, mv);
        assert_eq!(result, Some("▲１７竜右".to_string()));
    }

    #[test]
    fn normal_works_6() {
        // Examples found in https://www.shogi.or.jp/faq/kihuhyouki.html.
        // A
        let pos = PartialPosition::from_usi("sfen +B+B7/9/9/9/9/9/9/9/4K1k2 b - 1").unwrap();
        let mv = Move::Normal {
            from: Square::new(9, 1).unwrap(),
            to: Square::new(8, 2).unwrap(),
            promote: false,
        };
        let result = display_single_move(&pos, mv);
        assert_eq!(result, Some("▲８２馬左".to_string()));
        let mv = Move::Normal {
            from: Square::new(8, 1).unwrap(),
            to: Square::new(8, 2).unwrap(),
            promote: false,
        };
        let result = display_single_move(&pos, mv);
        assert_eq!(result, Some("▲８２馬右".to_string()));

        // B
        let pos = PartialPosition::from_usi("sfen 9/9/3+B5/9/+B8/9/9/9/4K1k2 b - 1").unwrap();
        let mv = Move::Normal {
            from: Square::new(9, 5).unwrap(),
            to: Square::new(8, 5).unwrap(),
            promote: false,
        };
        let result = display_single_move(&pos, mv);
        assert_eq!(result, Some("▲８５馬寄".to_string()));
        let mv = Move::Normal {
            from: Square::new(6, 3).unwrap(),
            to: Square::new(8, 5).unwrap(),
            promote: false,
        };
        let result = display_single_move(&pos, mv);
        assert_eq!(result, Some("▲８５馬引".to_string()));

        // C
        let pos = PartialPosition::from_usi("sfen 8+B/9/9/6+B2/9/9/9/9/4K1k2 b - 1").unwrap();
        let mv = Move::Normal {
            from: Square::new(1, 1).unwrap(),
            to: Square::new(1, 2).unwrap(),
            promote: false,
        };
        let result = display_single_move(&pos, mv);
        assert_eq!(result, Some("▲１２馬引".to_string()));
        let mv = Move::Normal {
            from: Square::new(3, 4).unwrap(),
            to: Square::new(1, 2).unwrap(),
            promote: false,
        };
        let result = display_single_move(&pos, mv);
        assert_eq!(result, Some("▲１２馬上".to_string()));

        // D
        let pos = PartialPosition::from_usi("sfen 9/9/9/9/9/9/9/9/+B3+BK1k1 b - 1").unwrap();
        let mv = Move::Normal {
            from: Square::new(9, 9).unwrap(),
            to: Square::new(7, 7).unwrap(),
            promote: false,
        };
        let result = display_single_move(&pos, mv);
        assert_eq!(result, Some("▲７７馬左".to_string()));
        let mv = Move::Normal {
            from: Square::new(5, 9).unwrap(),
            to: Square::new(7, 7).unwrap(),
            promote: false,
        };
        let result = display_single_move(&pos, mv);
        assert_eq!(result, Some("▲７７馬右".to_string()));

        // E
        let pos = PartialPosition::from_usi("sfen 9/9/9/9/9/9/5+B3/8+B/2k1K4 b - 1").unwrap();
        let mv = Move::Normal {
            from: Square::new(4, 7).unwrap(),
            to: Square::new(2, 9).unwrap(),
            promote: false,
        };
        let result = display_single_move(&pos, mv);
        assert_eq!(result, Some("▲２９馬左".to_string()));
        let mv = Move::Normal {
            from: Square::new(1, 8).unwrap(),
            to: Square::new(2, 9).unwrap(),
            promote: false,
        };
        let result = display_single_move(&pos, mv);
        assert_eq!(result, Some("▲２９馬右".to_string()));
    }

    #[test]
    fn drop_works_0() {
        let pos = PartialPosition::from_usi("sfen 4k4/9/9/9/9/9/9/4G4/4K4 b G 1").unwrap();
        let mv = Move::Drop {
            to: Square::new(4, 8).unwrap(),
            piece: Piece::new(PieceKind::Gold, Color::Black),
        };
        let result = display_single_move(&pos, mv);
        assert_eq!(result, Some("▲４８金打".to_string()));
    }

    #[test]
    fn drop_works_1() {
        let pos = PartialPosition::from_usi("sfen 4k4/9/9/9/9/9/9/9/4K4 b G 1").unwrap();
        let mv = Move::Drop {
            to: Square::new(4, 8).unwrap(),
            piece: Piece::new(PieceKind::Gold, Color::Black),
        };
        let result = display_single_move(&pos, mv);
        assert_eq!(result, Some("▲４８金".to_string()));
    }
}
