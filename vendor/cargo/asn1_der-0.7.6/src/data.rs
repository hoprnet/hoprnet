use crate::{error::ErrorChain, Asn1DerError};
use core::{iter, slice};

/// A trait defining a byte source
pub trait Source: Sized {
    /// Reads the next element
    fn read(&mut self) -> Result<u8, Asn1DerError>;

    /// Creates a counting source
    #[cfg_attr(feature = "no_panic", no_panic::no_panic)]
    fn counting_source(self, ctr: &mut usize) -> CountingSource<Self> {
        CountingSource { source: self, ctr }
    }
    /// Creates a copying source
    #[cfg_attr(feature = "no_panic", no_panic::no_panic)]
    fn copying_source<U: Sink>(self, sink: U) -> CopyingSource<Self, U> {
        CopyingSource { source: self, sink }
    }
}
impl<S: Source> Source for &mut S {
    #[cfg_attr(feature = "no_panic", no_panic::no_panic)]
    fn read(&mut self) -> Result<u8, Asn1DerError> {
        (*self).read()
    }
}
impl<'a> Source for slice::Iter<'a, u8> {
    #[cfg_attr(feature = "no_panic", no_panic::no_panic)]
    fn read(&mut self) -> Result<u8, Asn1DerError> {
        match self.next() {
            Some(e) => Ok(*e),
            None => Err(eio!("Cannot read beyond end of slice-iterator")),
        }
    }
}
impl<'a, A: Iterator<Item = &'a u8>, B: Iterator<Item = &'a u8>> Source for iter::Chain<A, B> {
    #[cfg_attr(feature = "no_panic", no_panic::no_panic)]
    fn read(&mut self) -> Result<u8, Asn1DerError> {
        match self.next() {
            Some(e) => Ok(*e),
            None => Err(eio!("Cannot read beyond end of chain-iterator")),
        }
    }
}
impl<'a, I: Iterator<Item = &'a u8> + 'a> Source for iter::Skip<I> {
    #[cfg_attr(feature = "no_panic", no_panic::no_panic)]
    fn read(&mut self) -> Result<u8, Asn1DerError> {
        match self.next() {
            Some(e) => Ok(*e),
            None => Err(eio!("Cannot read beyond end of chain-iterator")),
        }
    }
}

/// A source that counts the amount of elements read
///
/// __Warning: if a call to `read` would cause `ctr` to exceed `usize::max_value()`, this source
/// will return an error and the element that has been read from the underlying source will be
/// lost__
pub struct CountingSource<'a, S: Source> {
    source: S,
    ctr: &'a mut usize,
}
impl<'a, S: Source> Source for CountingSource<'a, S> {
    #[cfg_attr(feature = "no_panic", no_panic::no_panic)]
    fn read(&mut self) -> Result<u8, Asn1DerError> {
        let e = self.source.read().propagate(e!("Failed to read element from underlying source"))?;
        match self.ctr.checked_add(1) {
            Some(ctr_next) => {
                *self.ctr = ctr_next;
                Ok(e)
            }
            _ => Err(eio!("Cannot read more because the position counter would overflow")),
        }
    }
}

/// A source that also copies each read element to the `sink`
///
/// __Warning: if a call to `write` fails, this source will return an error and the element that has
/// been read from the underlying source will be lost__
pub struct CopyingSource<S: Source, U: Sink> {
    source: S,
    sink: U,
}
impl<S: Source, U: Sink> CopyingSource<S, U> {
    /// Copies the next element
    #[cfg_attr(feature = "no_panic", no_panic::no_panic)]
    pub fn copy_next(&mut self) -> Result<(), Asn1DerError> {
        self.read()?;
        Ok(())
    }
    /// Copies the next `n` elements
    #[cfg_attr(feature = "no_panic", no_panic::no_panic)]
    pub fn copy_n(&mut self, n: usize) -> Result<(), Asn1DerError> {
        (0..n).try_for_each(|_| self.copy_next())
    }
}
impl<S: Source, U: Sink> Source for CopyingSource<S, U> {
    #[cfg_attr(feature = "no_panic", no_panic::no_panic)]
    fn read(&mut self) -> Result<u8, Asn1DerError> {
        let e = self.source.read().propagate(e!("Failed to read element from underlying source"))?;
        self.sink.write(e).propagate(e!("Failed to copy element to sink"))?;
        Ok(e)
    }
}

/// A trait defining a byte sink
pub trait Sink: Sized {
    /// Writes `e` to `self`
    fn write(&mut self, e: u8) -> Result<(), Asn1DerError>;
    /// Creates a counting sink
    #[cfg_attr(feature = "no_panic", no_panic::no_panic)]
    fn counting_sink(self, ctr: &mut usize) -> CountingSink<Self> {
        CountingSink { sink: self, ctr }
    }
}
impl<S: Sink> Sink for &mut S {
    #[cfg_attr(feature = "no_panic", no_panic::no_panic)]
    fn write(&mut self, e: u8) -> Result<(), Asn1DerError> {
        (*self).write(e)
    }
}
impl<'a> Sink for slice::IterMut<'a, u8> {
    #[cfg_attr(feature = "no_panic", no_panic::no_panic)]
    fn write(&mut self, e: u8) -> Result<(), Asn1DerError> {
        match self.next() {
            Some(t) => {
                *t = e;
                Ok(())
            }
            None => Err(eio!("Cannot write beyond end of slice-iterator")),
        }
    }
}
#[cfg(all(feature = "std", not(feature = "no_panic")))]
impl Sink for Vec<u8> {
    fn write(&mut self, e: u8) -> Result<(), Asn1DerError> {
        self.push(e);
        Ok(())
    }
}

/// A sink that counts the amount of elements written
///
/// __Warning: if a call to `write` would cause `ctr` to exceed `usize::max_value()`, this sink
/// will return an error but the element is written nonetheless__
pub struct CountingSink<'a, S: Sink> {
    sink: S,
    ctr: &'a mut usize,
}
impl<'a, S: Sink> Sink for CountingSink<'a, S> {
    #[cfg_attr(feature = "no_panic", no_panic::no_panic)]
    fn write(&mut self, e: u8) -> Result<(), Asn1DerError> {
        self.sink.write(e).propagate(e!("Failed to write element to underlying source"))?;
        *self.ctr = match self.ctr.checked_add(1) {
            Some(ctr_next) => ctr_next,
            None => Err(eio!("Cannot write more because the position counter would overflow"))?,
        };
        Ok(())
    }
}

/// A slice-backed sink
pub struct SliceSink<'a> {
    slice: &'a mut [u8],
    pos: &'a mut usize,
}
impl<'a> SliceSink<'a> {
    /// Creates a new `SliceSink` with `slice` as backing and `pos` as position counter
    #[cfg_attr(feature = "no_panic", no_panic::no_panic)]
    pub fn new(slice: &'a mut [u8], pos: &'a mut usize) -> Self {
        Self { slice, pos }
    }
}
impl<'a> Sink for SliceSink<'a> {
    #[cfg_attr(feature = "no_panic", no_panic::no_panic)]
    fn write(&mut self, e: u8) -> Result<(), Asn1DerError> {
        match self.slice.get_mut(*self.pos) {
            Some(t) => match self.pos.checked_add(1) {
                Some(pos_next) => {
                    *self.pos = pos_next;
                    *t = e;
                    Ok(())
                }
                None => Err(eio!("Cannot write more because the position counter would overflow"))?,
            },
            None => Err(eio!("Cannot write beyond the end-of-slice"))?,
        }
    }
}
impl<'a> From<SliceSink<'a>> for &'a [u8] {
    #[cfg_attr(feature = "no_panic", no_panic::no_panic)]
    fn from(sink: SliceSink<'a>) -> Self {
        match sink.slice.len() {
            len if *sink.pos < len => &sink.slice[..*sink.pos],
            _ => sink.slice,
        }
    }
}

/// A newtype wrapper around a `&'a mut Vec<u8>` that implements `Sink` and `Into<&'a [u8]>`
#[cfg(all(feature = "std", not(feature = "no_panic")))]
pub struct VecBacking<'a>(pub &'a mut Vec<u8>);
#[cfg(all(feature = "std", not(feature = "no_panic")))]
impl<'a> Sink for VecBacking<'a> {
    fn write(&mut self, e: u8) -> Result<(), Asn1DerError> {
        self.0.push(e);
        Ok(())
    }
}
#[cfg(all(feature = "std", not(feature = "no_panic")))]
impl<'a> From<VecBacking<'a>> for &'a [u8] {
    fn from(backing: VecBacking<'a>) -> &'a [u8] {
        backing.0.as_slice()
    }
}
