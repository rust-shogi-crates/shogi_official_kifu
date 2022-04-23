use core::num::NonZeroU8;

/// A square.
///
/// `Square` and `Option<Square>` are both 1-byte data types.
/// Because they are cheap to copy, they implement [`Copy`](https://doc.rust-lang.org/core/marker/trait.Copy.html).
#[repr(transparent)]
#[derive(Eq, PartialEq, Clone, Copy, Debug)]
#[cfg_attr(feature = "ord", derive(PartialOrd, Ord))]
#[cfg_attr(feature = "hash", derive(Hash))]
pub struct Square(NonZeroU8);

impl Square {
    /// Creates a new `Square` with given `file` and `rank`.
    ///
    /// `file` and `rank` must be between 1 and 9 (both inclusive).
    /// If this condition is not met, this function returns None.
    #[export_name = "Square_new"]
    pub extern "C" fn new(file: u8, rank: u8) -> Option<Self> {
        if file.wrapping_sub(1) >= 9 || rank.wrapping_sub(1) >= 9 {
            return None;
        }
        // Safety: file >= 1 && rank >= 1 implies file * 9 + rank - 9 >= 1
        Some(Square(unsafe {
            NonZeroU8::new_unchecked(file * 9 + rank - 9)
        }))
    }
    #[export_name = "Square_file"]
    pub extern "C" fn file(self) -> u8 {
        (self.0.get() + 8) / 9
    }
    #[export_name = "Square_rank"]
    pub extern "C" fn rank(self) -> u8 {
        ((self.0.get() - 1) % 9) + 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_works() {
        for file in 0..256 {
            for rank in 0..256 {
                let file = file as u8;
                let rank = rank as u8;
                let result = Square::new(file, rank);
                assert_eq!(
                    result.is_some(),
                    (1..=9).contains(&file) && (1..=9).contains(&rank),
                );
                if let Some(sq) = result {
                    assert_eq!(sq.file(), file);
                    assert_eq!(sq.rank(), rank);
                }
            }
        }
    }
}
