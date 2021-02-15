use hex;

// Warning: i wrote this before i knew i could do 0xAA and such for hex
// numbers. I'm using strings and coverting to numbers. I'll fix it later.

// Master start bytes
const MA0: u8 = 0x55;
const MA1: u8 = 0xAA;
// Master end byte
const MAE: u8 = 0x77;


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


    fn checksum_as_hex(&self) -> u8 {
        let sum = self.data.iter().map(|&val| val as i32 ).sum::<i32>();
        return (sum % 0x100) as u8;
    }

    /// Returns a String of all bytes as hex, padded to 2 spaces
    pub fn full(&self) -> String {
        let data_strings = self.data.iter().map(|&val| format!("{:0>2}", format!("{:x}", val)) ).collect::<Vec<String>>();
        format!("{:0>2x}{:0>2x}{}{:0>2x}{:0>2x}", MA0, MA1, data_strings.join(""), self.checksum_as_hex(), MAE)
    }

    pub fn to_bytes(self) -> Vec<u8> {
        let mut bytes: Vec<u8> = vec![MA0, MA1];
        for byte in &self.data {
            bytes.push(*byte);
        }
        bytes.push(self.checksum_as_hex());
        bytes.push(MAE);
        return bytes;
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
    fn full_bytestring() {
        assert_eq!("55aafeff01030177", Bytestring::from(vec![254, 255, 1, 3]).full());
        assert_eq!("55aafefe77", Bytestring::from(vec![254]).full());
        assert_eq!("55aa010177", Bytestring::from(vec![1]).full());
        assert_eq!("55aa0077", Bytestring::from(vec![]).full());
    }

    #[test]
    fn checksum_as_hex() {
        let bs = Bytestring::from(vec![5, 5, 10]);
        assert_eq!(0x14, bs.checksum_as_hex());
    }
}
