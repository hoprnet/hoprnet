use crate::{
    error::ErrorChain,
    typed::{DerDecodable, DerEncodable, DerTypeView},
    Asn1DerError, DerObject, Sink,
};

/// An ASN.1-DER null object view
#[derive(Copy, Clone)]
pub struct Null<'a> {
    object: DerObject<'a>,
}
impl<'a> Null<'a> {
    /// Writes a new null object into `sink` and returns a type view over it
    #[cfg_attr(feature = "no_panic", no_panic::no_panic)]
    pub fn new<S: Sink + Into<&'a [u8]>>(mut sink: S) -> Result<Self, Asn1DerError> {
        Self::write(&mut sink).propagate(e!("Failed to construct null object"))?;
        let object = DerObject::decode(sink.into()).propagate(e!("Failed to load constructed null object"))?;
        Ok(Self { object })
    }

    /// Writes a boolean `value` as DER-object to `sink`
    #[cfg_attr(feature = "no_panic", no_panic::no_panic)]
    pub fn write<S: Sink>(sink: &mut S) -> Result<(), Asn1DerError> {
        DerObject::write(Self::TAG, 0, &mut b"".iter(), sink).propagate(e!("Failed to write null object"))
    }
}
impl<'a> DerTypeView<'a> for Null<'a> {
    const TAG: u8 = b'\x05';

    #[cfg_attr(feature = "no_panic", no_panic::no_panic)]
    fn object(&self) -> DerObject<'a> {
        self.object
    }
}
impl<'a> DerDecodable<'a> for Null<'a> {
    #[cfg_attr(feature = "no_panic", no_panic::no_panic)]
    fn load(object: DerObject<'a>) -> Result<Self, Asn1DerError> {
        match object.value() {
            _ if object.tag() != Self::TAG => Err(einval!("DER object is not a null object"))?,
            b"" => Ok(Self { object }),
            _ => Err(einval!("DER object is not a valid null object")),
        }
    }
}
impl<'a> DerEncodable for Null<'a> {
    /// Encodes `self` to `sink`
    #[cfg_attr(feature = "no_panic", no_panic::no_panic)]
    fn encode<S: Sink>(&self, sink: &mut S) -> Result<(), Asn1DerError> {
        self.object().encode(sink).propagate(e!("Failed to encode null object"))
    }
}

impl<'a> DerDecodable<'a> for () {
    #[cfg_attr(feature = "no_panic", no_panic::no_panic)]
    fn load(object: DerObject<'a>) -> Result<Self, Asn1DerError> {
        Null::load(object).propagate(e!("Failed to load null object"))?;
        Ok(())
    }
}
impl DerEncodable for () {
    #[cfg_attr(feature = "no_panic", no_panic::no_panic)]
    fn encode<S: Sink>(&self, sink: &mut S) -> Result<(), Asn1DerError> {
        Null::write(sink).propagate(e!("Failed to encode null object"))
    }
}

impl<'a, T: DerDecodable<'a>> DerDecodable<'a> for Option<T> {
    #[cfg_attr(feature = "no_panic", no_panic::no_panic)]
    fn load(object: DerObject<'a>) -> Result<Self, Asn1DerError> {
        match object.tag() {
            Null::TAG => {
                Null::load(object).propagate(e!("Failed to load null object"))?;
                Ok(None)
            }
            _ => {
                let object = T::load(object).propagate(e!("Failed to load object"))?;
                Ok(Some(object))
            }
        }
    }
}
impl<T: DerEncodable> DerEncodable for Option<T> {
    #[cfg_attr(feature = "no_panic", no_panic::no_panic)]
    #[cfg_attr(feature = "no_panic", inline(always))]
    fn encode<S: Sink>(&self, sink: &mut S) -> Result<(), Asn1DerError> {
        match self {
            Some(object) => object.encode(sink).propagate(e!("Failed to encode object")),
            None => Null::write(sink).propagate(e!("Failed to encode null object")),
        }
    }
}
