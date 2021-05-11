pub enum Bits {
    U24(u8, u8, u8),
    U8(u8),
}

impl From<usize> for Bits {
    fn from(v: usize) -> Self {
        if v <= (u8::MAX as usize) {
            Bits::U8(v as u8)
        } else {
            Bits::U24((v >> 16) as u8, (v >> 8) as u8, v as u8)
        }
    }
}

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
