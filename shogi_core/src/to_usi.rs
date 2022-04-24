use core::fmt::{Result as FmtResult, Write};

pub trait ToUsi {
    /// Write `self` in USI format.
    fn to_usi<W: Write>(&self, sink: &mut W) -> FmtResult;

    #[cfg(feature = "alloc")]
    fn to_usi_owned(&self) -> alloc::string::String {
        let mut s = alloc::string::String::new();
        self.to_usi(&mut s).unwrap();
        s
    }
}
