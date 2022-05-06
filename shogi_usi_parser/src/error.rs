/// An error that can occur while parsing.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// The parser failed to parse `s[from..to]`.
    InvalidInput {
        from: usize,
        to: usize,
        description: &'static str,
    },
    /// After parsing the input, `s[from..]` was left unread.
    Extra { from: usize },
    /// Parsing was successful, but we got an invalid position.
    InvalidPosition,
}

pub type Result<T> = core::result::Result<T, Error>;
