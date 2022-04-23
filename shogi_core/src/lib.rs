#![cfg_attr(not(feature = "std"), no_std)] // Forbids using std::*.

mod color;
mod piece;
mod piece_kind;
mod square;

#[doc(inline)]
pub use crate::color::Color;

#[doc(inline)]
pub use crate::square::Square;

#[doc(inline)]
pub use crate::piece_kind::PieceKind;

#[doc(inline)]
pub use crate::piece::Piece;

#[cfg(not(feature = "std"))]
#[panic_handler]
fn panic(_panic: &core::panic::PanicInfo<'_>) -> ! {
    loop {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discriminant_elision_works() {
        use core::mem::size_of;

        assert_eq!(size_of::<Option<Color>>(), size_of::<Color>());
        assert_eq!(size_of::<Option<Square>>(), size_of::<Square>());
        assert_eq!(size_of::<Option<PieceKind>>(), size_of::<PieceKind>());
        assert_eq!(size_of::<Option<Piece>>(), size_of::<Piece>());
    }
}
