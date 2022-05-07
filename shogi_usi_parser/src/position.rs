use shogi_core::{Color, Hand, PartialPosition, Piece, Square};

use crate::{try_with_progress, Error, FromUsi, Result};

/// ```
/// # use shogi_core::{Color, PartialPosition, Position};
/// use shogi_usi_parser::FromUsi;
/// let position = Position::from_usi("sfen lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 101 moves 7g7f").unwrap();
/// assert_eq!(position.ply(), 101 + 1);
///
/// // move count is optional. If omitted 1 is used. Multiple whitespaces between tokens are allowed.
/// let position = Position::from_usi("sfen lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b -  moves  7g7f").unwrap();
/// assert_eq!(position.ply(), 1 + 1);
/// assert_eq!(position.initial_position(), &PartialPosition::startpos());
///
/// // startpos is startpos
/// let position = Position::from_usi("startpos moves 7g7f 3c3d 8h2b+ 3a2b").unwrap();
/// assert_eq!(position.initial_position(), &PartialPosition::startpos());
/// ```
#[cfg(feature = "alloc")]
impl FromUsi for shogi_core::Position {
    fn parse_usi_slice(s: &[u8]) -> Result<(&[u8], Self)> {
        let (s, partial) = crate::bind!(PartialPosition::parse_usi_slice(s));
        let orig = s;
        // handles moves
        let s = match parse_many_whitespaces(s) {
            Ok(s) => s,
            Err(_) => return Ok((s, shogi_core::Position::arbitrary_position(partial))),
        };
        if s.get(..5) != Some(b"moves") {
            return Ok((orig, shogi_core::Position::arbitrary_position(partial)));
        }
        let mut s = &s[5..];
        let mut position = shogi_core::Position::arbitrary_position(partial);
        loop {
            let orig = s;
            // optionally read whitespaces and a move
            let next = match parse_many_whitespaces(s) {
                Ok(next) => next,
                Err(_) => return Ok((s, position)),
            };
            let (next, mv) = match shogi_core::Move::parse_usi_slice(next) {
                Ok((next, mv)) => (next, mv),
                Err(_) => return Ok((orig, position)),
            };
            // Even if the read move does not make sense, the parser will not emit an error.
            let _ = position.make_move(mv);
            s = next;
        }
    }
}

/// ```
/// # use shogi_core::{Color, PartialPosition};
/// use shogi_usi_parser::FromUsi;
/// let position = PartialPosition::from_usi("sfen lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 101").unwrap();
/// assert_eq!(position.ply(), 101);
///
/// // move count is optional. If omitted 1 is used.
/// let position = PartialPosition::from_usi("sfen  lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b -").unwrap();
/// assert_eq!(position.ply(), 1);
/// assert_eq!(position, PartialPosition::startpos());
///
/// // startpos is startpos
/// let position = PartialPosition::from_usi("startpos").unwrap();
/// assert_eq!(position, PartialPosition::startpos());
/// ```
impl FromUsi for PartialPosition {
    fn parse_usi_slice(mut s: &[u8]) -> Result<(&[u8], Self)> {
        use core::cmp::min;

        if s.len() >= 8 {
            let (startpos, rest) = s.split_at(8); // Cannot panic
            if startpos == b"startpos" {
                return Ok((rest, PartialPosition::startpos()));
            }
        }

        let mut position = PartialPosition::empty();

        let orig = s;
        if s.get(..4) != Some(b"sfen") {
            return Err(Error::InvalidInput {
                from: 0,
                to: min(4, s.len()),
                description: "invalid token: `sfen` was expected",
            });
        }
        s = &s[4..];

        let mut s = try_with_progress!(parse_many_whitespaces(s), orig.len() - s.len());

        for i in 0..9 {
            let (slash, row) = try_with_progress!(parse_row(s), orig.len() - s.len());
            for j in 0..9 {
                position.piece_set(
                    // Safety: 1 <= 9 - j <= 9, 1 <= i + 1 <= 9
                    unsafe { Square::new(9 - j, i + 1).unwrap_unchecked() },
                    row[j as usize],
                );
            }
            s = slash;
            if i < 8 {
                if s.get(0).copied() != Some(b'/') {
                    return Err(Error::InvalidInput {
                        from: orig.len() - slash.len(),
                        to: orig.len() - slash.len() + min(s.len(), 1),
                        description: "`/` was expected",
                    });
                }
                // Safety: s.len() >= 1
                s = unsafe { s.get_unchecked(1..) };
            }
        }
        // It is unclear whether multiple whitespaces between the components of SFEN are allowed.
        // Here we assume they aren't.
        if s.get(0).copied() != Some(b' ') {
            return Err(Error::InvalidInput {
                from: orig.len() - s.len(),
                to: orig.len() - s.len() + min(s.len(), 1),
                description: "` ` (whitespace) was expected",
            });
        }
        // Safety: s.len() >= 1
        let s = unsafe { s.get_unchecked(1..) };
        let (s, side) = try_with_progress!(Color::parse_usi_slice(s), orig.len() - s.len());
        position.side_to_move_set(side);
        if s.get(0).copied() != Some(b' ') {
            return Err(Error::InvalidInput {
                from: orig.len() - s.len(),
                to: orig.len() - s.len() + 1,
                description: "` ` (whitespace) was expected",
            });
        }
        // Safety: s.len() >= 1
        let s = unsafe { s.get_unchecked(1..) };
        let (s, hand) = try_with_progress!(<[Hand; 2]>::parse_usi_slice(s), orig.len() - s.len());
        *position.hand_of_a_player_mut(Color::Black) = hand[0];
        *position.hand_of_a_player_mut(Color::White) = hand[1];
        // optional move count
        if s.get(0).copied() != Some(b' ') {
            return Ok((s, position));
        }
        // Safety: s.len() >= 1
        let mut s = unsafe { s.get_unchecked(1..) };
        let mut count: u16 = 0;
        while matches!(s.get(0).copied(), Some(b'0'..=b'9')) {
            // Safety: s.len() >= 1
            let digit = (*unsafe { s.get_unchecked(0) } - b'0') as u16;
            count = count.saturating_mul(10).saturating_add(digit);
            // Safety: s.len() >= 1
            s = unsafe { s.get_unchecked(1..) };
        }
        // We can ignore the result because even if setting move count fails, there is no problem.
        let _ = position.ply_set(count);
        Ok((s, position))
    }
}

// Skips /\s+/.
fn parse_many_whitespaces(s: &[u8]) -> Result<&[u8]> {
    use core::cmp::min;

    let mut s = if let Some((&b' ', s)) = s.split_first() {
        s
    } else {
        return Err(Error::InvalidInput {
            from: 0,
            to: min(1, s.len()),
            description: "` ` (whitespace) was expected",
        });
    };
    while let Some((&b' ', rest)) = s.split_first() {
        s = rest;
    }
    Ok(s)
}

fn parse_row(s: &[u8]) -> Result<(&[u8], [Option<Piece>; 9])> {
    let mut this_row = s;
    let mut seen = 0; // how many squares did we find?
    let mut result = [None; 9];
    while let Some((&first, next)) = this_row.split_first() {
        if !(seen <= 90 && first != b'/' && first != b' ') {
            break;
        }
        if matches!(first, b'1'..=b'9') {
            seen += first - b'0';
            this_row = next;
            continue;
        }
        let (next, piece) =
            try_with_progress!(Piece::parse_usi_slice(this_row), s.len() - this_row.len());
        if seen < 9 {
            result[seen as usize] = Some(piece);
        }
        seen += 1;
        this_row = next;
    }
    if seen != 9 {
        return Err(Error::InvalidInput {
            from: 0,
            to: s.len() - this_row.len(),
            description: "exactly 9 squares are expected",
        });
    }
    Ok((this_row, result))
}

/// C interface of `Position::parse_usi_slice`.
/// If parse error occurs, it returns -1.
/// If parsing succeeds, it returns the number of read bytes.
///
/// # Safety
/// `position` must be a valid pointer to a PartialPosition.
/// `s` must be a nul-terminated C string.
#[no_mangle]
#[cfg(feature = "alloc")]
pub unsafe extern "C" fn Position_parse_usi_slice(
    position: &mut shogi_core::Position,
    s: *const u8,
) -> isize {
    crate::common::make_parse_usi_slice_c(position, s)
}

/// C interface of `PartialPosition::parse_usi_slice`.
/// If parse error occurs, it returns -1.
/// If parsing succeeds, it returns the number of read bytes.
///
/// # Safety
/// `position` must be a valid pointer to a PartialPosition.
/// `s` must be a nul-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn PartialPosition_parse_usi_slice(
    position: &mut PartialPosition,
    s: *const u8,
) -> isize {
    crate::common::make_parse_usi_slice_c(position, s)
}
