//! This crate contains basic types used throughout the entire HOPR codebase.
//! Types from this crate are not necessarily specific only to HOPR.

pub mod errors;
pub mod primitives;
pub mod sma;
pub mod traits;

// TODO: remove in 3.0
// #[deprecated(note = "RLP encoding will be removed in 3.0")]
pub mod rlp {
    use crate::errors::GeneralError;
    use std::time::Duration;

    pub fn encode(data: &[u8], timestamp: Duration) -> Box<[u8]> {
        // For compatibility with JS, strip the leading 2 bytes if the timestamp byte array is longer than 6 bytes
        let ts = (timestamp.as_millis() as u64).to_be_bytes();
        let ts_encoded = if ts.len() > 6 { &ts[2..] } else { &ts };

        rlp::encode_list::<&[u8], &[u8]>(&[data, ts_encoded])
            .to_vec()
            .into_boxed_slice()
    }

    pub fn decode(data: &[u8]) -> crate::errors::Result<(Box<[u8]>, Duration)> {
        let mut list = rlp::decode_list::<Vec<u8>>(data);
        if list.len() != 2 {
            return Err(GeneralError::ParseError);
        }

        let enc_ts = list.remove(1);
        let ts_len = enc_ts.len();
        if ts_len > 8 {
            return Err(GeneralError::ParseError);
        }

        let mut ts = [0u8; 8];
        ts[8 - ts_len..].copy_from_slice(&enc_ts);

        let ts = u64::from_be_bytes(ts);
        Ok((list.remove(0).into_boxed_slice(), Duration::from_millis(ts)))
    }
}

pub mod prelude {
    pub use super::errors::GeneralError;
    pub use super::primitives::*;
    pub use super::sma::*;
    pub use super::traits::*;
}

#[allow(deprecated)]
#[cfg(test)]
mod tests {
    use hex_literal::hex;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    #[test]
    fn test_rlp() {
        let mut b_1 = [0u8; 100];
        let ts_1 = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();

        hopr_crypto_random::random_fill(&mut b_1);

        let (b_2, ts_2) = crate::rlp::decode(crate::rlp::encode(&b_1, ts_1).as_ref()).expect("must decode");

        assert_eq!(&b_1, b_2.as_ref(), "data must be equal");
        assert_eq!(
            ts_1.as_millis(),
            ts_2.as_millis(),
            "timestamps must be equal up to milliseconds"
        );
    }

    #[test]
    fn test_rlp_fixed() {
        let b_1 = b"hello";
        let ts_1 = Duration::from_millis(1703086927316);

        let data = hex!("cd8568656c6c6f86018c87e42dd4");
        let (b_2, ts_2) = crate::rlp::decode(&data).expect("must decode");

        assert_eq!(b_1, b_2.as_ref(), "data must be equal");
        assert_eq!(ts_1, ts_2, "timestamps must be equal up to milliseconds");
    }

    #[test]
    fn test_rlp_zero() {
        let b_1 = [0u8; 0];
        let ts_1 = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();

        let (b_2, ts_2) = crate::rlp::decode(crate::rlp::encode(&b_1, ts_1).as_ref()).expect("must decode");

        assert_eq!(&b_1, b_2.as_ref(), "data must be equal");
        assert_eq!(
            ts_1.as_millis(),
            ts_2.as_millis(),
            "timestamps must be equal up to milliseconds"
        );
    }
}
