use crate::{
	Asn1DerError, DerObject, Sink, error::ErrorChain, rust::str,
	typed::{ DerTypeView, DerDecodable, DerEncodable }
};


/// An ASN.1-DER UTF-8 string view
#[derive(Copy, Clone)]
pub struct Utf8String<'a> {
	object: DerObject<'a>
}
impl<'a> Utf8String<'a> {
	/// Writes a new UTF8String object with `value` into `sink` and returns a type view over it
	#[cfg_attr(feature = "no_panic", no_panic::no_panic)]
	pub fn new<S: Sink + Into<&'a[u8]>>(value: &str, mut sink: S) -> Result<Self, Asn1DerError> {
		Self::write(value, &mut sink).propagate(e!("Failed to construct UTF-8 string"))?;
		let object = DerObject::decode(sink.into())
			.propagate(e!("Failed to load constructed UTF-8 string"))?;
		Ok(Self{ object })
	}
	/// Gets the UTF-8 string value
	#[allow(unsafe_code)] #[cfg_attr(feature = "no_panic", no_panic::no_panic)]
	pub fn get(&self) -> &str {
		match self.object.value() {
			s if Self::is_utf8(s) => unsafe{ str::from_utf8_unchecked(s) },
			// #implicit_validation: Since we validate the string on `load`, this codepath is only
			// possible if the underlying object has been modified in an invalid way
			_ => ""
		}
	}
	
	/// Writes an UTF-8 string `value` as DER-object to `sink`
	#[cfg_attr(feature = "no_panic", no_panic::no_panic)]
	pub fn write<S: Sink>(value: &str, sink: &mut S) -> Result<(), Asn1DerError> {
		DerObject::write(Self::TAG, value.len(), &mut value.as_bytes().iter(), sink)
			.propagate(e!("Failed to write UTF-8 string"))
	}
	
	/// Checks if a string is UTF-8
	///
	/// _#implicit_validation: we use a `#[inline(never)] extern "C" fn` here to hide this code from
	/// `no_panic` because either way we have to assume that the code in the standard library is
	/// correct – see also
	/// [Is there a way to use potentially panicking code in a #[no_panic] function #16](https://github.com/dtolnay/no-panic/issues/16)_
	#[inline(never)] #[allow(improper_ctypes_definitions)]
	extern "C" fn is_utf8(slice: &[u8]) -> bool {
		str::from_utf8(slice).is_ok()
	}
}
impl<'a> DerTypeView<'a> for Utf8String<'a> {
	const TAG: u8 = b'\x0c';
	
	#[cfg_attr(feature = "no_panic", no_panic::no_panic)]
	fn object(&self) -> DerObject<'a> {
		self.object
	}
}
impl<'a> DerDecodable<'a> for Utf8String<'a> {
	#[allow(unsafe_code)] #[cfg_attr(feature = "no_panic", no_panic::no_panic)]
	fn load(object: DerObject<'a>) -> Result<Self, Asn1DerError> {
		match object.value() {
			_ if object.tag() != Self::TAG => Err(einval!("DER object is not an UTF-8 string"))?,
			s => match Self::is_utf8(s) {
				true => Ok(Self{ object }),
				false => Err(einval!("DER object is not a valid UTF-8 string"))
			}
		}
	}
}
impl<'a> DerEncodable for Utf8String<'a> {
	/// Encodes `self` to `sink`
	#[cfg_attr(feature = "no_panic", no_panic::no_panic)]
	fn encode<U: Sink>(&self, sink: &mut U) -> Result<(), Asn1DerError> {
		self.object().encode(sink).propagate(e!("Failed to encode UTF-8 string"))
	}
}


#[cfg(not(any(feature = "no_std", feature = "no_panic")))]
impl<'a> DerDecodable<'a> for String {
	fn load(object: DerObject<'a>) -> Result<Self, Asn1DerError> {
		let string = Utf8String::load(object).propagate(e!("Failed to load UTF-8 string"))?;
		Ok(string.get().to_string())
	}
}
#[cfg(not(any(feature = "no_std", feature = "no_panic")))]
impl DerEncodable for String {
	fn encode<S: Sink>(&self, sink: &mut S) -> Result<(), Asn1DerError> {
		Utf8String::write(self, sink).propagate(e!("Failed to encode UTF-8 string"))
	}
}