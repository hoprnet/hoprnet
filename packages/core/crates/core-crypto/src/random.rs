use elliptic_curve::rand_core::OsRng;
use elliptic_curve::sec1::ToEncodedPoint;
use k256::NonZeroScalar;
use crate::errors::CryptoError::InvalidInputValue;

use crate::errors::Result;

use rand::{Rng, RngCore};

pub const MAX_RANDOM_INTEGER: u64 = 9007199254740991;

pub fn random_float() -> f64 {
    OsRng.gen()
}

pub fn random_bounded_integer(bound: u64) -> u64 {
    OsRng.gen_range(0..bound)
}

pub fn random_integer(start: u64, end: Option<u64>) -> Result<u64> {
    let real_end = end.unwrap_or(MAX_RANDOM_INTEGER);

    if real_end <= start || real_end > MAX_RANDOM_INTEGER {
        Err(InvalidInputValue)
    }
    else {
        Ok(start + random_bounded_integer(real_end - start))
    }
}

pub fn random_group_element(compressed: bool) -> (Box<[u8]>,Box<[u8]>) {
    let scalar = NonZeroScalar::random(&mut OsRng);
    let point = k256::ProjectivePoint::GENERATOR * scalar.as_ref();

    let encoded = point.to_encoded_point(compressed);

    (scalar.to_bytes().as_slice().into(), encoded.as_bytes().into())
}

pub fn random_fill(buffer: &mut [u8], from: usize, len: usize) {
    assert!(from + len <= buffer.len());
    OsRng.fill_bytes(&mut buffer[from..from + len]);
}

pub mod wasm {
    use js_sys::Uint8Array;
    use wasm_bindgen::prelude::wasm_bindgen;
    use crate::utils::{as_jsvalue, JsResult};

    #[wasm_bindgen]
    pub struct GroupElement {
        coeff: Box<[u8]>,
        element: Box<[u8]>,
        compressed: bool
    }

    #[wasm_bindgen]
    impl GroupElement {
        pub fn random(compressed: bool) -> GroupElement {
            let (coeff, element) = crate::random::random_group_element(compressed);
            Self {
                coeff,
                element,
                compressed
            }
        }

        pub fn coefficient(&self) -> Uint8Array {
            self.coeff.as_ref().into()
        }

        pub fn element(&self) -> Uint8Array {
            self.element.as_ref().into()
        }

        pub fn is_compressed(&self) -> bool {
            self.compressed
        }
    }

    #[wasm_bindgen]
    pub fn random_float() -> f64 {
        crate::random::random_float()
    }

    #[wasm_bindgen]
    pub fn random_bounded_integer(bound: u64) -> u64 {
        crate::random::random_bounded_integer(bound)
    }

    #[wasm_bindgen]
    pub fn random_integer(start: u64, end: Option<u64>) -> JsResult<u64> {
        crate::random::random_integer(start, end).map_err(as_jsvalue)
    }


    #[wasm_bindgen]
    pub fn random_fill(buffer: Uint8Array, from: usize, len: usize) {
        let mut buf = vec![0u8; buffer.length() as usize];
        crate::random::random_fill(buf.as_mut_slice(), from, len);
        buffer.copy_from(buf.as_slice());
    }
}