// Like zfill in python :)
pub fn zfill(val: u8) -> String {
    format!("{:02}", val)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zfill() {
        assert_eq!("05",  zfill(5));
        assert_eq!("14",  zfill(14));
        assert_eq!("145", zfill(145));
    }
}
