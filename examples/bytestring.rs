use brewdrivers::relays::Bytestring;

fn main() {
    let bs = Bytestring::from(vec![0xF3, 0xF3]);

    // 0xF3 + 0xF3 = 0x01E6
    assert_eq!(bs.checksum_as_hex(), 0xE6);
    println!("Full bytestring: {}", bs);
    assert_eq!(bs.to_bytes(), vec![0x55, 0xAA, 0xF3, 0xF3, 0xE6, 0x77]);
}
