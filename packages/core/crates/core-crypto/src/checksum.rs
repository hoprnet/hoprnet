use sha3::{digest::DynDigest, Keccak256};
use utils_types::primitives::Address;

// https://eips.ethereum.org/EIPS/eip-55
pub fn to_checksum(address: Address) -> String {
    let address_hex = address._to_hex();
    let mut hasher = Keccak256::default();
    hasher.update(address_hex.as_bytes());
    let hash = hasher.finalize_reset();

    let mut ret = String::from("0x");

    for (i, c) in address._to_hex().chars().enumerate() {
        let nibble = hash[i / 2] >> (((i + 1) % 2) * 4) & 0xf;
        if nibble >= 8 {
            ret.push(c.to_ascii_uppercase());
        } else {
            ret.push(c);
        }
    }
    ret
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn address_to_checksum_test_all_caps() {
        let addr_1 = Address::from_str("52908400098527886e0f7030069857d2e4169ee7").unwrap();
        let value_1 = to_checksum(addr_1);
        let addr_2 = Address::from_str("8617e340b3d01fa5f11f306f4090fd50e238070d").unwrap();
        let value_2 = to_checksum(addr_2);

        assert_eq!(
            value_1, "0x52908400098527886E0F7030069857D2E4169EE7",
            "checksumed address does not match"
        );
        assert_eq!(
            value_2, "0x8617E340B3D01FA5F11F306F4090FD50E238070D",
            "checksumed address does not match"
        );
    }

    #[test]
    fn address_to_checksum_test_all_lower() {
        let addr_1 = Address::from_str("de709f2102306220921060314715629080e2fb77").unwrap();
        let value_1 = to_checksum(addr_1);
        let addr_2 = Address::from_str("27b1fdb04752bbc536007a920d24acb045561c26").unwrap();
        let value_2 = to_checksum(addr_2);

        assert_eq!(
            value_1, "0xde709f2102306220921060314715629080e2fb77",
            "checksumed address does not match"
        );
        assert_eq!(
            value_2, "0x27b1fdb04752bbc536007a920d24acb045561c26",
            "checksumed address does not match"
        );
    }

    #[test]
    fn address_to_checksum_test_all_normal() {
        let addr_1 = Address::from_str("5aaeb6053f3e94c9b9a09f33669435e7ef1beaed").unwrap();
        let value_1 = to_checksum(addr_1);
        let addr_2 = Address::from_str("fb6916095ca1df60bb79ce92ce3ea74c37c5d359").unwrap();
        let value_2 = to_checksum(addr_2);
        let addr_3 = Address::from_str("dbf03b407c01e7cd3cbea99509d93f8dddc8c6fb").unwrap();
        let value_3 = to_checksum(addr_3);
        let addr_4 = Address::from_str("d1220a0cf47c7b9be7a2e6ba89f429762e7b9adb").unwrap();
        let value_4 = to_checksum(addr_4);

        assert_eq!(
            value_1, "0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAed",
            "checksumed address does not match"
        );
        assert_eq!(
            value_2, "0xfB6916095ca1df60bB79Ce92cE3Ea74c37c5d359",
            "checksumed address does not match"
        );
        assert_eq!(
            value_3, "0xdbF03B407c01E7cD3CBea99509d93f8DDDC8C6FB",
            "checksumed address does not match"
        );
        assert_eq!(
            value_4, "0xD1220A0cf47c7B9Be7A2E6BA89F429762e7b9aDb",
            "checksumed address does not match"
        );
    }
}
