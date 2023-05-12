// ZigZag endoging used for efficient transfer of signed integers
// https://developers.google.com/protocol-buffers/docs/encoding#types

pub fn decode_zig_zag_32(n: u32) -> i32 {
    ((n >> 1) as i32) ^ (-((n & 1) as i32))
}

pub fn decode_zig_zag_64(n: u64) -> i64 {
    ((n >> 1) as i64) ^ (-((n & 1) as i64))
}

pub fn encode_zig_zag_32(n: i32) -> u32 {
    ((n << 1) ^ (n >> 31)) as u32
}

pub fn encode_zig_zag_64(n: i64) -> u64 {
    ((n << 1) ^ (n >> 63)) as u64
}

#[cfg(test)]
mod test {

    use super::decode_zig_zag_32;
    use super::decode_zig_zag_64;
    use super::encode_zig_zag_32;
    use super::encode_zig_zag_64;

    #[test]
    fn test_zig_zag() {
        fn test_zig_zag_pair_64(decoded: i64, encoded: u64) {
            assert_eq!(decoded, decode_zig_zag_64(encoded));
            assert_eq!(encoded, encode_zig_zag_64(decoded));
        }

        fn test_zig_zag_pair(decoded: i32, encoded: u32) {
            assert_eq!(decoded, decode_zig_zag_32(encoded));
            assert_eq!(encoded, encode_zig_zag_32(decoded));
            test_zig_zag_pair_64(decoded as i64, encoded as u64);
        }

        test_zig_zag_pair(0, 0);
        test_zig_zag_pair(-1, 1);
        test_zig_zag_pair(1, 2);
        test_zig_zag_pair(-2, 3);
        test_zig_zag_pair(2147483647, 4294967294);
        test_zig_zag_pair(-2147483648, 4294967295);
        test_zig_zag_pair_64(9223372036854775807, 18446744073709551614);
        test_zig_zag_pair_64(-9223372036854775808, 18446744073709551615);
    }
}
