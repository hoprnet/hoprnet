#![cfg_attr(test, deny(warnings))]
#![cfg(feature = "native_types")]

pub mod helpers;

use crate::helpers::{test_ok, ResultExt};
#[cfg(all(feature = "std", not(feature = "no_panic")))]
use asn1_der::typed::SequenceVec;
use asn1_der::{
    typed::{Boolean, DerDecodable, DerEncodable, DerTypeView, Integer, Null, OctetString, Sequence, Utf8String},
    DerObject, SliceSink,
};
use core::convert::TryFrom;

#[test]
fn boolean() {
    for test in test_ok::load().typed.bool {
        // Decode the object
        let boolean = Boolean::decode(&test.bytes).assert(&test.name);
        assert_eq!(boolean.get(), test.bool, "@\"{}\"", &test.name);

        let native = bool::decode(&test.bytes).assert(&test.name);
        assert_eq!(native, test.bool, "@\"{}\"", &test.name);

        // Encode the object
        let mut bytes = vec![0; test.bytes.len()];
        boolean.encode(&mut bytes.iter_mut()).assert(&test.name);
        assert_eq!(bytes, test.bytes, "@\"{}\"", &test.name);

        let mut bytes = vec![0; test.bytes.len()];
        test.bool.encode(&mut bytes.iter_mut()).assert(&test.name);
        assert_eq!(bytes, test.bytes, "@\"{}\"", &test.name);

        let (mut bytes, mut pos) = ([0; 1024], 0);
        let sink = SliceSink::new(&mut bytes, &mut pos);
        Boolean::new(test.bool, sink).assert(&test.name);
        assert_eq!(&bytes[..pos], test.bytes.as_slice(), "@\"{}\"", &test.name);
    }
}

#[test]
fn integer() {
    for test in test_ok::load().typed.integer {
        // Decode the object
        let object = Integer::decode(test.bytes.as_slice()).assert(&test.name);
        assert_eq!(object.object().value(), test.value.as_slice(), "@\"{}\"", &test.name);

        // Encode the object
        let mut bytes = vec![0; test.bytes.len()];
        object.encode(&mut bytes.iter_mut()).assert(&test.name);
        assert_eq!(bytes, test.bytes, "@\"{}\"", &test.name);

        // Test native types
        macro_rules! native {
			($num:ty, $field:ident, $is_signed:expr) => {
				if let Some(value) = test.$field.and_then(|n| <$num>::try_from(n).ok()) {
					// Decode native
					let native = <$num>::decode(test.bytes.as_slice()).assert(&test.name);
					assert_eq!(native, value, "@\"{}\"", &test.name);

					// Encode native
					let mut bytes = vec![0; test.bytes.len()];
					value.encode(&mut bytes.iter_mut()).assert(&test.name);
					assert_eq!(bytes, test.bytes, "@\"{}\"", &test.name);

					let (mut bytes, mut pos) = ([0; 1024], 0);
					let sink = SliceSink::new(&mut bytes, &mut pos);
					Integer::new(&value.to_be_bytes(), $is_signed(value), sink).assert(&test.name);
					assert_eq!(&bytes[..pos], test.bytes.as_slice(), "@\"{}\"", &test.name);
				}
			};
			(unsigned: $( $num:ty ),+) => ($( native!($num, uint, |_| false); )+);
		}
        native!(unsigned: u8, u16, u32, u64, u128, usize);
    }
}

#[test]
fn null() {
    for test in test_ok::load().typed.null {
        const TRUE: &[u8] = b"\x01\x01\xff";
        type OptBool = Option<bool>;

        // Decode the object
        let object = Null::decode(test.bytes.as_slice()).assert(&test.name);

        let native = OptBool::decode(test.bytes.as_slice()).assert(&test.name);
        assert!(native.is_none(), "@\"{}\"", &test.name);

        let native = OptBool::decode(TRUE).assert(&test.name);
        assert_eq!(native, Some(true), "@\"{}\"", &test.name);

        // Encode the object
        let mut bytes = vec![0; test.bytes.len()];
        object.encode(&mut bytes.iter_mut()).assert(&test.name);
        assert_eq!(bytes, test.bytes, "@\"{}\"", &test.name);

        let (mut bytes, mut pos) = ([0; 1024], 0);
        let sink = SliceSink::new(&mut bytes, &mut pos);
        Null::new(sink).assert(&test.name);
        assert_eq!(&bytes[..pos], test.bytes.as_slice(), "@\"{}\"", &test.name);

        let mut bytes = [0; 2];
        OptBool::None.encode(&mut bytes.iter_mut()).assert(&test.name);
        assert_eq!(bytes.as_ref(), test.bytes.as_slice(), "@\"{}\"", &test.name);
    }
}

#[test]
fn octet_string() {
    for test in test_ok::load().typed.octet_string {
        // Decode the object
        let object = OctetString::decode(test.bytes.as_slice()).assert(&test.name);
        assert_eq!(object.get(), test.value.as_slice(), "@\"{}\"", &test.name);

        #[cfg(all(feature = "std", not(feature = "no_panic")))]
        {
            let native = Vec::<u8>::decode(test.bytes.as_slice()).assert(&test.name);
            assert_eq!(native, test.value, "@\"{}\"", &test.name);
        }

        // Encode the object
        let mut bytes = vec![0; test.bytes.len()];
        object.encode(&mut bytes.iter_mut()).assert(&test.name);
        assert_eq!(bytes, test.bytes, "@\"{}\"", &test.name);

        #[cfg(all(feature = "std", not(feature = "no_panic")))]
        {
            let mut bytes = vec![0; test.bytes.len()];
            test.value.encode(&mut bytes.iter_mut()).assert(&test.name);
            assert_eq!(bytes, test.bytes, "@\"{}\"", &test.name);
        }

        let (mut bytes, mut pos) = ([0; 1024], 0);
        let sink = SliceSink::new(&mut bytes, &mut pos);
        OctetString::new(&test.value, sink).assert(&test.name);
        assert_eq!(&bytes[..pos], test.bytes.as_slice(), "@\"{}\"", &test.name);
    }
}

#[test]
fn sequence() {
    for test in test_ok::load().typed.sequence {
        // Decode the object
        let object = Sequence::decode(test.bytes.as_slice()).assert(&test.name);
        assert_eq!(object.object().value(), test.value.as_slice(), "@\"{}\"", &test.name);

        for (i, obj) in test.sequence.iter().enumerate() {
            let object = object.get(i).assert_index(&test.name, i);
            assert_eq!(object.tag(), obj.tag, "@\"{}\"", &test.name);
            assert_eq!(object.value(), obj.value.as_slice(), "@\"{}\":{}", &test.name, i);
        }

        #[cfg(all(feature = "std", not(feature = "no_panic")))]
        {
            let native = SequenceVec::<Vec<u8>>::decode(test.bytes.as_slice()).assert(&test.name);
            for (i, obj) in test.sequence.iter().enumerate() {
                assert_eq!(native[i], obj.value.as_slice(), "@\"{}\":{}", &test.name, i);
            }
        }

        // Encode the object
        let mut bytes = vec![0; test.bytes.len()];
        object.encode(&mut bytes.iter_mut()).assert(&test.name);
        assert_eq!(bytes, test.bytes, "@\"{}\"", &test.name);

        #[cfg(all(feature = "std", not(feature = "no_panic")))]
        {
            let values: Vec<_> =
                test.sequence.iter().map(|o| DerObject::decode(o.bytes.as_slice()).assert(&test.name)).collect();
            let mut bytes = vec![0; test.bytes.len()];
            SequenceVec(values).encode(&mut bytes.iter_mut()).assert(&test.name);
            assert_eq!(bytes, test.bytes, "@\"{}\"", &test.name);
        }

        {
            let values: Vec<_> =
                test.sequence.iter().map(|o| DerObject::decode(o.bytes.as_slice()).assert(&test.name)).collect();

            let (mut bytes, mut pos) = ([0; 4096], 0);
            let sink = SliceSink::new(&mut bytes, &mut pos);
            Sequence::new(&values, sink).assert(&test.name);
            assert_eq!(&bytes[..pos], test.bytes.as_slice(), "@\"{}\"", &test.name);
        }
    }
}

#[test]
fn utf8_string() {
    for test in test_ok::load().typed.utf8_string {
        // Decode the object
        let object = Utf8String::decode(test.bytes.as_slice()).assert(&test.name);
        assert_eq!(object.get(), test.utf8str.as_str(), "@\"{}\"", &test.name);

        #[cfg(all(feature = "std", not(feature = "no_panic")))]
        {
            let native = String::decode(test.bytes.as_slice()).assert(&test.name);
            assert_eq!(native, test.utf8str, "@\"{}\"", &test.name);
        }

        // Encode the object
        let mut bytes = vec![0; test.bytes.len()];
        object.encode(&mut bytes.iter_mut()).assert(&test.name);
        assert_eq!(bytes, test.bytes, "@\"{}\"", &test.name);

        #[cfg(all(feature = "std", not(feature = "no_panic")))]
        {
            let mut bytes = vec![0; test.bytes.len()];
            test.utf8str.encode(&mut bytes.iter_mut()).assert(&test.name);
            assert_eq!(bytes, test.bytes, "@\"{}\"", &test.name);
        }

        let (mut bytes, mut pos) = ([0; 1024], 0);
        let sink = SliceSink::new(&mut bytes, &mut pos);
        Utf8String::new(&test.utf8str, sink).assert(&test.name);
        assert_eq!(&bytes[..pos], test.bytes.as_slice(), "@\"{}\"", &test.name);
    }
}
