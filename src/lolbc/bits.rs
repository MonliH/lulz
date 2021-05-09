pub fn usize_from_u8(hi: u8, mi: u8, lo: u8) -> usize {
    ((hi as usize) << 16) + ((mi as usize) << 8) + (lo as usize)
}

#[test]
fn test_usize_from_u8() {
    assert_eq!(
        usize_from_u8(0b10000000, 0b00010101, 0b11111111),
        0b100000000001010111111111
    );
}
