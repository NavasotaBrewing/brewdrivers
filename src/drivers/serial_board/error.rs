use std::fmt;
use std::error::Error;
use std::str::FromStr;

/// A generic board error. This is used when communication with any board is unsuccessful.
#[derive(Debug)]
pub struct BoardError {
    pub msg: String,
    pub address: Option<u8>
}

impl fmt::Display for BoardError {
    /// Displays the given message of a board error, including the board address
    /// if there is one provided with the error
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(addr) = self.address {
            write!(f, "Board 0x{:X?}: {}", addr, self.msg)
        } else {
            write!(f, "{}", self.msg)
        }
    }
}

impl Error for BoardError {}

impl FromStr for BoardError {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(BoardError {
                msg: String::from(s),
                address: None
            })
    }
}

