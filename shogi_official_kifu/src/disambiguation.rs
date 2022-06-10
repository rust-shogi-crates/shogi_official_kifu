use shogi_core::{Bitboard, Color, PartialPosition, PieceKind, Square};

use core::fmt::Write;
use std::cmp::Ordering;

pub fn run<W: Write>(
    position: &PartialPosition,
    from: Square,
    to: Square,
    candidates: Bitboard,
    w: &mut W,
) -> Result<Option<()>, std::fmt::Error> {
    if candidates.is_empty() {
        return Ok(None);
    }
    // Needs nothing
    if candidates.count() == 1 {
        return Ok(Some(()));
    }
    let (subset2, char2) = if let Some(result) = run_move(position, from, to, candidates) {
        result
    } else {
        return Ok(None);
    };
    let (subset1, char1) = if let Some(result) = run_file(position, from, to, candidates) {
        result
    } else {
        return Ok(None);
    };
    // Preference: nothing > 2 > 1 > 1 + 2
    if subset2.count() == 1 {
        w.write_char(char2)?;
        return Ok(Some(()));
    }
    if subset1.count() == 1 {
        w.write_char(char1)?;
        return Ok(Some(()));
    }
    if (subset1 & subset2).count() == 1 {
        w.write_char(char1)?;
        w.write_char(char2)?;
        return Ok(Some(()));
    }
    Ok(None)
}

fn run_move(
    position: &PartialPosition,
    from: Square,
    to: Square,
    candidates: Bitboard,
) -> Option<(Bitboard, char)> {
    let side = position.side_to_move();
    let delta = (from.relative_rank(side) as i8 - to.relative_rank(side) as i8).signum();
    let mut new_candidates = Bitboard::empty();
    for c_from in candidates {
        let c_delta = (c_from.relative_rank(side) as i8 - to.relative_rank(side) as i8).signum();
        if c_delta == delta {
            new_candidates |= c_from;
        }
    }
    if new_candidates.is_empty() {
        return None;
    }
    let vertical = match delta.cmp(&0) {
        Ordering::Greater => '上', // goes up
        Ordering::Less => '引',    // pull back
        Ordering::Equal => '寄',
    };

    Some((new_candidates, vertical))
}

fn run_file(
    position: &PartialPosition,
    from: Square,
    to: Square,
    candidates: Bitboard,
) -> Option<(Bitboard, char)> {
    let side = position.side_to_move();
    let piece_kind = position.piece_at(from)?.piece_kind();
    if is_gold_like(piece_kind) {
        // Use |from.file() - to.file()| to disambiguate.
        let file_diff = from.file() as i8 - to.file() as i8;
        if file_diff == 0 && from.relative_rank(side) as i8 - to.relative_rank(side) as i8 > 0 {
            // We should use '直' for this particular case.
            return Some((Bitboard::single(from), '直'));
        }
        let file_diff_relative = file_diff * if side == Color::Black { 1 } else { -1 };
        let horizontal = match file_diff_relative.cmp(&0) {
            Ordering::Less => '右',
            Ordering::Greater => '左',
            Ordering::Equal => '縦',
        };
        let mut new_candidates = Bitboard::empty();
        for c_from in candidates {
            let c_file_diff = c_from.file() as i8 - to.file() as i8;
            if c_file_diff == file_diff {
                new_candidates |= c_from;
            }
        }
        return Some((new_candidates, horizontal));
    }
    // Use relative file difference between two candidates to disambiguate.
    // It is guaranteed that |candidates| <= 2.
    if candidates.count() != 2 {
        return Some((candidates, '壱'));
    }
    let mut candidates_cp = candidates;
    // TODO stop panicking
    let cand1 = candidates_cp.pop().unwrap();
    let cand2 = candidates_cp.pop().unwrap();
    if cand1.file() == cand2.file() {
        return Some((candidates, '？'));
    }
    let mut cand = [cand1, cand2];
    cand.sort_unstable_by_key(|&c| c.file() as i8 * if side == Color::Black { 1 } else { -1 });
    let relative_file = if from == cand[0] {
        '右'
    } else if from == cand[1] {
        '左'
    } else {
        return Some((Bitboard::empty(), '無'));
    };
    Some((Bitboard::single(from), relative_file))
}

fn is_gold_like(piece_kind: PieceKind) -> bool {
    use PieceKind::*;
    matches!(
        piece_kind,
        Gold | Silver | ProPawn | ProLance | ProKnight | ProSilver,
    )
}
