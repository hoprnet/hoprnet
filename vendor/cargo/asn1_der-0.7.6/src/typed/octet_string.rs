use crate::{
    error::ErrorChain,
    typed::{DerDecodable, DerEncodable, DerTypeView},
    Asn1DerError, DerObject, Sink,
};

/// An ASN.1-DER octet string view
#[derive(Copy, Clone)]
pub struct OctetString<'a> {
    object: DerObject<'a>,
}
impl<'a> OctetString<'a> {
    /// Writes a new octet string object with `value` into `sink` and returns a type view over it
    #[cfg_attr(feature = "no_panic", no_panic::no_panic)]
    pub fn new<S: Sink + Into<&'a [u8]>>(value: &[u8], mut sink: S) -> Result<Self, Asn1DerError> {
        Self::write(value, &mut sink).propagate(e!("Failed to construct octet string"))?;
        let object = DerObject::decode(sink.into()).propagate(e!("Failed to load constructed octet string"))?;
        Ok(Self { object })
    }
    /// Gets the octet string value
    #[cfg_attr(feature = "no_panic", no_panic::no_panic)]
    pub fn get(&self) -> &[u8] {
        self.object.value()
    }

    /// Writes an octet string `value` as DER-object to `sink`
    #[cfg_attr(feature = "no_panic", no_panic::no_panic)]
    pub fn write<S: Sink>(value: &[u8], sink: &mut S) -> Result<(), Asn1DerError> {
        DerObject::write(Self::TAG, value.len(), &mut value.iter(), sink).propagate(e!("Failed to write octet string"))
    }
}
impl<'a> DerTypeView<'a> for OctetString<'a> {
    const TAG: u8 = b'\x04';

    #[cfg_attr(feature = "no_panic", no_panic::no_panic)]
    fn object(&self) -> DerObject<'a> {
        self.object
    }
}
impl<'a> DerDecodable<'a> for OctetString<'a> {
    #[cfg_attr(feature = "no_panic", no_panic::no_panic)]
    fn load(object: DerObject<'a>) -> Result<Self, Asn1DerError> {
        match object.value() {
            _ if object.tag() != Self::TAG => Err(einval!("DER object is not an octet string"))?,
            _ => Ok(Self { object }),
        }
    }
}
impl<'a> DerEncodable for OctetString<'a> {
    /// Encodes `self` to `sink`
    #[cfg_attr(feature = "no_panic", no_panic::no_panic)]
    fn encode<U: Sink>(&self, sink: &mut U) -> Result<(), Asn1DerError> {
        self.object().encode(sink).propagate(e!("Failed to encode octet string"))
    }
}

#[cfg(all(feature = "std", not(feature = "no_panic")))]
impl<'a> DerDecodable<'a> for Vec<u8> {
    fn load(object: DerObject<'a>) -> Result<Self, Asn1DerError> {
        let octet_string = OctetString::load(object).propagate(e!("Failed to load octet string"))?;
        Ok(octet_string.get().to_vec())
    }
}
#[cfg(all(feature = "std", not(feature = "no_panic")))]
impl DerEncodable for Vec<u8> {
    fn encode<S: Sink>(&self, sink: &mut S) -> Result<(), Asn1DerError> {
        OctetString::write(self, sink).propagate(e!("Failed to encode octet string"))
    }
}
