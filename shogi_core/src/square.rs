use core::num::NonZeroU8;

use crate::Color;

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

    /// Finds the file in range `1..=9`.
    ///
    /// Examples:
    /// ```
    /// use shogi_core::Square;
    /// assert_eq!(Square::new(3, 4).unwrap().file(), 3);
    /// ```

    #[export_name = "Square_file"]
    pub extern "C" fn file(self) -> u8 {
        (self.0.get() + 8) / 9
    }

    /// Finds the rank in range `1..=9`.
    ///
    /// Examples:
    /// ```
    /// use shogi_core::Square;
    /// assert_eq!(Square::new(3, 4).unwrap().rank(), 4);
    /// ```
    #[export_name = "Square_rank"]
    pub extern "C" fn rank(self) -> u8 {
        ((self.0.get() - 1) % 9) + 1
    }

    /// Finds the index of `self` in range `1..=81`.
    /// It is guaranteed that the result is equal to the internal representation, 9 * file + rank - 9.
    ///
    /// Examples:
    /// ```
    /// use shogi_core::Square;
    /// assert_eq!(Square::new(3, 4).unwrap().index(), 22);
    /// ```
    #[inline(always)]
    #[export_name = "Square_index"]
    pub extern "C" fn index(self) -> u8 {
        self.0.get()
    }

    /// Finds the rank from the perspective of `color`.
    #[export_name = "Square_relative_rank"]
    pub extern "C" fn relative_rank(self, color: Color) -> u8 {
        let rank = self.rank();
        match color {
            Color::Black => rank,
            Color::White => 10 - rank,
        }
    }

    /// Finds the file from the perspective of `color`.
    #[export_name = "Square_relative_file"]
    pub extern "C" fn relative_file(self, color: Color) -> u8 {
        let file = self.file();
        match color {
            Color::Black => file,
            Color::White => 10 - file,
        }
    }

    /// Finds the reflected square of `self`.
    ///
    /// Examples:
    /// ```
    /// use shogi_core::Square;
    /// assert_eq!(Square::new(1, 1).unwrap().flip(), Square::new(9, 9).unwrap());
    /// assert_eq!(Square::new(3, 4).unwrap().flip(), Square::new(7, 6).unwrap());
    /// ```
    #[export_name = "Square_flip"]
    pub extern "C" fn flip(self) -> Self {
        // Safety: self.0.get() is in range 1..=81.
        unsafe { Self::from_u8(82 - self.0.get()) }
    }

    /// Convert u8 to `Square`.
    ///
    ///  # Safety
    /// `value` must be in range 1..=81
    #[inline(always)]
    pub unsafe fn from_u8(value: u8) -> Self {
        Self(NonZeroU8::new_unchecked(value))
    }

    /// Returns an iterator that iterates over all possible `Square`s
    /// in the ascending order of their indices.
    pub fn all() -> impl core::iter::Iterator<Item = Self> {
        (1..=81).map(|index| unsafe { Self::from_u8(index) })
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
                    assert_eq!(sq.relative_file(Color::Black), file);
                    assert_eq!(sq.relative_rank(Color::Black), rank);
                    assert_eq!(sq.relative_file(Color::White), 10 - file);
                    assert_eq!(sq.relative_rank(Color::White), 10 - rank);
                }
            }
        }
    }

    #[test]
    fn flip_works() {
        for file in 1..=9 {
            for rank in 1..=9 {
                let sq = Square::new(file, rank).unwrap();
                assert_eq!(sq.flip(), Square::new(10 - file, 10 - rank).unwrap());
            }
        }
    }
}
