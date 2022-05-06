use core::slice;
use shogi_core::{Color, Hand, PartialPosition, Piece, Square};

use crate::{try_with_progress, Error, FromUsi, Result};

/// ```
/// # use shogi_core::{Color, PartialPosition, Piece, PieceKind};
/// use shogi_usi_parser::FromUsi;
/// let position = PartialPosition::from_usi("lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 101").unwrap();
/// assert_eq!(position.ply(), 101);
///
/// // move count is optional. If omitted 1 is used.
/// let position = PartialPosition::from_usi("lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b -").unwrap();
/// assert_eq!(position.ply(), 1);
/// assert_eq!(position, PartialPosition::startpos());
/// ```
impl FromUsi for PartialPosition {
    fn parse_usi_slice(mut s: &[u8]) -> Result<(&[u8], Self)> {
        let mut position = PartialPosition::startpos();
        let orig = s;
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
                if s[0] != b'/' {
                    return Err(Error::InvalidInput {
                        from: orig.len() - slash.len(),
                        to: orig.len() - slash.len() + 1,
                        description: "`/` was expected",
                    });
                }
                s = &s[1..];
            }
        }
        // It is unclear whether multiple whitespaces between the components of SFEN are allowed.
        // Here we assume they aren't.
        if s[0] != b' ' {
            return Err(Error::InvalidInput {
                from: orig.len() - s.len(),
                to: orig.len() - s.len() + 1,
                description: "` ` (whitespace) was expected",
            });
        }
        let s = &s[1..];
        let (s, side) = try_with_progress!(Color::parse_usi_slice(s), orig.len() - s.len());
        position.side_to_move_set(side);
        if s[0] != b' ' {
            return Err(Error::InvalidInput {
                from: orig.len() - s.len(),
                to: orig.len() - s.len() + 1,
                description: "` ` (whitespace) was expected",
            });
        }
        let s = &s[1..];
        let (s, hand) = try_with_progress!(<[Hand; 2]>::parse_usi_slice(s), orig.len() - s.len());
        *position.hand_of_a_player_mut(Color::Black) = hand[0];
        *position.hand_of_a_player_mut(Color::White) = hand[1];
        // optional move count
        if s.get(0).copied() != Some(b' ') {
            return Ok((s, position));
        }
        let mut s = &s[1..];
        let mut count: u16 = 0;
        while !s.is_empty() && matches!(s[0], b'0'..=b'9') {
            let digit = (s[0] - b'0') as u16;
            count = count.saturating_mul(10).saturating_add(digit);
            s = &s[1..];
        }
        // We can ignore the result because even if setting move count fails, there is no problem.
        let _ = position.ply_set(count);
        Ok((s, position))
    }
}

fn parse_row(s: &[u8]) -> Result<(&[u8], [Option<Piece>; 9])> {
    let mut this_row = s;
    let mut seen = 0; // how many squares did we find?
    let mut result = [None; 9];
    while !this_row.is_empty() && seen <= 90 && this_row[0] != b'/' && this_row[0] != b' ' {
        if matches!(this_row[0], b'1'..=b'9') {
            seen += this_row[0] - b'0';
            this_row = &this_row[1..];
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
    let mut length = 0;
    while *s.add(length) != 0 {
        length += 1;
    }
    let slice = slice::from_raw_parts(s, length);
    match PartialPosition::parse_usi_slice(slice) {
        Ok((slice, resulting_position)) => {
            *position = resulting_position;
            slice.as_ptr().offset_from(s)
        }
        Err(_) => -1,
    }
}
