use crate::ringbuffer_trait::{RingBufferIntoIterator, RingBufferIterator, RingBufferMutIterator};
use crate::with_alloc::alloc_ringbuffer::RingbufferSize;
use crate::{AllocRingBuffer, RingBuffer};
use alloc::collections::VecDeque;
use core::ops::{Deref, DerefMut, Index, IndexMut};

/// A growable ringbuffer. Once capacity is reached, the size is doubled.
/// Wrapper of the built-in [`VecDeque`] struct.
///
/// The reason this is a wrapper, is that we want `RingBuffers` to implement `Index<isize>`,
/// which we cannot do for remote types like `VecDeque`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GrowableAllocRingBuffer<T>(VecDeque<T>);

impl<T, const N: usize> From<[T; N]> for GrowableAllocRingBuffer<T> {
    fn from(value: [T; N]) -> Self {
        Self(VecDeque::from(value))
    }
}

impl<T> From<VecDeque<T>> for GrowableAllocRingBuffer<T> {
    fn from(value: VecDeque<T>) -> Self {
        Self(value)
    }
}

impl<T: Clone, const N: usize> From<&[T; N]> for GrowableAllocRingBuffer<T> {
    // the cast here is actually not trivial
    #[allow(trivial_casts)]
    fn from(value: &[T; N]) -> Self {
        Self::from(value as &[T])
    }
}

impl<T: Clone> From<&[T]> for GrowableAllocRingBuffer<T> {
    fn from(value: &[T]) -> Self {
        let mut rb = Self::new();
        rb.extend(value.iter().cloned());
        rb
    }
}

impl<T, SIZE: RingbufferSize> From<AllocRingBuffer<T, SIZE>> for GrowableAllocRingBuffer<T> {
    fn from(mut v: AllocRingBuffer<T, SIZE>) -> GrowableAllocRingBuffer<T> {
        let mut rb = GrowableAllocRingBuffer::new();
        rb.extend(v.drain());
        rb
    }
}

impl<T: Clone> From<&mut [T]> for GrowableAllocRingBuffer<T> {
    fn from(value: &mut [T]) -> Self {
        Self::from(&*value)
    }
}

impl<T: Clone, const CAP: usize> From<&mut [T; CAP]> for GrowableAllocRingBuffer<T> {
    fn from(value: &mut [T; CAP]) -> Self {
        Self::from(value.clone())
    }
}

impl<T> From<alloc::vec::Vec<T>> for GrowableAllocRingBuffer<T> {
    fn from(value: alloc::vec::Vec<T>) -> Self {
        let mut res = GrowableAllocRingBuffer::new();
        res.extend(value.into_iter());
        res
    }
}

impl<T> From<alloc::collections::LinkedList<T>> for GrowableAllocRingBuffer<T> {
    fn from(value: alloc::collections::LinkedList<T>) -> Self {
        let mut res = GrowableAllocRingBuffer::new();
        res.extend(value.into_iter());
        res
    }
}

impl From<alloc::string::String> for GrowableAllocRingBuffer<char> {
    fn from(value: alloc::string::String) -> Self {
        let mut res = GrowableAllocRingBuffer::new();
        res.extend(value.chars());
        res
    }
}

impl From<&str> for GrowableAllocRingBuffer<char> {
    fn from(value: &str) -> Self {
        let mut res = GrowableAllocRingBuffer::new();
        res.extend(value.chars());
        res
    }
}

impl<T, const CAP: usize> From<crate::ConstGenericRingBuffer<T, CAP>>
    for GrowableAllocRingBuffer<T>
{
    fn from(mut value: crate::ConstGenericRingBuffer<T, CAP>) -> Self {
        let mut res = GrowableAllocRingBuffer::new();
        res.extend(value.drain());
        res
    }
}

impl<T> Deref for GrowableAllocRingBuffer<T> {
    type Target = VecDeque<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for GrowableAllocRingBuffer<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> Default for GrowableAllocRingBuffer<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> AsRef<VecDeque<T>> for GrowableAllocRingBuffer<T> {
    fn as_ref(&self) -> &VecDeque<T> {
        &self.0
    }
}

impl<T> GrowableAllocRingBuffer<T> {
    /// Creates an empty ringbuffer.
    #[must_use]
    pub fn new() -> Self {
        Self(VecDeque::new())
    }

    /// Creates an empty ringbuffer with space for at least capacity elements.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self(VecDeque::with_capacity(capacity))
    }
}

impl<T> IntoIterator for GrowableAllocRingBuffer<T> {
    type Item = T;
    type IntoIter = RingBufferIntoIterator<T, Self>;

    fn into_iter(self) -> Self::IntoIter {
        RingBufferIntoIterator::new(self)
    }
}

impl<'a, T> IntoIterator for &'a GrowableAllocRingBuffer<T> {
    type Item = &'a T;
    type IntoIter = RingBufferIterator<'a, T, GrowableAllocRingBuffer<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut GrowableAllocRingBuffer<T> {
    type Item = &'a mut T;
    type IntoIter = RingBufferMutIterator<'a, T, GrowableAllocRingBuffer<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

unsafe impl<T> RingBuffer<T> for GrowableAllocRingBuffer<T> {
    unsafe fn ptr_len(rb: *const Self) -> usize {
        (*rb).0.len()
    }

    unsafe fn ptr_capacity(rb: *const Self) -> usize {
        (*rb).0.capacity()
    }

    fn dequeue(&mut self) -> Option<T> {
        self.pop_front()
    }

    fn push(&mut self, value: T) {
        self.push_back(value);
    }

    fn fill_with<F: FnMut() -> T>(&mut self, mut f: F) {
        self.clear();
        let initial_capacity = self.0.capacity();
        for _ in 0..initial_capacity {
            self.0.push_back(f());
        }

        debug_assert_eq!(initial_capacity, self.0.capacity());
    }

    fn clear(&mut self) {
        self.0.clear();
    }

    fn get(&self, index: isize) -> Option<&T> {
        if self.is_empty() {
            None
        } else if index >= 0 {
            self.0.get(crate::mask_modulo(self.0.len(), index as usize))
        } else {
            let positive_index = -index as usize - 1;
            let masked = crate::mask_modulo(self.0.len(), positive_index);
            let index = self.0.len() - 1 - masked;

            self.0.get(index)
        }
    }

    unsafe fn ptr_get_mut(rb: *mut Self, index: isize) -> Option<*mut T> {
        #[allow(trivial_casts)]
        if RingBuffer::ptr_len(rb) == 0 {
            None
        } else if index >= 0 {
            (*rb).0.get_mut(index as usize)
        } else {
            let len = Self::ptr_len(rb);

            let positive_index = -index as usize + 1;
            let masked = crate::mask_modulo(len, positive_index);
            let index = len - 1 - masked;

            (*rb).0.get_mut(index)
        }
        .map(|i| i as *mut T)
    }

    fn get_absolute(&self, _index: usize) -> Option<&T> {
        unimplemented!()
    }

    fn get_absolute_mut(&mut self, _index: usize) -> Option<&mut T> {
        unimplemented!()
    }
}

impl<T> Extend<T> for GrowableAllocRingBuffer<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        self.0.extend(iter);
    }
}

impl<T> Index<isize> for GrowableAllocRingBuffer<T> {
    type Output = T;

    fn index(&self, index: isize) -> &Self::Output {
        self.get(index).expect("index out of bounds")
    }
}

impl<T> IndexMut<isize> for GrowableAllocRingBuffer<T> {
    fn index_mut(&mut self, index: isize) -> &mut Self::Output {
        self.get_mut(index).expect("index out of bounds")
    }
}

impl<T> FromIterator<T> for GrowableAllocRingBuffer<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self(VecDeque::from_iter(iter))
    }
}
