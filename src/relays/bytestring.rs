use hex;

// Master start bytes
const MA0: &str = "55";
const MA1: &str = "aa";
// Master end byte
const MAE: &str = "77";


#[derive(Debug)]
pub struct Bytestring {
    pub data: Vec<u8>,
}

impl Bytestring {
    /// Returns a new Bytestring from a Vec<u8> of data bytes (u8)
    pub fn from(bytes: Vec<u8>) -> Bytestring {
        Bytestring {
            data: bytes
        }
    }


    /// Returns a Bytestring from a vector of hex bytes
    /// Each hex byte should be exactly 2 characters
    /// ```rust
    /// use brewdrivers::relays::Bytestring;
    ///
    /// let b = Bytestring::from_hex(vec!["fe", "ff", "01", "01", "45"]);
    /// assert_eq!("55aafeff0101454477".to_owned(), b.full());
    /// ```
    pub fn from_hex(hexes: Vec<&str>) -> Bytestring {
        let bytes = hexes.iter().map(|val| Bytestring::to_u8(val).expect("Invalid hex pair") ).collect::<Vec<u8>>();
        Bytestring::from(bytes)
    }

    /// Converts a u8 to a 2-character hex String
    /// ```rust
    /// use brewdrivers::relays::Bytestring;
    ///
    /// assert_eq!("ff".to_owned(), Bytestring::to_hex(255));
    /// assert_eq!("01".to_owned(), Bytestring::to_hex(1));
    /// assert_eq!("16".to_owned(), Bytestring::to_hex(22));
    /// ```
    pub fn to_hex(val: u8) -> String {
        let hex = format!("{:x}", val);
        if hex.len() == 1 {
            return format!("0{}", hex);
        }
        hex
    }

    /// Converts a 2 character hex String to a u8
    ///
    /// Inverse of `to_hex`
    /// ```rust
    /// use brewdrivers::relays::Bytestring;
    ///
    /// assert_eq!(Some(1), Bytestring::to_u8("01"));
    /// assert_eq!(Some(22), Bytestring::to_u8("16"));
    /// assert_eq!(Some(255), Bytestring::to_u8("ff"));
    /// ```
    pub fn to_u8(hex: &str) -> Option<u8> {
        if hex.len() > 2 {
            return None;
        }

        match hex::decode(hex) {
            Ok(val) => Some(val[0]),
            Err(_) => None,
        }
    }

    fn checksum_as_hex(&self) -> String {
        let sum = self.data.iter().map(|&val| val as i32 ).sum::<i32>();
        let hex_string = format!("{:x}", sum);
        if hex_string.len() == 1 {
            format!("0{}", hex_string)
        } else {
            hex_string[hex_string.len() - 2..].to_string()
        }
    }

    /// Returns a String of all bytes as hex
    pub fn full(&self) -> String {
        let data_strings = self.data.iter().map(|&val| Bytestring::to_hex(val) ).collect::<Vec<String>>();
        format!("{}{}{}{}{}", MA0, MA1, data_strings.join(""), self.checksum_as_hex(), MAE)
    }

    /// Decodes all hex bytes to a Vec<u8>
    pub fn to_bytes(&self) -> Vec<u8> {
        hex::decode(&self.full()).unwrap_or(vec![])
    }
}

impl std::fmt::Display for Bytestring {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.full())
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bytestring_hex_to_u8() {
        assert_eq!(Some(254), Bytestring::to_u8("fe"));
        assert_eq!(Some(0),   Bytestring::to_u8("00"));
        assert_eq!(Some(16),  Bytestring::to_u8("10"));
        assert_eq!(None,      Bytestring::to_u8("0"));
        assert_eq!(None,      Bytestring::to_u8("fefe"));
        assert_eq!(None,      Bytestring::to_u8("covfefe"));
    }

    #[test]
    fn full_bytestring() {
        assert_eq!("55aafeff01030177", Bytestring::from(vec![254, 255, 1, 3]).full());
        assert_eq!("55aafefe77", Bytestring::from(vec![254]).full());
        assert_eq!("55aa010177", Bytestring::from(vec![1]).full());
        assert_eq!("55aa0077", Bytestring::from(vec![]).full());
    }

    #[test]
    fn checksum_as_hex() {
        let bs = Bytestring::from(vec![5, 5, 10]);
        assert_eq!("14", bs.checksum_as_hex());
    }

    #[test]
    fn from_hex() {
        assert_eq!("55aafeff01030177", Bytestring::from_hex(vec!["fe", "ff", "01", "03"]).full());
        assert_eq!("55aa0077", Bytestring::from_hex(vec![]).full());
    }

    #[test]
    #[should_panic]
    fn from_hex_with_errors() {
        // Invalid hex pairs
        assert_eq!("55aafeff01030177", Bytestring::from_hex(vec!["fe", "ff", "1", "3"]).full());
    }
}
