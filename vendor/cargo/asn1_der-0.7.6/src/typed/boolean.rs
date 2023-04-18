use crate::{
    error::ErrorChain,
    typed::{DerDecodable, DerEncodable, DerTypeView},
    Asn1DerError, DerObject, Sink,
};

/// An ASN.1-DER boolean type view
#[derive(Copy, Clone)]
pub struct Boolean<'a> {
    object: DerObject<'a>,
}
impl<'a> Boolean<'a> {
    /// Writes a new boolean object with `value` into `sink` and returns a type view over it
    #[cfg_attr(feature = "no_panic", no_panic::no_panic)]
    pub fn new<S: Sink + Into<&'a [u8]>>(value: bool, mut sink: S) -> Result<Self, Asn1DerError> {
        Self::write(value, &mut sink).propagate(e!("Failed to construct boolean"))?;
        let object = DerObject::decode(sink.into()).propagate(e!("Failed to load constructed boolean"))?;
        Ok(Self { object })
    }
    /// Gets the boolean value
    #[cfg_attr(feature = "no_panic", no_panic::no_panic)]
    pub fn get(&self) -> bool {
        match self.object.value() {
            b"\x00" => false,
            // #implicit_validation: Since we validate this value at `load`, the only possible value
            // here is `b"\xff"` unless the underlying object has been modified in an invalid way
            _ => true,
        }
    }

    /// Writes a boolean `value` as DER-object to `sink`
    #[cfg_attr(feature = "no_panic", no_panic::no_panic)]
    pub fn write<S: Sink>(value: bool, sink: &mut S) -> Result<(), Asn1DerError> {
        let value = match value {
            true => b"\xff".as_ref(),
            false => b"\x00".as_ref(),
        };
        DerObject::write(Self::TAG, value.len(), &mut value.iter(), sink).propagate(e!("Failed to write boolean"))
    }
}
impl<'a> DerTypeView<'a> for Boolean<'a> {
    const TAG: u8 = b'\x01';

    #[cfg_attr(feature = "no_panic", no_panic::no_panic)]
    fn object(&self) -> DerObject<'a> {
        self.object
    }
}
impl<'a> DerDecodable<'a> for Boolean<'a> {
    #[cfg_attr(feature = "no_panic", no_panic::no_panic)]
    fn load(object: DerObject<'a>) -> Result<Self, Asn1DerError> {
        match object.value() {
            _ if object.tag() != Self::TAG => Err(einval!("DER object is not a boolean"))?,
            b"\x00" | b"\xff" => Ok(Self { object }),
            _ => Err(einval!("DER object is not a valid boolean")),
        }
    }
}
impl<'a> DerEncodable for Boolean<'a> {
    #[cfg_attr(feature = "no_panic", no_panic::no_panic)]
    fn encode<U: Sink>(&self, sink: &mut U) -> Result<(), Asn1DerError> {
        self.object().encode(sink).propagate(e!("Failed to encode boolean"))
    }
}

impl<'a> DerDecodable<'a> for bool {
    #[cfg_attr(feature = "no_panic", no_panic::no_panic)]
    fn load(object: DerObject<'a>) -> Result<Self, Asn1DerError> {
        let boolean = Boolean::load(object).propagate(e!("Failed to load boolean"))?;
        Ok(boolean.get())
    }
}
impl DerEncodable for bool {
    #[cfg_attr(feature = "no_panic", no_panic::no_panic)]
    fn encode<S: Sink>(&self, sink: &mut S) -> Result<(), Asn1DerError> {
        Boolean::write(*self, sink).propagate(e!("Failed to encode boolean"))
    }
}
