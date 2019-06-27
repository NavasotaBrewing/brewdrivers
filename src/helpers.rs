// // Like zfill in python :)
// pub fn zfill(val: u8) -> String {
//     format!("{:02}", val)
// }

pub fn zfill_string(val: String) -> String {
    format!("{:02}", val)
}

/// Returns a 2 digit hex
///
/// Only accepts u8, 0-255
pub fn to_hex(val: u8) -> String {
    let hex = format!("{:x}", val);
    if hex.len() == 1 {
        return format!("0{}", hex);
    }
    hex
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_hex() {
        assert_eq!("05", to_hex(5));
        assert_eq!("0e", to_hex(14));
        assert_eq!("91", to_hex(145));
    }
}
