use core::ops::{Index, IndexMut};

use crate::ringbuffer_trait::{
    RingBuffer, RingBufferIntoIterator, RingBufferIterator, RingBufferMutIterator,
};

extern crate alloc;

// We need boxes, so depend on alloc
use crate::{mask_and, GrowableAllocRingBuffer};
use core::ptr;

/// The `AllocRingBuffer` is a `RingBuffer` which is based on a Vec. This means it allocates at runtime
/// on the heap, and therefore needs the [`alloc`] crate. This struct and therefore the dependency on
/// alloc can be disabled by disabling the `alloc` (default) feature.
///
/// # Example
/// ```
/// use ringbuffer::{AllocRingBuffer, RingBuffer};
///
/// let mut buffer = AllocRingBuffer::new(2);
///
/// // First entry of the buffer is now 5.
/// buffer.push(5);
///
/// // The last item we pushed is 5
/// assert_eq!(buffer.back(), Some(&5));
///
/// // Second entry is now 42.
/// buffer.push(42);
///
/// assert_eq!(buffer.peek(), Some(&5));
/// assert!(buffer.is_full());
///
/// // Because capacity is reached the next push will be the first item of the buffer.
/// buffer.push(1);
/// assert_eq!(buffer.to_vec(), vec![42, 1]);
/// ```
#[derive(Debug)]
pub struct AllocRingBuffer<T> {
    buf: *mut T,

    // the size of the allocation. Next power of 2 up from the capacity
    size: usize,
    // maximum number of elements actually allowed in the ringbuffer.
    // Always less than or equal than the size
    capacity: usize,

    readptr: usize,
    writeptr: usize,
}

// SAFETY: all methods that require mutable access take &mut,
// being send and sync was the old behavior but broke when we switched to *mut T.
unsafe impl<T: Sync> Sync for AllocRingBuffer<T> {}
unsafe impl<T: Send> Send for AllocRingBuffer<T> {}

impl<T, const N: usize> From<[T; N]> for AllocRingBuffer<T> {
    fn from(value: [T; N]) -> Self {
        let mut rb = Self::new(value.len());
        rb.extend(value);
        rb
    }
}

impl<T: Clone, const N: usize> From<&[T; N]> for AllocRingBuffer<T> {
    // the cast here is actually not trivial
    #[allow(trivial_casts)]
    fn from(value: &[T; N]) -> Self {
        Self::from(value as &[T])
    }
}

impl<T: Clone> From<&[T]> for AllocRingBuffer<T> {
    fn from(value: &[T]) -> Self {
        let mut rb = Self::new(value.len());
        rb.extend(value.iter().cloned());
        rb
    }
}

impl<T> From<GrowableAllocRingBuffer<T>> for AllocRingBuffer<T> {
    fn from(mut v: GrowableAllocRingBuffer<T>) -> AllocRingBuffer<T> {
        let mut rb = AllocRingBuffer::new(v.len());
        rb.extend(v.drain());
        rb
    }
}

impl<T: Clone> From<&mut [T]> for AllocRingBuffer<T> {
    fn from(value: &mut [T]) -> Self {
        Self::from(&*value)
    }
}

impl<T: Clone, const CAP: usize> From<&mut [T; CAP]> for AllocRingBuffer<T> {
    fn from(value: &mut [T; CAP]) -> Self {
        Self::from(value.clone())
    }
}

impl<T> From<alloc::vec::Vec<T>> for AllocRingBuffer<T> {
    fn from(value: alloc::vec::Vec<T>) -> Self {
        let mut res = AllocRingBuffer::new(value.len());
        res.extend(value);
        res
    }
}

impl<T> From<alloc::collections::VecDeque<T>> for AllocRingBuffer<T> {
    fn from(value: alloc::collections::VecDeque<T>) -> Self {
        let mut res = AllocRingBuffer::new(value.len());
        res.extend(value);
        res
    }
}

impl<T> From<alloc::collections::LinkedList<T>> for AllocRingBuffer<T> {
    fn from(value: alloc::collections::LinkedList<T>) -> Self {
        let mut res = AllocRingBuffer::new(value.len());
        res.extend(value);
        res
    }
}

impl From<alloc::string::String> for AllocRingBuffer<char> {
    fn from(value: alloc::string::String) -> Self {
        let mut res = AllocRingBuffer::new(value.len());
        res.extend(value.chars());
        res
    }
}

impl From<&str> for AllocRingBuffer<char> {
    fn from(value: &str) -> Self {
        let mut res = AllocRingBuffer::new(value.len());
        res.extend(value.chars());
        res
    }
}

impl<T, const CAP: usize> From<crate::ConstGenericRingBuffer<T, CAP>> for AllocRingBuffer<T> {
    fn from(mut value: crate::ConstGenericRingBuffer<T, CAP>) -> Self {
        let mut res = AllocRingBuffer::new(value.len());
        res.extend(value.drain());
        res
    }
}

impl<T> Drop for AllocRingBuffer<T> {
    fn drop(&mut self) {
        self.drain().for_each(drop);

        let layout = alloc::alloc::Layout::array::<T>(self.size).unwrap();
        unsafe {
            alloc::alloc::dealloc(self.buf as *mut u8, layout);
        }
    }
}

impl<T: Clone> Clone for AllocRingBuffer<T> {
    fn clone(&self) -> Self {
        debug_assert_ne!(self.capacity, 0);

        let mut new = Self::new(self.capacity);
        self.iter().cloned().for_each(|i| new.push(i));
        new
    }
}

impl<T: PartialEq> PartialEq for AllocRingBuffer<T> {
    fn eq(&self, other: &Self) -> bool {
        self.capacity == other.capacity
            && self.len() == other.len()
            && self.iter().zip(other.iter()).all(|(a, b)| a == b)
    }
}

impl<T: Eq + PartialEq> Eq for AllocRingBuffer<T> {}

impl<T> IntoIterator for AllocRingBuffer<T> {
    type Item = T;
    type IntoIter = RingBufferIntoIterator<T, Self>;

    fn into_iter(self) -> Self::IntoIter {
        RingBufferIntoIterator::new(self)
    }
}

impl<'a, T> IntoIterator for &'a AllocRingBuffer<T> {
    type Item = &'a T;
    type IntoIter = RingBufferIterator<'a, T, AllocRingBuffer<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut AllocRingBuffer<T> {
    type Item = &'a mut T;
    type IntoIter = RingBufferMutIterator<'a, T, AllocRingBuffer<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<T> Extend<T> for AllocRingBuffer<T> {
    fn extend<A: IntoIterator<Item = T>>(&mut self, iter: A) {
        let iter = iter.into_iter();

        for i in iter {
            self.push(i);
        }
    }
}

unsafe impl<T> RingBuffer<T> for AllocRingBuffer<T> {
    #[inline]
    unsafe fn ptr_capacity(rb: *const Self) -> usize {
        (*rb).capacity
    }

    #[inline]
    unsafe fn ptr_buffer_size(rb: *const Self) -> usize {
        (*rb).size
    }

    impl_ringbuffer!(readptr, writeptr);

    #[inline]
    fn push(&mut self, value: T) {
        if self.is_full() {
            // mask with and is allowed here because size is always a power of two
            let previous_value =
                unsafe { ptr::read(get_unchecked_mut(self, mask_and(self.size, self.readptr))) };

            // make sure we drop whatever is being overwritten
            // SAFETY: the buffer is full, so this must be initialized
            //       : also, index has been masked
            // make sure we drop because it won't happen automatically
            unsafe {
                drop(previous_value);
            }

            self.readptr += 1;
        }

        // mask with and is allowed here because size is always a power of two
        let index = mask_and(self.size, self.writeptr);

        unsafe {
            ptr::write(get_unchecked_mut(self, index), value);
        }

        self.writeptr += 1;
    }

    fn dequeue(&mut self) -> Option<T> {
        if self.is_empty() {
            None
        } else {
            // mask with and is allowed here because size is always a power of two
            let index = mask_and(self.size, self.readptr);
            let res = unsafe { get_unchecked_mut(self, index) };
            self.readptr += 1;

            // Safety: the fact that we got this maybeuninit from the buffer (with mask) means that
            // it's initialized. If it wasn't the is_empty call would have caught it. Values
            // are always initialized when inserted so this is safe.
            unsafe { Some(ptr::read(res)) }
        }
    }

    impl_ringbuffer_ext!(
        get_unchecked,
        get_unchecked_mut,
        readptr,
        writeptr,
        mask_and
    );

    #[inline]
    fn fill_with<F: FnMut() -> T>(&mut self, mut f: F) {
        self.clear();

        self.readptr = 0;
        self.writeptr = self.capacity;

        for i in 0..self.capacity {
            unsafe { ptr::write(get_unchecked_mut(self, i), f()) };
        }
    }
}

impl<T> AllocRingBuffer<T> {
    /// Creates a `AllocRingBuffer` with a certain capacity. The actual capacity is the input to the
    /// function raised to the power of two (effectively the input is the log2 of the actual capacity)
    #[inline]
    #[must_use]
    pub fn with_capacity_power_of_2(cap_power_of_two: usize) -> Self {
        Self::new(1 << cap_power_of_two)
    }

    #[inline]
    /// Alias of [`with_capacity`](AllocRingBuffer::new).
    #[must_use]
    #[deprecated = "alias of new"]
    pub fn with_capacity(cap: usize) -> Self {
        Self::new(cap)
    }

    /// Creates a `AllocRingBuffer` with a certain capacity. The capacity must not be zero.
    ///
    /// # Panics
    /// Panics when capacity is zero
    #[inline]
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        assert_ne!(capacity, 0, "Capacity must be greater than 0");
        let size = capacity.next_power_of_two();
        let layout = alloc::alloc::Layout::array::<T>(size).unwrap();
        let buf = unsafe { alloc::alloc::alloc(layout) as *mut T };
        Self {
            buf,
            size,
            capacity,
            readptr: 0,
            writeptr: 0,
        }
    }
}

/// Get a reference from the buffer without checking it is initialized.
///
/// Caller must be sure the index is in bounds, or this will panic.
#[inline]
unsafe fn get_unchecked<'a, T>(rb: *const AllocRingBuffer<T>, index: usize) -> &'a T {
    let p = (*rb).buf.add(index);
    // Safety: caller makes sure the index is in bounds for the ringbuffer.
    // All in bounds values in the ringbuffer are initialized
    &*p
}

/// Get a mut reference from the buffer without checking it is initialized.
///
/// Caller must be sure the index is in bounds, or this will panic.
#[inline]
unsafe fn get_unchecked_mut<T>(rb: *mut AllocRingBuffer<T>, index: usize) -> *mut T {
    let p = (*rb).buf.add(index);

    // Safety: caller makes sure the index is in bounds for the ringbuffer.
    // All in bounds values in the ringbuffer are initialized
    p.cast()
}

impl<T> Index<usize> for AllocRingBuffer<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).expect("index out of bounds")
    }
}

impl<T> IndexMut<usize> for AllocRingBuffer<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.get_mut(index).expect("index out of bounds")
    }
}

#[cfg(test)]
mod tests {
    use crate::{AllocRingBuffer, RingBuffer};

    // just test that this compiles
    #[test]
    fn test_generic_clone() {
        fn helper(a: &AllocRingBuffer<i32>) -> AllocRingBuffer<i32> {
            a.clone()
        }

        _ = helper(&AllocRingBuffer::new(2));
        _ = helper(&AllocRingBuffer::new(5));
    }

    #[test]
    fn test_not_power_of_two() {
        let mut rb = AllocRingBuffer::new(10);
        const NUM_VALS: usize = 1000;

        // recycle the ringbuffer a bunch of time to see if noneof the logic
        // messes up
        for _ in 0..100 {
            for i in 0..NUM_VALS {
                rb.enqueue(i);
            }
            assert!(rb.is_full());

            for i in 0..10 {
                assert_eq!(Some(i + NUM_VALS - rb.capacity()), rb.dequeue())
            }

            assert!(rb.is_empty())
        }
    }

    #[test]
    fn test_with_capacity_power_of_two() {
        let b = AllocRingBuffer::<i32>::with_capacity_power_of_2(2);
        assert_eq!(b.capacity, 4);
    }

    #[test]
    #[should_panic]
    fn test_index_zero_length() {
        let b = AllocRingBuffer::<i32>::new(2);
        let _ = b[2];
    }

    #[test]
    fn test_extend() {
        let mut buf = AllocRingBuffer::<u8>::new(4);
        (0..4).for_each(|_| buf.push(0));

        let new_data = [0, 1, 2];
        buf.extend(new_data);

        let expected = [0, 0, 1, 2];

        for i in 0..4 {
            let actual = buf[i];
            let expected = expected[i];
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn test_extend_with_overflow() {
        let mut buf = AllocRingBuffer::<u8>::new(8);
        (0..8).for_each(|_| buf.push(0));

        let new_data = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        buf.extend(new_data);

        let expected = [2, 3, 4, 5, 6, 7, 8, 9];

        for i in 0..8 {
            let actual = buf[i];
            let expected = expected[i];
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn test_conversions() {
        // from &[T]
        let data: &[i32] = &[1, 2, 3, 4];
        let buf = AllocRingBuffer::from(data);
        assert_eq!(buf.capacity, 4);
        assert_eq!(buf.to_vec(), alloc::vec![1, 2, 3, 4]);

        // from &[T; N]
        let buf = AllocRingBuffer::from(&[1, 2, 3, 4]);
        assert_eq!(buf.capacity, 4);
        assert_eq!(buf.to_vec(), alloc::vec![1, 2, 3, 4]);

        // from [T; N]
        let buf = AllocRingBuffer::from([1, 2, 3, 4]);
        assert_eq!(buf.capacity, 4);
        assert_eq!(buf.to_vec(), alloc::vec![1, 2, 3, 4]);
    }
}
