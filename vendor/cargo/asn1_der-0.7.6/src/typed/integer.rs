use crate::{
    error::ErrorChain,
    typed::{DerDecodable, DerEncodable, DerTypeView},
    Asn1DerError, DerObject, Sink,
};
use core::mem;

/// An ASN.1-DER integer view
#[derive(Copy, Clone)]
pub struct Integer<'a> {
    object: DerObject<'a>,
}
impl<'a> Integer<'a> {
    /// Writes a new integer object with the big-endian encoded `be_value` into `sink` and returns a
    /// type view over it
    #[cfg_attr(feature = "no_panic", no_panic::no_panic)]
    pub fn new<S: Sink + Into<&'a [u8]>>(
        be_value: &[u8],
        is_negative: bool,
        mut sink: S,
    ) -> Result<Self, Asn1DerError> {
        Self::write(be_value, is_negative, &mut sink).propagate(e!("Failed to construct integer"))?;
        let object = DerObject::decode(sink.into()).propagate(e!("Failed to load constructed integer"))?;
        Ok(Self { object })
    }

    /// Returns if the number is negative or not
    #[cfg_attr(feature = "no_panic", no_panic::no_panic)]
    pub fn is_negative(&self) -> bool {
        match self.object.value().first() {
            Some(first) => first & 0b1000_0000 != 0,
            // #implicit_validation: Since we validate the length at `load`, there is always a first
            // element which in turn does not begin with a leading `1`-bit unless the underlying
            // object has been modified in an invalid way
            None => false,
        }
    }
    /// Get the number bytes
    ///
    /// __Important: Any leading zero-byte that might indicate a positive number is stripped off.
    /// This means that a return value of `0b1111_1111` can be either `255` or `-1` depending on
    /// whether the number is negative or not. Use `is_negative` to determine the correct sign.__
    #[cfg_attr(feature = "no_panic", no_panic::no_panic)]
    pub fn get_numbytes(&self) -> &[u8] {
        let slice = self.object.value();
        match slice.len() {
            len if len >= 1 && slice[0] == 0 => &slice[1..],
            _ => slice,
        }
    }

    /// Copies the num bytes into `buf` if they can fit
    ///
    /// __Important: Any leading zero-byte that might indicate a positive number is stripped off.
    /// This means that a return value of `0b1111_1111` can be either `255` or `-1` depending on
    /// whether the number is negative or not. Use `is_negative` to determine the correct sign.__
    #[cfg_attr(feature = "no_panic", no_panic::no_panic)]
    pub fn copy_numbytes<T: AsMut<[u8]>>(&self, mut buf: T) -> Result<T, Asn1DerError> {
        // Ensure that the number fits
        let (slice, buf_slice) = (self.get_numbytes(), buf.as_mut());
        let to_skip = match slice.len() {
            len if len > buf_slice.len() => Err(eunsupported!("The numeric value is too large"))?,
            len => buf_slice.len() - len,
        };

        // Copy the number and ensure that it can be represented
        buf_slice.iter_mut().skip(to_skip).zip(slice.iter()).for_each(|(t, b)| *t = *b);
        Ok(buf)
    }
    /// Writes an integer `value` as DER-object to `sink`
    #[cfg_attr(feature = "no_panic", no_panic::no_panic)]
    #[inline(always)]
    pub fn write<S: Sink>(value: &[u8], is_negative: bool, sink: &mut S) -> Result<(), Asn1DerError> {
        // Determine the amount leading zero bytes to skip
        let to_skip = value.iter().take_while(|b| **b == 0).count();
        let len = value.iter().skip(to_skip).count();

        // Construct a leading zero byte prefix if necessary
        let value_prefix = match value.get(to_skip) {
            None => b"\x00".as_ref(),
            Some(first) if first & 0b1000_0000 != 0 && !is_negative => b"\x00".as_ref(),
            _ => b"".as_ref(),
        };
        let len = match len.checked_add(value_prefix.len()) {
            Some(len) => len,
            None => Err(eunsupported!("The number length would exceed `usize::max_value()`"))?,
        };

        // Encode integer
        let mut value = value_prefix.iter().chain(value.iter().skip(to_skip));
        DerObject::write(Self::TAG, len, &mut value, sink).propagate(e!("Failed to write integer"))
    }
}
impl<'a> DerTypeView<'a> for Integer<'a> {
    const TAG: u8 = b'\x02';

    #[cfg_attr(feature = "no_panic", no_panic::no_panic)]
    fn object(&self) -> DerObject<'a> {
        self.object
    }
}
impl<'a> DerDecodable<'a> for Integer<'a> {
    #[cfg_attr(feature = "no_panic", no_panic::no_panic)]
    fn load(object: DerObject<'a>) -> Result<Self, Asn1DerError> {
        match object.value() {
            _ if object.tag() != Self::TAG => Err(einval!("DER object is not an integer"))?,
            value if value.is_empty() => Err(einval!("DER object is not a valid integer"))?,
            value if value.len() >= 2 && value[0] == b'\x00' && value[1] & 0b1000_0000 == 0 => {
                Err(einval!("DER object is not a valid integer"))
            }
            value if value.len() >= 2 && value[0] == b'\xff' && value[1] & 0b1000_0000 != 0 => {
                Err(einval!("DER object is not a valid integer"))
            }
            _ => Ok(Self { object }),
        }
    }
}
impl<'a> DerEncodable for Integer<'a> {
    /// Encodes `self` to `sink`
    #[cfg_attr(feature = "no_panic", no_panic::no_panic)]
    fn encode<S: Sink>(&self, sink: &mut S) -> Result<(), Asn1DerError> {
        self.object().encode(sink).propagate(e!("Failed to encode integer"))
    }
}

/// Implements `DerCodable`
macro_rules! impl_dercodable {
	(unsigned: $num:ty) => {
		impl<'a> DerDecodable<'a> for $num {
			#[cfg_attr(feature = "no_panic", no_panic::no_panic)]
			fn load(object: DerObject<'a>) -> Result<Self, Asn1DerError> {
				// Load integer
				let integer = Integer::load(object).propagate(e!("Failed to load integer"))?;
				let buf = integer.copy_numbytes([0; mem::size_of::<Self>()])
					.propagate(e!("The numeric value is too large"))?;

				// Validate the integer
				match Self::from_be_bytes(buf) {
					_ if integer.is_negative() =>
						Err(eunsupported!("The numeric value is negative")),
					num => Ok(num)
				}
			}
		}
		impl DerEncodable for $num {
			#[cfg_attr(feature = "no_panic", no_panic::no_panic)]
			fn encode<S: Sink>(&self, sink: &mut S) -> Result<(), Asn1DerError> {
				Integer::write(&self.to_be_bytes(), false, sink)
			}
		}
	};
	(unsigned: $($num:ty),+) => ($( impl_dercodable!(unsigned: $num); )+);
}
impl_dercodable!(unsigned: u8, u16, u32, u64, u128, usize);
