//! Some traits to de-/encode DER objects via type-specific zero-copy views as well as direct
//! de-/encode implementations for some native Rust types

mod boolean;
mod integer;
mod null;
mod octet_string;
mod sequence;
mod utf8_string;

pub use crate::typed::{
    boolean::Boolean, integer::Integer, null::Null, octet_string::OctetString, sequence::Sequence,
    utf8_string::Utf8String,
};
use crate::{error::ErrorChain, Asn1DerError, DerObject, Sink, Source};
#[cfg(all(feature = "std", not(feature = "no_panic")))]
pub use sequence::SequenceVec;

/// A trait for DER type views
pub trait DerTypeView<'a>: Sized {
    /// The tag for this type
    const TAG: u8;
    /// Provides raw access to the underlying `DerObject`
    fn object(&self) -> DerObject<'a>;
}

/// A trait for DER decodable types
pub trait DerDecodable<'a>: Sized {
    /// Loads `object` as `Self`
    fn load(object: DerObject<'a>) -> Result<Self, Asn1DerError>;
    /// Decodes an object as `Self`
    #[cfg_attr(feature = "no_panic", no_panic::no_panic)]
    fn decode(raw: &'a [u8]) -> Result<Self, Asn1DerError> {
        Self::decode_at(raw, 0)
    }
    /// Decodes an object as `Self`
    #[cfg_attr(feature = "no_panic", no_panic::no_panic)]
    fn decode_at(raw: &'a [u8], header_start: usize) -> Result<Self, Asn1DerError> {
        let object = DerObject::decode_at(raw, header_start).propagate(e!("Failed to decode object"))?;
        Self::load(object).propagate(e!("Failed to load object"))
    }
    /// Reads an object from `source` by parsing the length field and copying the necessary bytes
    /// into `sink` and decoding it from `sink`
    #[cfg_attr(feature = "no_panic", no_panic::no_panic)]
    fn decode_from_source<A: Source, B: Sink + Into<&'a [u8]>>(source: &mut A, sink: B) -> Result<Self, Asn1DerError> {
        let object = DerObject::decode_from_source(source, sink).propagate(e!("Failed to decode object"))?;
        Self::load(object).propagate(e!("Failed to load object"))
    }
}
impl<'a> DerDecodable<'a> for DerObject<'a> {
    #[cfg_attr(feature = "no_panic", no_panic::no_panic)]
    fn load(object: DerObject<'a>) -> Result<Self, Asn1DerError> {
        Ok(object)
    }
}

/// A trait for DER encodable types
pub trait DerEncodable: Sized {
    /// Encodes `self` into `sink`
    fn encode<S: Sink>(&self, sink: &mut S) -> Result<(), Asn1DerError>;

    /// Creates an DER object from an encodable type
    #[cfg_attr(feature = "no_panic", no_panic::no_panic)]
    fn der_object<'a, S: Sink + Into<&'a [u8]>>(&self, mut sink: S) -> Result<DerObject<'a>, Asn1DerError> {
        self.encode(&mut sink).propagate(e!("Failed to encode object"))?;
        DerObject::decode(sink.into()).propagate("Failed to load constructed object")
    }
}
impl<'a> DerEncodable for DerObject<'a> {
    #[cfg_attr(feature = "no_panic", no_panic::no_panic)]
    fn encode<S: Sink>(&self, sink: &mut S) -> Result<(), Asn1DerError> {
        self.encode(sink).propagate(e!("Failed to encode object"))
    }
}
impl<T: DerEncodable> DerEncodable for &T {
    #[cfg_attr(feature = "no_panic", no_panic::no_panic)]
    fn encode<S: Sink>(&self, sink: &mut S) -> Result<(), Asn1DerError> {
        (*self).encode(sink)
    }
}
impl<T: DerEncodable> DerEncodable for &mut T {
    #[cfg_attr(feature = "no_panic", no_panic::no_panic)]
    fn encode<S: Sink>(&self, sink: &mut S) -> Result<(), Asn1DerError> {
        (*self as &T).encode(sink)
    }
}
