use crate::{
    der,
    error::ErrorChain,
    typed::{DerDecodable, DerEncodable, DerTypeView},
    Asn1DerError, DerObject, Sink,
};
#[cfg(all(feature = "std", not(feature = "no_panic")))]
use core::ops::{Deref, DerefMut};

/// A counting sink that swallows each element and increments a counter
struct CountingSink(pub usize);
impl Sink for CountingSink {
    #[cfg_attr(feature = "no_panic", no_panic::no_panic)]
    fn write(&mut self, _e: u8) -> Result<(), Asn1DerError> {
        match self.0.checked_add(1) {
            Some(next) => {
                self.0 = next;
                Ok(())
            }
            None => Err(eunsupported!("Cannot write more than `usize::max_value()` bytes")),
        }
    }
}

/// An ASN.1-DER sequence view
#[derive(Copy, Clone)]
pub struct Sequence<'a> {
    object: DerObject<'a>,
}
impl<'a> Sequence<'a> {
    /// Writes a new sequence object with `objs` as subobjects into `sink` and returns a type view
    /// over it
    #[cfg_attr(feature = "no_panic", no_panic::no_panic)]
    pub fn new<S: Sink + Into<&'a [u8]>, T: DerEncodable>(objs: &[T], mut sink: S) -> Result<Self, Asn1DerError> {
        Self::write(objs, &mut sink).propagate(e!("Failed to construct sequence"))?;
        let object = DerObject::decode(sink.into()).propagate(e!("Failed to load constructed sequence"))?;
        Ok(Self { object })
    }

    /// The amount of subelements in the sequence
    ///
    /// _Note: since there is no underlying index, the amount of subelements has to be recomputed
    /// every time. If you need the length more than once, consider caching it._
    #[cfg_attr(feature = "no_panic", no_panic::no_panic)]
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        let (mut pos, mut ctr) = (0usize, 0usize);
        // #implicit_validation: Since we validate the subelements at `load`, end-of-elements is
        // the only possible error here unless the underlying object has been modified in an
        // invalid way
        while self.subobject_at(&mut pos).is_ok() {
            // #implicit_validation: The counter can never overflow an usize because this would
            // imply object lengths < 1
            ctr = ctr.saturating_add(1);
        }
        ctr
    }
    /// Gets the `n`th subobject
    ///
    /// _Note: since there is no underlying index, the position of each subelement has to be
    /// recomputed every time. If you need the subobjects more than once, consider caching them._
    #[cfg_attr(feature = "no_panic", no_panic::no_panic)]
    pub fn get(&self, n: usize) -> Result<DerObject<'a>, Asn1DerError> {
        let mut pos = 0;
        for _ in 0..n {
            // #implicit_validation: Since we validate the subelements at `load`, end-of-elements is
            // the only possible error here unless the underlying object has been modified in an
            // invalid way
            self.subobject_at(&mut pos).propagate(e!("No subobject for given index"))?;
        }
        self.subobject_at(&mut pos).propagate(e!("No subobject for given index"))
    }
    /// Gets the `n`th subobject as `T`
    ///
    /// _Note: since there is no underlying index, the position of each subelement has to be
    /// recomputed every time. If you need the subobjects more than once, consider caching them._
    #[cfg_attr(feature = "no_panic", no_panic::no_panic)]
    pub fn get_as<T: DerDecodable<'a>>(&self, n: usize) -> Result<T, Asn1DerError> {
        let object = self.get(n).propagate(e!("No subobject for given index"))?;
        T::load(object).propagate(e!("Failed to load subobject"))
    }

    /// Gets the subobject at `pos`
    #[cfg_attr(feature = "no_panic", no_panic::no_panic)]
    fn subobject_at(&self, pos: &mut usize) -> Result<DerObject<'a>, Asn1DerError> {
        // Load object
        let sequence_value = self.object.value();
        let object = DerObject::decode_at(sequence_value, *pos).propagate(e!("Failed to decode subobject"))?;
        let (object_header, object_value) = (object.header(), object.value());

        // #implicit_validation: since both slices are subslices of the same slice, their lengths
        // can never exceed `usize::max_value()`
        let len = object_header.len().saturating_add(object_value.len());
        match pos.checked_add(len) {
            Some(next_pos) => *pos = next_pos,
            None => Err(einval!("The new object cannot be as long as announced"))?,
        }
        Ok(object)
    }

    /// Writes a sequence consisting of `objs` as DER-object to `sink`
    #[cfg_attr(feature = "no_panic", no_panic::no_panic)]
    pub fn write<S: Sink, T: DerEncodable>(objs: &[T], sink: &mut S) -> Result<(), Asn1DerError> {
        // Compute the total length
        let mut ctr = CountingSink(0);
        objs.iter().try_for_each(|o| o.encode(&mut ctr).propagate(e!("Failed to size subobject")))?;

        // Encode the object by hand
        sink.write(Self::TAG).propagate(e!("Failed to write tag"))?;
        der::length::encode(ctr.0, sink).propagate(e!("Failed to encode length"))?;
        objs.iter().try_for_each(|o| o.encode(sink).propagate(e!("Failed to encode subobject")))
    }
}
impl<'a> DerTypeView<'a> for Sequence<'a> {
    const TAG: u8 = b'\x30';

    #[cfg_attr(feature = "no_panic", no_panic::no_panic)]
    fn object(&self) -> DerObject<'a> {
        self.object
    }
}
impl<'a> DerDecodable<'a> for Sequence<'a> {
    /// Loads the `Sequence` and performs a shallow validation that each underlying object is a
    /// valid DER object
    ///
    /// _Note: This function does not look "into" the underlying elements nor does it perform any
    /// type-specific validation – only the tag-length constructions are validated._
    #[cfg_attr(feature = "no_panic", no_panic::no_panic)]
    fn load(object: DerObject<'a>) -> Result<Self, Asn1DerError> {
        // Validate the tag
        let this = match object.tag() {
            Self::TAG => Self { object },
            _ => Err(einval!("DER object is not a valid sequence"))?,
        };

        // Validate the subobjects
        let (mut pos, total_len) = (0, this.object.value().len());
        while pos < total_len {
            this.subobject_at(&mut pos).propagate(e!("Invalid subobject in sequence"))?;
        }
        Ok(this)
    }
}
impl<'a> DerEncodable for Sequence<'a> {
    /// Encodes `self` to `sink`
    #[cfg_attr(feature = "no_panic", no_panic::no_panic)]
    fn encode<S: Sink>(&self, sink: &mut S) -> Result<(), Asn1DerError> {
        self.object().encode(sink).propagate(e!("Failed to encode sequence"))
    }
}

/// A newtype wrapper around `Vec` to work with sequences in a `Vec`-like way
///
/// _Note: We use a newtype wrapper here because Rust's generic type system does not allow
/// specializations, so a direct implementation for `Vec<T>` would conflict with other
/// implementations; e.g. the octet string implementation for `Vec<u8>`_
#[cfg(all(feature = "std", not(feature = "no_panic")))]
pub struct SequenceVec<T>(pub Vec<T>);
#[cfg(all(feature = "std", not(feature = "no_panic")))]
impl<T> Deref for SequenceVec<T> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
#[cfg(all(feature = "std", not(feature = "no_panic")))]
impl<T> DerefMut for SequenceVec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
#[cfg(all(feature = "std", not(feature = "no_panic")))]
impl<'a, T: DerDecodable<'a>> DerDecodable<'a> for SequenceVec<T> {
    fn load(object: DerObject<'a>) -> Result<Self, Asn1DerError> {
        let sequence = Sequence::load(object).propagate(e!("Failed to load sequence"))?;
        let objects = (0..sequence.len()).try_fold(Vec::new(), |mut vec, i| {
            let subobject: T = sequence.get_as(i).propagate(e!("Failed to load subelement"))?;
            vec.push(subobject);
            Ok(vec)
        })?;
        Ok(Self(objects))
    }
}
#[cfg(all(feature = "std", not(feature = "no_panic")))]
impl<T: DerEncodable> DerEncodable for SequenceVec<T> {
    fn encode<S: Sink>(&self, sink: &mut S) -> Result<(), Asn1DerError> {
        Sequence::write(self, sink).propagate(e!("Failed to write sequence"))
    }
}
