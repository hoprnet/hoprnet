/// Used to flip the most significant bit of 32-bit values
const BITMASK_32: u32 = 0x80000000;
/// Used to flip the most significant bit of 64-bit values
const BITMASK_64: u64 = 0x8000000000000000;

/// Maps [u64::MIN, u64::MAX] to [i64::MIN, i64::MAX]
#[inline]
pub fn u64_to_i64(ticket_index: u64) -> i64 {
    i64::from_be_bytes((ticket_index ^ BITMASK_64).to_be_bytes())
}

#[inline]
pub fn i64_to_u64(db_index: i64) -> u64 {
    u64::from_be_bytes(db_index.to_be_bytes()) ^ BITMASK_64
}

#[inline]
pub fn u32_to_i32(ticket_index: u32) -> i32 {
    i32::from_be_bytes((ticket_index ^ BITMASK_32).to_be_bytes())
}

#[inline]
pub fn i32_to_u32(db_index: i32) -> u32 {
    u32::from_be_bytes(db_index.to_be_bytes()) ^ BITMASK_32
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_index_conversion_64() {
        assert_eq!(super::u64_to_i64(u64::MIN), i64::MIN);
        assert_eq!(super::u64_to_i64((1u64 << 63) - 1), -1);
        assert_eq!(super::u64_to_i64(1u64 << 63), 0);
        assert_eq!(super::u64_to_i64((1u64 << 63) + 1), 1);
        assert_eq!(super::u64_to_i64(u64::MAX), i64::MAX);

        assert_eq!(super::i64_to_u64(i64::MIN), u64::MIN);
        assert_eq!(super::i64_to_u64(-1), (1u64 << 63) - 1);
        assert_eq!(super::i64_to_u64(0), 1u64 << 63);
        assert_eq!(super::i64_to_u64(1), (1u64 << 63) + 1);
        assert_eq!(super::i64_to_u64(i64::MAX), u64::MAX)
    }

    #[test]
    fn test_index_conversion_32() {
        assert_eq!(super::u32_to_i32(u32::MIN), i32::MIN);
        assert_eq!(super::u32_to_i32((1u32 << 31) - 1), -1);
        assert_eq!(super::u32_to_i32(1u32 << 31), 0);
        assert_eq!(super::u32_to_i32((1u32 << 31) + 1), 1);
        assert_eq!(super::u32_to_i32(u32::MAX), i32::MAX);

        assert_eq!(super::i32_to_u32(i32::MIN), u32::MIN);
        assert_eq!(super::i32_to_u32(-1), (1u32 << 31) - 1);
        assert_eq!(super::i32_to_u32(0), 1u32 << 31);
        assert_eq!(super::i32_to_u32(1), (1u32 << 31) + 1);
        assert_eq!(super::i32_to_u32(i32::MAX), u32::MAX)
    }
}
