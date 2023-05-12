use crate::{error::ErrorChain, Asn1DerError, Sink, Source};

/// A mod for ASN.1-length-coding
pub mod length {
    use crate::{error::ErrorChain, Asn1DerError, Sink, Source};
    use core::mem;

    /// The byte length of an `usize`
    const SIZE: usize = mem::size_of::<usize>();

    /// Tries to read the length or returns `None` if there are not enough bytes
    #[cfg_attr(feature = "no_panic", no_panic::no_panic)]
    #[cfg_attr(feature = "no_panic", inline(always))]
    pub fn decode<S: Source>(source: &mut S) -> Result<Option<usize>, Asn1DerError> {
        // Read first byte
        let first = match source.read() {
            Ok(first) => first,
            Err(_) => return Ok(None),
        };

        // Check if we have a simple or a complex length
        match first as usize {
            len if len < 0b1000_0000 => Ok(Some(len)),
            size if size & 0b0111_1111 > SIZE => {
                Err(eunsupported!("The object length is greater than `usize::max_value()`"))
            }
            size => {
                // Prepare buffer
                let skip = SIZE - (size & 0b0111_1111);
                let mut buf = [0u8; SIZE];

                // Read the complex length bytes or return `None` in case the length is truncated
                for target in buf.iter_mut().skip(skip) {
                    *target = match source.read() {
                        Ok(next) => next,
                        Err(_) => return Ok(None),
                    };
                }

                // Validate the length
                match usize::from_be_bytes(buf) {
                    len if len < 0b1000_0000 => Err(einval!("Encountered complex length < 128"))?,
                    len => Ok(Some(len)),
                }
            }
        }
    }

    /// Encodes `len` to `sink` and returns the amount of
    #[cfg_attr(feature = "no_panic", no_panic::no_panic)]
    #[cfg_attr(feature = "no_panic", inline(always))]
    pub fn encode<S: Sink>(len: usize, sink: &mut S) -> Result<(), Asn1DerError> {
        match len {
            // Write simple length
            len if len < 128 => sink.write(len as u8).propagate(e!("Failed to write length byte")),
            // Encode complex length
            len => {
                // Compute the size of the length (without the complex length header byte)
                // #implicit_validation: Since the amount of leading zero bytes cannot be larger
                // than the amount of bytes in an `usize` (aka `SIZE`), both subtractions cannot
                // overflow so a `saturating_sub` is safe
                let size = SIZE.saturating_sub(len.leading_zeros() as usize / 8);
                let skip = SIZE.saturating_sub(size);

                // Write the length
                sink.write(0x80 | size as u8).propagate(e!("Failed to write length byte"))?;
                len.to_be_bytes()
                    .iter()
                    .skip(skip)
                    .try_for_each(|b| sink.write(*b).propagate(e!("Failed to write length byte")))
            }
        }
    }
}

/// An untyped DER object
#[derive(Copy, Clone)]
pub struct DerObject<'a> {
    raw: &'a [u8],
    header: &'a [u8],
    tag: u8,
    value: &'a [u8],
}
impl<'a> DerObject<'a> {
    /// Writes a new DER object with `tag` and `value` into `sink` and returns a view over it
    #[cfg_attr(feature = "no_panic", no_panic::no_panic)]
    pub fn new<S: Sink + Into<&'a [u8]>>(tag: u8, value: &[u8], sink: S) -> Result<Self, Asn1DerError> {
        Self::new_from_source(tag, value.len(), &mut value.iter(), sink)
    }
    /// Writes a new DER object with `tag`, `len` and `value` into `sink` and returns a view over it
    #[cfg_attr(feature = "no_panic", no_panic::no_panic)]
    pub fn new_from_source<A: Source, B: Sink + Into<&'a [u8]>>(
        tag: u8,
        len: usize,
        value: &mut A,
        mut sink: B,
    ) -> Result<Self, Asn1DerError> {
        Self::write(tag, len, value, &mut sink).propagate(e!("Failed to construct boolean"))?;
        DerObject::decode(sink.into()).propagate(e!("Failed to load constructed object"))
    }

    /// Decodes an object from `raw`
    #[cfg_attr(feature = "no_panic", no_panic::no_panic)]
    pub fn decode(raw: &'a [u8]) -> Result<Self, Asn1DerError> {
        Self::decode_at(raw, 0)
    }
    /// Decodes an object from `&raw[header_start..]`
    #[doc(hidden)]
    #[cfg_attr(feature = "no_panic", no_panic::no_panic)]
    pub fn decode_at(raw: &'a [u8], header_start: usize) -> Result<Self, Asn1DerError> {
        // Create iterator
        let mut value_start = header_start;
        let mut iter = raw.iter().skip(header_start).counting_source(&mut value_start);

        // Skip tag and read length
        let tag = iter.read().propagate(e!("Failed to read tag"))?;
        let len =
            length::decode(&mut iter).propagate(e!("Failed to decode length"))?.ok_or(eio!("Truncated length"))?;
        let value_end = match value_start.checked_add(len) {
            Some(value_end) => value_end,
            None => Err(eunsupported!("The object bounds would exceed `usize::max_value()`"))?,
        };

        // Get the header slice
        let header = match raw.len() {
            len if len >= value_start => &raw[..value_start],
            _ => Err(eio!("The object is truncated"))?,
        };
        let header = match header.len() {
            len if len >= header_start => &header[header_start..],
            _ => Err(eio!("The object is truncated"))?,
        };

        // Get the value slice
        let value = match raw.len() {
            len if len >= value_end => &raw[..value_end],
            _ => Err(eio!("The object is truncated"))?,
        };
        let value = match value.len() {
            len if len >= value_start => &value[value_start..],
            _ => Err(eio!("The object is truncated"))?,
        };

        Ok(Self { raw, header, tag, value })
    }
    /// Reads a DER-TLV structure from `source` by parsing the length field and copying the
    /// necessary bytes into `sink` and returns a view over it
    #[cfg_attr(feature = "no_panic", no_panic::no_panic)]
    pub fn decode_from_source<A: Source, B: Sink + Into<&'a [u8]>>(
        source: &mut A,
        mut sink: B,
    ) -> Result<Self, Asn1DerError> {
        // Create a copying iterator and copy the tag
        let mut source = source.copying_source(&mut sink);
        source.copy_next().propagate(e!("Failed to read tag"))?;

        // Read the length and copy the value
        let len =
            length::decode(&mut source).propagate(e!("Failed to decode length"))?.ok_or(eio!("Truncated length"))?;
        source.copy_n(len).propagate(e!("Failed to copy object value"))?;

        // Load the object
        Self::decode(sink.into()).propagate(e!("Failed to decode object"))
    }

    /// The underlying raw slice
    #[cfg_attr(feature = "no_panic", no_panic::no_panic)]
    pub fn raw(self) -> &'a [u8] {
        self.raw
    }
    /// The object header
    #[cfg_attr(feature = "no_panic", no_panic::no_panic)]
    pub fn header(self) -> &'a [u8] {
        self.header
    }
    /// The object tag
    #[cfg_attr(feature = "no_panic", no_panic::no_panic)]
    pub fn tag(self) -> u8 {
        self.tag
    }
    /// The object value
    #[cfg_attr(feature = "no_panic", no_panic::no_panic)]
    pub fn value(self) -> &'a [u8] {
        self.value
    }

    /// Encodes `self` to `sink`
    #[cfg_attr(feature = "no_panic", no_panic::no_panic)]
    pub fn encode<U: Sink>(&self, sink: &mut U) -> Result<(), Asn1DerError> {
        Self::write(self.tag, self.value.len(), &mut self.value.iter(), sink)
            .propagate(e!("Failed to write DER object"))
    }

    /// Writes a `tag`-`len`-`value` combination as DER-TLV structure into `sink`
    #[cfg_attr(feature = "no_panic", no_panic::no_panic)]
    pub fn write<A: Source, B: Sink>(tag: u8, len: usize, value: &mut A, sink: &mut B) -> Result<(), Asn1DerError> {
        sink.write(tag).propagate(e!("Failed to write tag"))?;
        length::encode(len, sink).propagate(e!("Failed to write length"))?;
        value.copying_source(sink).copy_n(len).propagate(e!("Failed to write value"))
    }
}
