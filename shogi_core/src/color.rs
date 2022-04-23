/// A player.
///
/// `Color` and `Option<Color>` are both 1-byte data types.
/// Because they are cheap to copy, they implement [`Copy`](https://doc.rust-lang.org/core/marker/trait.Copy.html).
#[repr(u8)]
#[derive(Eq, PartialEq, Clone, Copy, Debug)]
#[cfg_attr(feature = "ord", derive(PartialOrd, Ord))]
#[cfg_attr(feature = "hash", derive(Hash))]
pub enum Color {
    /// Black, who plays first. Known as `先手` (*sente*).
    Black = 1,
    /// White, who plays second. Known as `後手` (*gote*).
    White = 2,
}

impl Color {
    /// Flips the player.
    ///
    /// Examples:
    /// ```
    /// use shogi_core::Color;
    /// assert_eq!(Color::Black.flip(), Color::White);
    /// assert_eq!(Color::White.flip(), Color::Black);
    /// assert_eq!(core::mem::size_of::<Option<Color>>(), 1);
    /// ```
    #[export_name = "Color_flip"]
    pub extern "C" fn flip(self) -> Self {
        unsafe { core::mem::transmute(self as u8 ^ 3) }
    }
    #[cfg(test)]
    pub(crate) fn all() -> [Self; 2] {
        [Color::Black, Color::White]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flip_works() {
        let colors = Color::all();
        assert_eq!(colors[0].flip(), colors[1]);
        assert_eq!(colors[1].flip(), colors[0]);
    }
}
