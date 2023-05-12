#![cfg_attr(test, deny(warnings))]

pub mod helpers;

use crate::helpers::{test_ok, OptionExt, ResultExt};
#[cfg(all(feature = "std", not(feature = "no_panic")))]
use asn1_der::VecBacking;
use asn1_der::{der, DerObject, Sink};

#[test]
fn length() {
    for test in test_ok::load().length {
        if let Some(value) = test.value {
            // Test valid lengths
            if value <= usize::max_value() as u64 {
                // Decode length
                let len = der::length::decode(&mut test.bytes.iter()).assert(&test.name).assert(&test.name);
                assert_eq!(len, value as usize, "@\"{}\"", &test.name);

                // Encode length
                let (mut buf, mut buf_len) = ([0; 9], 0);
                let mut sink = buf.iter_mut().counting_sink(&mut buf_len);
                der::length::encode(len, &mut sink).assert(&test.name);
                assert_eq!(&buf[..buf_len], test.bytes.as_slice(), "@\"{}\"", &test.name);
            }
        } else {
            // Test truncated lengths
            let len = der::length::decode(&mut test.bytes.iter()).assert(&test.name);
            assert!(len.is_none(), "@\"{}\"", &test.name);
        }
    }
}

#[test]
fn object() {
    for test in test_ok::load().object {
        // Test-copy the object
        #[cfg(all(feature = "std", not(feature = "no_panic")))]
        {
            let mut bytes = Vec::new();
            DerObject::decode_from_source(&mut test.bytes.iter(), VecBacking(&mut bytes)).assert(&test.name);
            assert_eq!(bytes, test.bytes, "@\"{}\"", &test.name);
        }

        // Decode the object
        let object = DerObject::decode(test.bytes.as_slice()).assert(&test.name);
        assert_eq!(object.tag(), test.tag, "@\"{}\"", &test.name);
        assert_eq!(object.value(), test.value.as_slice(), "@\"{}\"", &test.name);

        // Encode the object
        let mut bytes = vec![0; test.bytes.len()];
        object.encode(&mut bytes.iter_mut()).assert(&test.name);
        assert_eq!(bytes, test.bytes, "@\"{}\"", &test.name)
    }
}
