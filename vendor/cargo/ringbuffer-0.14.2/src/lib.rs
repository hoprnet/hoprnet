#![no_std]
#![deny(missing_docs)]
#![deny(warnings)]
#![deny(unused_import_braces)]
#![deny(unused_results)]
#![deny(trivial_casts)]
#![deny(trivial_numeric_casts)]
#![deny(unused_qualifications)]
#![deny(clippy::must_use_candidate)]
#![deny(clippy::default_trait_access)]
#![deny(clippy::doc_markdown)]
#![deny(clippy::semicolon_if_nothing_returned)]
#![allow(unused_unsafe)] // to support older rust versions
#![doc = include_str!("../README.md")]

#[cfg(feature = "alloc")]
extern crate alloc;

#[macro_use]
pub(crate) mod ringbuffer_trait;

pub use ringbuffer_trait::RingBuffer;

#[cfg(feature = "alloc")]
mod with_alloc;
#[cfg(feature = "alloc")]
pub use with_alloc::alloc_ringbuffer::AllocRingBuffer;
#[cfg(feature = "alloc")]
pub use with_alloc::alloc_ringbuffer::{NonPowerOfTwo, PowerOfTwo, RingbufferSize};
#[cfg(feature = "alloc")]
pub use with_alloc::vecdeque::GrowableAllocRingBuffer;

mod with_const_generics;
pub use with_const_generics::ConstGenericRingBuffer;

/// Used internally. Computes the bitmask used to properly wrap the ringbuffers.
#[inline]
const fn mask(cap: usize, index: usize) -> usize {
    index & (cap - 1)
}

/// Used internally. Computes the bitmask used to properly wrap the ringbuffers.
#[inline]
const fn mask_modulo(cap: usize, index: usize) -> usize {
    index % cap
}

#[cfg(test)]
#[allow(non_upper_case_globals)]
mod tests {
    extern crate std;

    use core::fmt::Debug;
    use std::vec;
    use std::vec::Vec;

    use crate::ringbuffer_trait::{RingBufferIterator, RingBufferMutIterator};
    use crate::{AllocRingBuffer, ConstGenericRingBuffer, GrowableAllocRingBuffer, RingBuffer};

    #[test]
    fn run_test_neg_index() {
        //! Test for issue #43

        const capacity: usize = 8;
        fn test_neg_index(mut b: impl RingBuffer<usize>) {
            for i in 0..capacity + 2 {
                b.push(i);
                assert_eq!(b.get(-1), Some(&i));
            }
        }

        test_neg_index(AllocRingBuffer::new(capacity));
        test_neg_index(ConstGenericRingBuffer::<usize, capacity>::new());
        test_neg_index(GrowableAllocRingBuffer::with_capacity(capacity));
    }

    #[test]
    fn run_test_default() {
        fn test_default(b: impl RingBuffer<i32>) {
            assert_eq!(b.capacity(), 8);
            assert_eq!(b.len(), 0);
        }

        test_default(AllocRingBuffer::new(8));
        test_default(GrowableAllocRingBuffer::with_capacity(8));
        test_default(ConstGenericRingBuffer::<i32, 8>::new());
    }

    #[test]
    fn run_test_new() {
        fn test_new(b: impl RingBuffer<i32>) {
            assert_eq!(b.capacity(), 8);
            assert_eq!(b.len(), 0);
        }

        test_new(AllocRingBuffer::new(8));
        test_new(GrowableAllocRingBuffer::with_capacity(8));
        test_new(ConstGenericRingBuffer::<i32, 8>::new());
    }

    #[test]
    fn test_default_eq_new() {
        assert_eq!(
            GrowableAllocRingBuffer::<i32>::default(),
            GrowableAllocRingBuffer::<i32>::new()
        );
        assert_eq!(
            ConstGenericRingBuffer::<i32, 8>::default(),
            ConstGenericRingBuffer::<i32, 8>::new()
        );
    }

    #[test]
    fn run_test_len() {
        fn test_len(mut b: impl RingBuffer<i32>) {
            assert_eq!(0, b.len());
            b.push(1);
            assert_eq!(1, b.len());
            b.push(2);
            assert_eq!(2, b.len())
        }

        test_len(AllocRingBuffer::new(8));
        test_len(GrowableAllocRingBuffer::with_capacity(8));
        test_len(ConstGenericRingBuffer::<i32, 8>::new());
    }

    #[test]
    fn run_test_len_wrap() {
        fn test_len_wrap(mut b: impl RingBuffer<i32>) {
            assert_eq!(0, b.len());
            b.push(1);
            assert_eq!(1, b.len());
            b.push(2);
            assert_eq!(2, b.len());
            // Now we are wrapping
            b.push(3);
            assert_eq!(2, b.len());
            b.push(4);
            assert_eq!(2, b.len());
        }

        test_len_wrap(AllocRingBuffer::new(2));
        test_len_wrap(ConstGenericRingBuffer::<i32, 2>::new());

        // the growable ringbuffer actually should grow instead of wrap
        let mut grb = GrowableAllocRingBuffer::with_capacity(2);
        assert_eq!(0, grb.len());
        grb.push(0);
        assert_eq!(1, grb.len());
        grb.push(1);
        assert_eq!(2, grb.len());
        grb.push(2);
        assert_eq!(3, grb.len());
    }

    #[test]
    fn run_test_clear() {
        fn test_clear(mut b: impl RingBuffer<i32>) {
            b.push(1);
            b.push(2);
            b.push(3);

            b.clear();
            assert!(b.is_empty());
            assert_eq!(0, b.len());
        }

        test_clear(AllocRingBuffer::new(8));
        test_clear(GrowableAllocRingBuffer::with_capacity(8));
        test_clear(ConstGenericRingBuffer::<i32, 8>::new());
    }

    #[test]
    fn run_test_empty() {
        fn test_empty(mut b: impl RingBuffer<i32>) {
            assert!(b.is_empty());
            b.push(1);
            b.push(2);
            b.push(3);
            assert!(!b.is_empty());

            b.clear();
            assert!(b.is_empty());
            assert_eq!(0, b.len());
        }

        test_empty(AllocRingBuffer::new(8));
        test_empty(GrowableAllocRingBuffer::with_capacity(8));
        test_empty(ConstGenericRingBuffer::<i32, 8>::new());
    }

    #[test]
    fn run_test_iter() {
        fn test_iter(mut b: impl RingBuffer<i32>) {
            b.push(1);
            b.push(2);
            b.push(3);
            b.push(4);
            b.push(5);
            b.push(6);
            b.push(7);

            let mut iter = b.iter();
            assert_eq!(&1, iter.next().unwrap());
            assert_eq!(&7, iter.next_back().unwrap());
            assert_eq!(&2, iter.next().unwrap());
            assert_eq!(&3, iter.next().unwrap());
            assert_eq!(&6, iter.next_back().unwrap());
            assert_eq!(&5, iter.next_back().unwrap());
            assert_eq!(&4, iter.next().unwrap());
            assert_eq!(None, iter.next());
        }

        test_iter(AllocRingBuffer::new(8));
        test_iter(GrowableAllocRingBuffer::with_capacity(8));
        test_iter(ConstGenericRingBuffer::<i32, 8>::new());
    }

    #[test]
    fn run_test_iter_ref() {
        fn test_iter<B>(mut b: B)
        where
            B: RingBuffer<i32>,
            for<'a> &'a B: IntoIterator<Item = &'a i32, IntoIter = RingBufferIterator<'a, i32, B>>,
        {
            b.push(1);
            b.push(2);
            b.push(3);
            b.push(4);
            b.push(5);
            b.push(6);
            b.push(7);

            let mut iter = (&b).into_iter();
            assert_eq!(&1, iter.next().unwrap());
            assert_eq!(&7, iter.next_back().unwrap());
            assert_eq!(&2, iter.next().unwrap());
            assert_eq!(&3, iter.next().unwrap());
            assert_eq!(&6, iter.next_back().unwrap());
            assert_eq!(&5, iter.next_back().unwrap());
            assert_eq!(&4, iter.next().unwrap());
            assert_eq!(None, iter.next());
        }

        test_iter(AllocRingBuffer::new(8));
        test_iter(GrowableAllocRingBuffer::with_capacity(8));
        test_iter(ConstGenericRingBuffer::<i32, 8>::new());
    }

    #[test]
    fn run_test_into_iter() {
        fn test_iter(mut b: impl RingBuffer<i32>) {
            b.push(1);
            b.push(2);
            b.push(3);
            b.push(4);
            b.push(5);
            b.push(6);
            b.push(7);

            let mut iter = b.into_iter();
            assert_eq!(1, iter.next().unwrap());
            assert_eq!(2, iter.next().unwrap());
            assert_eq!(3, iter.next().unwrap());
            assert_eq!(4, iter.next().unwrap());
            assert_eq!(5, iter.next().unwrap());
            assert_eq!(6, iter.next().unwrap());
            assert_eq!(7, iter.next().unwrap());
            assert_eq!(None, iter.next());
        }

        test_iter(AllocRingBuffer::new(8));
        test_iter(GrowableAllocRingBuffer::with_capacity(8));
        test_iter(ConstGenericRingBuffer::<i32, 8>::new());
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn run_test_iter_with_lifetimes() {
        fn test_iter<'a>(string: &'a str, mut b: impl RingBuffer<&'a str>) {
            b.push(&string[0..1]);
            b.push(&string[1..2]);
            b.push(&string[2..3]);

            let mut iter = b.iter();
            assert_eq!(&&string[0..1], iter.next().unwrap());
            assert_eq!(&&string[1..2], iter.next().unwrap());
            assert_eq!(&&string[2..3], iter.next().unwrap());
        }

        extern crate alloc;
        use alloc::string::ToString as _;
        let string = "abc".to_string();

        test_iter(&string, AllocRingBuffer::new(8));
        test_iter(&string, GrowableAllocRingBuffer::with_capacity(8));
        test_iter(&string, ConstGenericRingBuffer::<&str, 8>::new());
    }

    #[test]
    fn run_test_double_iter() {
        fn test_double_iter(mut b: impl RingBuffer<i32>) {
            b.push(1);
            b.push(2);
            b.push(3);

            let mut iter1 = b.iter();
            let mut iter2 = b.iter();

            assert_eq!(&1, iter1.next().unwrap());
            assert_eq!(&2, iter1.next().unwrap());
            assert_eq!(&3, iter1.next().unwrap());
            assert_eq!(&1, iter2.next().unwrap());
            assert_eq!(&2, iter2.next().unwrap());
            assert_eq!(&3, iter2.next().unwrap());
        }

        test_double_iter(AllocRingBuffer::new(8));
        test_double_iter(GrowableAllocRingBuffer::with_capacity(8));
        test_double_iter(ConstGenericRingBuffer::<i32, 8>::new());
    }

    #[test]
    fn run_test_iter_wrap() {
        fn test_iter_wrap(mut b: impl RingBuffer<i32>) {
            b.push(1);
            b.push(2);
            // Wrap
            b.push(3);

            let mut iter = b.iter();
            assert_eq!(&2, iter.next().unwrap());
            assert_eq!(&3, iter.next().unwrap());
        }

        test_iter_wrap(AllocRingBuffer::new(2));
        test_iter_wrap(ConstGenericRingBuffer::<i32, 2>::new());

        // the growable ringbuffer shouldn't actually stop growing
        let mut b = GrowableAllocRingBuffer::with_capacity(2);

        b.push(1);
        b.push(2);
        // No wrap
        b.push(3);

        let mut iter = b.iter();
        assert_eq!(&1, iter.next().unwrap());
        assert_eq!(&2, iter.next().unwrap());
        assert_eq!(&3, iter.next().unwrap());
        assert!(iter.next().is_none());
    }

    #[test]
    fn run_test_iter_mut() {
        fn test_iter_mut(mut b: impl RingBuffer<i32>) {
            b.push(1);
            b.push(2);
            b.push(3);

            for el in b.iter_mut() {
                *el += 1;
            }

            assert_eq!(vec![2, 3, 4], b.to_vec())
        }

        test_iter_mut(AllocRingBuffer::new(8));
        test_iter_mut(GrowableAllocRingBuffer::with_capacity(8));
        test_iter_mut(ConstGenericRingBuffer::<i32, 8>::new());
    }

    #[test]
    fn run_test_iter_mut_ref() {
        fn test_iter_mut<B>(mut b: B)
        where
            B: RingBuffer<i32>,
            for<'a> &'a mut B:
                IntoIterator<Item = &'a mut i32, IntoIter = RingBufferMutIterator<'a, i32, B>>,
        {
            b.push(1);
            b.push(2);
            b.push(3);

            for el in &mut b {
                *el += 1;
            }

            assert_eq!(vec![2, 3, 4], b.to_vec())
        }

        test_iter_mut(AllocRingBuffer::new(8));
        test_iter_mut(GrowableAllocRingBuffer::with_capacity(8));
        test_iter_mut(ConstGenericRingBuffer::<i32, 8>::new());
    }

    #[test]
    fn test_iter_mut_wrap() {
        fn run_test_iter_mut_wrap(mut b: impl RingBuffer<i32>) {
            b.push(1);
            b.push(2);
            b.push(3);

            for i in b.iter_mut() {
                *i += 1;
            }

            assert_eq!(vec![3, 4], b.to_vec())
        }

        run_test_iter_mut_wrap(AllocRingBuffer::new(2));
        run_test_iter_mut_wrap(ConstGenericRingBuffer::<i32, 2>::new());

        // The growable ringbuffer actually shouldn't wrap
        let mut b = GrowableAllocRingBuffer::with_capacity(2);

        b.push(1);
        b.push(2);
        b.push(3);

        for i in b.iter_mut() {
            *i += 1;
        }

        assert_eq!(vec![2, 3, 4], b.to_vec())
    }

    #[test]
    fn test_iter_mut_miri_fail() {
        fn run_test_iter_mut_wrap(mut b: impl RingBuffer<i32>) {
            b.push(1);
            b.push(2);
            b.push(3);

            let buf = b.iter_mut().collect::<Vec<_>>();

            for i in buf {
                *i += 1;
            }

            assert_eq!(vec![3, 4], b.to_vec())
        }

        run_test_iter_mut_wrap(AllocRingBuffer::new(2));
        run_test_iter_mut_wrap(ConstGenericRingBuffer::<i32, 2>::new());

        // the growable ringbuffer actually shouldn't wrap
        let mut b = GrowableAllocRingBuffer::with_capacity(2);
        b.push(1);
        b.push(2);
        b.push(3);

        let buf = b.iter_mut().collect::<Vec<_>>();

        for i in buf {
            *i += 1;
        }

        assert_eq!(vec![2, 3, 4], b.to_vec())
    }

    #[test]
    fn run_test_to_vec() {
        fn test_to_vec(mut b: impl RingBuffer<i32>) {
            b.push(1);
            b.push(2);
            b.push(3);

            assert_eq!(vec![1, 2, 3], b.to_vec())
        }

        test_to_vec(AllocRingBuffer::new(8));
        test_to_vec(GrowableAllocRingBuffer::with_capacity(8));
        test_to_vec(ConstGenericRingBuffer::<i32, 8>::new());
    }

    #[test]
    fn run_test_to_vec_wrap() {
        fn test_to_vec_wrap(mut b: impl RingBuffer<i32>) {
            b.push(1);
            b.push(2);
            // Wrap
            b.push(3);

            assert_eq!(vec![2, 3], b.to_vec())
        }

        test_to_vec_wrap(AllocRingBuffer::new(2));
        test_to_vec_wrap(ConstGenericRingBuffer::<i32, 2>::new());

        // The growable ringbuffer should actually remember all items
        let mut b = GrowableAllocRingBuffer::with_capacity(2);

        b.push(1);
        b.push(2);
        b.push(3);

        assert_eq!(vec![1, 2, 3], b.to_vec())
    }

    #[test]
    fn run_test_index() {
        fn test_index(mut b: impl RingBuffer<i32>) {
            b.push(2);
            assert_eq!(b[0], 2)
        }

        test_index(AllocRingBuffer::new(8));
        test_index(GrowableAllocRingBuffer::with_capacity(8));
        test_index(ConstGenericRingBuffer::<i32, 8>::new());
    }

    #[test]
    fn run_test_index_mut() {
        fn test_index_mut(mut b: impl RingBuffer<i32>) {
            b.push(2);

            assert_eq!(b[0], 2);

            b[0] = 5;

            assert_eq!(b[0], 5);
        }

        test_index_mut(AllocRingBuffer::new(8));
        test_index_mut(GrowableAllocRingBuffer::with_capacity(8));
        test_index_mut(ConstGenericRingBuffer::<i32, 8>::new());
    }

    #[test]
    fn run_test_peek_some() {
        fn test_peek_some(mut b: impl RingBuffer<i32>) {
            b.push(1);
            b.push(2);

            assert_eq!(b.peek(), Some(&1));
        }

        test_peek_some(AllocRingBuffer::new(2));
        test_peek_some(GrowableAllocRingBuffer::with_capacity(2));
        test_peek_some(ConstGenericRingBuffer::<i32, 2>::new());
    }

    #[test]
    fn run_test_peek_none() {
        fn test_peek_none(b: impl RingBuffer<i32>) {
            assert_eq!(b.peek(), None);
        }

        test_peek_none(AllocRingBuffer::new(8));
        test_peek_none(GrowableAllocRingBuffer::with_capacity(8));
        test_peek_none(ConstGenericRingBuffer::<i32, 8>::new());
    }

    #[test]
    fn run_test_get_relative() {
        fn test_get_relative(mut b: impl RingBuffer<i32>) {
            b.push(0);
            b.push(1);

            // [0, ...]
            //      ^
            // [0, 1, ...]
            //         ^
            // get[(index + 0) % len] = 0 (wrap to 0 because len == 2)
            // get[(index + 1) % len] = 1
            assert_eq!(b.get(0).unwrap(), &0);
            assert_eq!(b.get(1).unwrap(), &1);

            // Wraps around
            assert_eq!(b.get(2).unwrap(), &0);
            assert_eq!(b.get(3).unwrap(), &1);
        }

        test_get_relative(AllocRingBuffer::new(8));
        test_get_relative(GrowableAllocRingBuffer::with_capacity(8));
        test_get_relative(ConstGenericRingBuffer::<i32, 8>::new());
    }

    #[test]
    fn run_test_wrapping_get_relative() {
        fn test_wrapping_get_relative(mut b: impl RingBuffer<i32>) {
            b.push(0);
            b.push(1);
            b.push(2);

            // [0, ...]
            //      ^
            // [0, 1]
            //  ^
            // [2, 1]
            //     ^
            // get(0) == b[index] = 1
            // get(1) == b[(index+1) % len] = 1
            assert_eq!(b.get(0).unwrap(), &1);
            assert_eq!(b.get(1).unwrap(), &2);
        }

        test_wrapping_get_relative(AllocRingBuffer::new(2));
        test_wrapping_get_relative(ConstGenericRingBuffer::<i32, 2>::new());

        // the growable ringbuffer actually shouldn't wrap
        let mut b = GrowableAllocRingBuffer::with_capacity(2);
        b.push(0);
        b.push(1);
        b.push(2);

        assert_eq!(b.get(0).unwrap(), &0);
        assert_eq!(b.get(1).unwrap(), &1);
        assert_eq!(b.get(2).unwrap(), &2);
    }

    #[test]
    fn run_test_get_relative_zero_length() {
        fn test_get_relative_zero_length(b: impl RingBuffer<i32>) {
            assert!(b.get(1).is_none());
        }

        test_get_relative_zero_length(AllocRingBuffer::new(8));
        test_get_relative_zero_length(GrowableAllocRingBuffer::with_capacity(8));
        test_get_relative_zero_length(ConstGenericRingBuffer::<i32, 8>::new());
    }

    #[test]
    fn run_test_get_relative_mut() {
        fn test_get_relative_mut(mut b: impl RingBuffer<i32>) {
            b.push(0);
            b.push(1);

            // [0, ...]
            //      ^
            // [0, 1, ...]
            //         ^
            // get[(index + 0) % len] = 0 (wrap to 0 because len == 2)
            // get[(index + 1) % len] = 1
            *b.get_mut(0).unwrap() = 3;
            *b.get_mut(1).unwrap() = 4;

            assert_eq!(b.get(0).unwrap(), &3);
            assert_eq!(b.get(1).unwrap(), &4);
        }

        test_get_relative_mut(AllocRingBuffer::new(8));
        test_get_relative_mut(GrowableAllocRingBuffer::with_capacity(8));
        test_get_relative_mut(ConstGenericRingBuffer::<i32, 8>::new());
    }

    #[test]
    fn run_test_wrapping_get_relative_mut() {
        fn test_wrapping_get_relative_mut(mut b: impl RingBuffer<i32>) {
            b.push(0);
            b.push(1);
            b.push(2);

            *b.get_mut(0).unwrap() = 3;

            // [0, ...]
            //      ^
            // [0, 1]
            //  ^
            // [2, 1]
            //     ^
            // get(0) == b[index] = 1
            // get(1) == b[(index+1) % len] = 1
            assert_eq!(b.get(0).unwrap(), &3);
            assert_eq!(b.get(1).unwrap(), &2);
        }

        test_wrapping_get_relative_mut(AllocRingBuffer::new(2));
        test_wrapping_get_relative_mut(ConstGenericRingBuffer::<i32, 2>::new());

        // the growable ringbuffer actually shouldn't wrap
        let mut b = GrowableAllocRingBuffer::with_capacity(2);

        b.push(0);
        b.push(1);
        b.push(2);

        *b.get_mut(0).unwrap() = 3;

        assert_eq!(b.get(0).unwrap(), &3);
        assert_eq!(b.get(1).unwrap(), &1);
        assert_eq!(b.get(2).unwrap(), &2);
    }

    #[test]
    fn run_test_get_relative_mut_zero_length() {
        fn test_get_relative_mut_zero_length(mut b: impl RingBuffer<i32>) {
            assert!(b.get_mut(1).is_none());
        }

        test_get_relative_mut_zero_length(AllocRingBuffer::new(8));
        test_get_relative_mut_zero_length(GrowableAllocRingBuffer::with_capacity(8));
        test_get_relative_mut_zero_length(ConstGenericRingBuffer::<i32, 8>::new());
    }

    #[test]
    #[allow(deprecated)]
    fn run_test_get_absolute() {
        fn test_get_absolute(mut b: impl RingBuffer<i32>) {
            b.push(0);
            b.push(1);

            // [0, ...]
            //      ^
            // [0, 1, ...]
            //         ^
            // get[0] = 0
            // get[1] = 1
            assert_eq!(b.get_absolute(0).unwrap(), &0);
            assert_eq!(b.get_absolute(1).unwrap(), &1);
            assert!(b.get_absolute(2).is_none());
        }

        test_get_absolute(AllocRingBuffer::with_capacity(8));
        test_get_absolute(ConstGenericRingBuffer::<i32, 8>::new());
    }

    #[test]
    fn run_test_from_iterator() {
        fn test_from_iterator<T: RingBuffer<i32> + FromIterator<i32>>() {
            let b: T = std::iter::repeat(1).take(1024).collect();
            assert_eq!(b.len(), 1024);
            assert_eq!(b.to_vec(), vec![1; 1024])
        }

        test_from_iterator::<GrowableAllocRingBuffer<i32>>();
        test_from_iterator::<ConstGenericRingBuffer<i32, 1024>>();
    }

    #[test]
    fn run_test_from_iterator_wrap() {
        fn test_from_iterator_wrap<T: RingBuffer<i32> + FromIterator<i32>>() {
            let b: T = std::iter::repeat(1).take(8000).collect();
            assert_eq!(b.len(), b.capacity());
            assert_eq!(b.to_vec(), vec![1; b.capacity()])
        }

        test_from_iterator_wrap::<GrowableAllocRingBuffer<i32>>();
        test_from_iterator_wrap::<ConstGenericRingBuffer<i32, 1024>>();
    }

    #[test]
    fn run_test_get_relative_negative() {
        fn test_get_relative_negative(mut b: impl RingBuffer<i32>) {
            b.push(0);
            b.push(1);

            // [0, ...]
            //      ^
            // [0, 1, ...]
            //         ^
            // get[(index + -1) % len] = 1
            // get[(index + -2) % len] = 0 (wrap to 1 because len == 2)
            assert_eq!(b.get(-1).unwrap(), &1);
            assert_eq!(b.get(-2).unwrap(), &0);

            // Wraps around
            assert_eq!(b.get(-3).unwrap(), &1);
            assert_eq!(b.get(-4).unwrap(), &0);
        }

        test_get_relative_negative(AllocRingBuffer::new(8));
        test_get_relative_negative(ConstGenericRingBuffer::<i32, 8>::new());
    }

    #[test]
    fn run_test_contains() {
        fn test_contains(mut b: impl RingBuffer<i32>) {
            b.push(1);
            b.push(2);

            assert!(b.contains(&1));
            assert!(b.contains(&2));
        }

        test_contains(AllocRingBuffer::new(8));
        test_contains(GrowableAllocRingBuffer::with_capacity(8));
        test_contains(ConstGenericRingBuffer::<i32, 8>::new());
    }

    #[test]
    fn run_test_is_full() {
        fn test_is_full(mut b: impl RingBuffer<i32>) {
            assert!(!b.is_full());
            b.push(1);
            assert!(!b.is_full());
            b.push(2);
            assert!(b.is_full());
        }

        test_is_full(AllocRingBuffer::new(2));
        test_is_full(GrowableAllocRingBuffer::with_capacity(2));
        test_is_full(ConstGenericRingBuffer::<i32, 2>::new());
    }

    #[test]
    fn run_test_front_some() {
        fn test_front_some(mut b: impl RingBuffer<i32>) {
            b.push(1);
            b.push(2);

            assert_eq!(b.front(), Some(&1));
        }

        test_front_some(AllocRingBuffer::new(2));
        test_front_some(GrowableAllocRingBuffer::with_capacity(2));
        test_front_some(ConstGenericRingBuffer::<i32, 2>::new());
    }

    #[test]
    fn run_test_front_none() {
        fn test_front_none(b: impl RingBuffer<i32>) {
            assert_eq!(b.front(), None);
        }

        test_front_none(AllocRingBuffer::new(8));
        test_front_none(GrowableAllocRingBuffer::with_capacity(8));
        test_front_none(ConstGenericRingBuffer::<i32, 8>::new());
    }

    #[test]
    fn run_test_back_some() {
        fn test_back_some(mut b: impl RingBuffer<i32>) {
            b.push(1);
            b.push(2);

            assert_eq!(b.back(), Some(&2));
        }

        test_back_some(AllocRingBuffer::new(2));
        test_back_some(GrowableAllocRingBuffer::with_capacity(2));
        test_back_some(ConstGenericRingBuffer::<i32, 2>::new());
    }

    #[test]
    fn run_test_back_none() {
        fn test_back_none(b: impl RingBuffer<i32>) {
            assert_eq!(b.back(), None);
        }

        test_back_none(AllocRingBuffer::new(8));
        test_back_none(GrowableAllocRingBuffer::with_capacity(8));
        test_back_none(ConstGenericRingBuffer::<i32, 8>::new());
    }

    #[test]
    fn run_test_front_some_mut() {
        fn test_front_some_mut(mut b: impl RingBuffer<i32>) {
            b.push(1);
            b.push(2);

            assert_eq!(b.front_mut(), Some(&mut 1));
        }

        test_front_some_mut(AllocRingBuffer::new(2));
        test_front_some_mut(GrowableAllocRingBuffer::with_capacity(2));
        test_front_some_mut(ConstGenericRingBuffer::<i32, 2>::new());
    }

    #[test]
    fn run_test_front_none_mut() {
        fn test_front_none_mut(mut b: impl RingBuffer<i32>) {
            assert_eq!(b.front_mut(), None);
        }

        test_front_none_mut(AllocRingBuffer::new(8));
        test_front_none_mut(GrowableAllocRingBuffer::with_capacity(8));
        test_front_none_mut(ConstGenericRingBuffer::<i32, 8>::new());
    }

    #[test]
    fn run_test_back_some_mut() {
        fn test_back_some_mut(mut b: impl RingBuffer<i32>) {
            b.push(1);
            b.push(2);

            assert_eq!(b.back_mut(), Some(&mut 2));
        }

        test_back_some_mut(AllocRingBuffer::new(2));
        test_back_some_mut(GrowableAllocRingBuffer::with_capacity(2));
        test_back_some_mut(ConstGenericRingBuffer::<i32, 2>::new());
    }

    #[test]
    fn run_test_back_none_mut() {
        fn test_back_none_mut(mut b: impl RingBuffer<i32>) {
            assert_eq!(b.back_mut(), None);
        }

        test_back_none_mut(AllocRingBuffer::new(8));
        test_back_none_mut(GrowableAllocRingBuffer::with_capacity(8));
        test_back_none_mut(ConstGenericRingBuffer::<i32, 8>::new());
    }

    #[test]
    fn run_test_dequeue() {
        fn run_test_dequeue(mut b: impl RingBuffer<i32>) {
            b.push(0);
            b.push(1);

            assert_eq!(b.len(), 2);

            assert_eq!(b.dequeue(), Some(0));
            assert_eq!(b.dequeue(), Some(1));

            assert_eq!(b.len(), 0);

            assert_eq!(b.dequeue(), None);
        }

        run_test_dequeue(AllocRingBuffer::new(8));
        run_test_dequeue(GrowableAllocRingBuffer::with_capacity(8));
        run_test_dequeue(ConstGenericRingBuffer::<i32, 8>::new());
    }

    #[test]
    fn run_test_skip() {
        fn test_skip(mut b: impl RingBuffer<i32>) {
            b.push(0);
            b.push(1);

            assert_eq!(b.len(), 2);

            b.skip();
            b.skip();

            assert_eq!(b.len(), 0)
        }

        test_skip(AllocRingBuffer::new(8));
        test_skip(GrowableAllocRingBuffer::with_capacity(8));
        test_skip(ConstGenericRingBuffer::<i32, 8>::new());
    }

    #[test]
    fn run_test_skip_2() {
        fn test_skip2(mut rb: impl RingBuffer<i32>) {
            rb.skip();
            rb.skip();
            rb.skip();
            rb.push(1);
            assert_eq!(rb.dequeue(), Some(1));
            assert_eq!(rb.dequeue(), None);
            rb.skip();
            assert_eq!(rb.dequeue(), None);
        }

        test_skip2(AllocRingBuffer::new(2));
        test_skip2(GrowableAllocRingBuffer::with_capacity(2));
        test_skip2(ConstGenericRingBuffer::<i32, 2>::new());
    }

    #[test]
    fn run_test_push_dequeue_push() {
        fn test_push_dequeue_push(mut b: impl RingBuffer<i32>) {
            b.push(0);
            b.push(1);

            assert_eq!(b.dequeue(), Some(0));
            assert_eq!(b.dequeue(), Some(1));
            assert_eq!(b.dequeue(), None);

            b.push(0);
            b.push(1);

            assert_eq!(b.dequeue(), Some(0));
            assert_eq!(b.dequeue(), Some(1));
            assert_eq!(b.dequeue(), None);
        }

        test_push_dequeue_push(AllocRingBuffer::new(8));
        test_push_dequeue_push(GrowableAllocRingBuffer::with_capacity(8));
        test_push_dequeue_push(ConstGenericRingBuffer::<i32, 8>::new());
    }

    #[test]
    fn run_test_enqueue_dequeue_push() {
        fn test_enqueue_dequeue_push(mut b: impl RingBuffer<i32>) {
            b.enqueue(0);
            b.enqueue(1);

            assert_eq!(b.dequeue(), Some(0));
            assert_eq!(b.dequeue(), Some(1));
            assert_eq!(b.dequeue(), None);

            b.enqueue(0);
            b.enqueue(1);

            assert_eq!(b.dequeue(), Some(0));
            assert_eq!(b.dequeue(), Some(1));
            assert_eq!(b.dequeue(), None);
        }

        test_enqueue_dequeue_push(AllocRingBuffer::new(8));
        test_enqueue_dequeue_push(GrowableAllocRingBuffer::with_capacity(8));
        test_enqueue_dequeue_push(ConstGenericRingBuffer::<i32, 8>::new());
    }

    #[test]
    fn large_negative_index() {
        fn test_large_negative_index(mut b: impl RingBuffer<i32>) {
            b.push(1);
            b.push(2);
            assert_eq!(b.get(1), Some(&2));
            assert_eq!(b.get(0), Some(&1));
            assert_eq!(b.get(-1), Some(&2));
            assert_eq!(b.get(-2), Some(&1));
            assert_eq!(b.get(-3), Some(&2));
        }

        test_large_negative_index(AllocRingBuffer::new(2));
        test_large_negative_index(ConstGenericRingBuffer::<i32, 2>::new());
        test_large_negative_index(GrowableAllocRingBuffer::<i32>::new());
    }

    #[test]
    fn large_negative_index_mut() {
        fn test_large_negative_index(mut b: impl RingBuffer<i32>) {
            b.push(1);
            b.push(2);
            assert_eq!(b.get_mut(1), Some(&mut 2));
            assert_eq!(b.get_mut(0), Some(&mut 1));
            assert_eq!(b.get_mut(-1), Some(&mut 2));
            assert_eq!(b.get_mut(-2), Some(&mut 1));
            assert_eq!(b.get_mut(-3), Some(&mut 2));
        }

        test_large_negative_index(AllocRingBuffer::new(2));
        test_large_negative_index(ConstGenericRingBuffer::<i32, 2>::new());
        test_large_negative_index(GrowableAllocRingBuffer::<i32>::new());
    }

    #[test]
    fn run_test_push_dequeue_push_full() {
        fn test_push_dequeue_push_full(mut b: impl RingBuffer<i32>) {
            b.push(0);
            b.push(1);
            b.push(2);

            assert_eq!(b.dequeue(), Some(1));
            assert_eq!(b.dequeue(), Some(2));
            assert_eq!(b.dequeue(), None);

            b.push(0);
            b.push(1);
            b.push(2);

            assert_eq!(b.dequeue(), Some(1));
            assert_eq!(b.dequeue(), Some(2));
            assert_eq!(b.dequeue(), None);
        }

        test_push_dequeue_push_full(AllocRingBuffer::new(2));
        test_push_dequeue_push_full(ConstGenericRingBuffer::<i32, 2>::new());

        // the growable ringbuffer should actually keep growing and dequeue all items
        let mut b = GrowableAllocRingBuffer::with_capacity(2);
        b.push(0);
        b.push(1);
        b.push(2);

        assert_eq!(b.dequeue(), Some(0));
        assert_eq!(b.dequeue(), Some(1));
        assert_eq!(b.dequeue(), Some(2));
        assert_eq!(b.dequeue(), None);

        b.push(0);
        b.push(1);
        b.push(2);

        assert_eq!(b.dequeue(), Some(0));
        assert_eq!(b.dequeue(), Some(1));
        assert_eq!(b.dequeue(), Some(2));
        assert_eq!(b.dequeue(), None);
    }

    #[test]
    fn run_test_push_dequeue_push_full_get() {
        fn test_push_dequeue_push_full_get(mut b: impl RingBuffer<i32>) {
            b.push(0);
            b.push(1);
            b.push(2);

            assert_eq!(b.dequeue(), Some(1));
            assert_eq!(b.dequeue(), Some(2));
            assert_eq!(b.dequeue(), None);

            b.push(0);
            b.push(1);
            b.push(2);

            assert_eq!(b.dequeue(), Some(1));
            assert_eq!(b.dequeue(), Some(2));
            assert_eq!(b.dequeue(), None);

            b.push(0);
            b.push(1);
            b.push(2);

            assert_eq!(b.get(-1), Some(&2));
            assert_eq!(b.get(-2), Some(&1));
            assert_eq!(b.get(-3), Some(&2));
        }

        test_push_dequeue_push_full_get(AllocRingBuffer::new(2));
        test_push_dequeue_push_full_get(ConstGenericRingBuffer::<i32, 2>::new());

        // the growable ringbuffer should actually keep growing and dequeue all items
        let mut b = GrowableAllocRingBuffer::with_capacity(2);

        b.push(0);
        b.push(1);
        b.push(2);

        assert_eq!(b.dequeue(), Some(0));
        assert_eq!(b.dequeue(), Some(1));
        assert_eq!(b.dequeue(), Some(2));
        assert_eq!(b.dequeue(), None);

        b.push(0);
        b.push(1);
        b.push(2);

        assert_eq!(b.dequeue(), Some(0));
        assert_eq!(b.dequeue(), Some(1));
        assert_eq!(b.dequeue(), Some(2));
        assert_eq!(b.dequeue(), None);

        b.push(0);
        b.push(1);
        b.push(2);

        assert_eq!(b.get(-1), Some(&2));
        assert_eq!(b.get(-2), Some(&1));
        assert_eq!(b.get(-3), Some(&0))
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    // this test takes far too long with Miri enabled
    fn run_test_push_dequeue_push_full_get_rep() {
        fn test_push_dequeue_push_full_get_rep(mut rb: impl RingBuffer<i32>) {
            for _ in 0..100_000 {
                rb.push(1);
                rb.push(2);

                assert_eq!(rb.dequeue(), Some(1));
                assert_eq!(rb.dequeue(), Some(2));

                rb.push(1);
                rb.push(2);

                assert_eq!(rb.dequeue(), Some(1));
                assert_eq!(rb.dequeue(), Some(2));

                rb.push(1);
                rb.push(2);

                assert_eq!(rb.get(-1), Some(&2));
                assert_eq!(rb.get(-2), Some(&1));
            }
        }

        test_push_dequeue_push_full_get_rep(AllocRingBuffer::new(8));
        test_push_dequeue_push_full_get_rep(GrowableAllocRingBuffer::with_capacity(8));
        test_push_dequeue_push_full_get_rep(ConstGenericRingBuffer::<i32, 8>::new());
    }

    #[test]
    fn run_test_clone() {
        fn test_clone(mut rb: impl RingBuffer<i32> + Clone + Eq + Debug) {
            rb.push(42);
            rb.push(32);
            rb.push(22);

            let mut other = rb.clone();

            assert_eq!(rb, other);

            rb.push(11);
            rb.push(12);
            other.push(11);
            other.push(12);

            assert_eq!(rb, other);
        }

        test_clone(AllocRingBuffer::new(4));
        test_clone(GrowableAllocRingBuffer::with_capacity(4));
        test_clone(ConstGenericRingBuffer::<i32, 4>::new());
    }

    #[test]
    fn run_test_default_fill() {
        fn test_default_fill(mut rb: impl RingBuffer<i32>) {
            for i in 0..rb.capacity() {
                for _ in 0..i {
                    rb.push(1);
                }

                assert_eq!(rb.len(), i);
                rb.fill_default();
                assert_eq!(rb.len(), 4);

                // 4x
                assert_eq!(rb.dequeue(), Some(0));
                assert_eq!(rb.dequeue(), Some(0));
                assert_eq!(rb.dequeue(), Some(0));
                assert_eq!(rb.dequeue(), Some(0));
            }
        }

        test_default_fill(AllocRingBuffer::new(4));
        test_default_fill(GrowableAllocRingBuffer::with_capacity(4));
        test_default_fill(ConstGenericRingBuffer::<i32, 4>::new());
    }

    #[test]
    fn run_test_eq() {
        let mut alloc_a = ConstGenericRingBuffer::<i32, 4>::new();
        let mut alloc_b = ConstGenericRingBuffer::<i32, 4>::new();

        assert!(alloc_a.eq(&alloc_b));
        alloc_a.push(1);
        assert!(!alloc_b.eq(&alloc_a));
        alloc_b.push(1);
        assert!(alloc_a.eq(&alloc_b));
        alloc_a.push(4);
        alloc_b.push(2);
        assert!(!alloc_b.eq(&alloc_a));
    }

    #[test]
    fn run_next_back_test() {
        fn next_back_test(mut rb: impl RingBuffer<i32>) {
            for i in 1..=4 {
                rb.push(i);
            }

            let mut it = rb.iter();
            assert_eq!(Some(&4), it.next_back());
            assert_eq!(Some(&3), it.next_back());
            assert_eq!(Some(&1), it.next());
            assert_eq!(Some(&2), it.next_back());
            assert_eq!(None, it.next_back());
        }

        next_back_test(ConstGenericRingBuffer::<i32, 8>::new());
        next_back_test(AllocRingBuffer::new(8));
        next_back_test(GrowableAllocRingBuffer::with_capacity(8));
    }

    #[test]
    fn run_next_back_test_mut() {
        fn next_back_test_mut(mut rb: impl RingBuffer<i32>) {
            for i in 1..=4 {
                rb.push(i);
            }

            let mut it = rb.iter_mut();
            assert_eq!(Some(&mut 4), it.next_back());
            assert_eq!(Some(&mut 3), it.next_back());
            assert_eq!(Some(&mut 1), it.next());
            assert_eq!(Some(&mut 2), it.next_back());
            assert_eq!(None, it.next_back());
        }

        next_back_test_mut(ConstGenericRingBuffer::<i32, 8>::new());
        next_back_test_mut(AllocRingBuffer::new(8));
        next_back_test_mut(GrowableAllocRingBuffer::with_capacity(8));
    }
    #[test]
    fn run_test_fill() {
        fn test_fill(mut rb: impl RingBuffer<i32>) {
            for i in 0..rb.capacity() {
                for _ in 0..i {
                    rb.push(1);
                }

                assert_eq!(rb.len(), i);
                rb.fill(3);
                assert_eq!(rb.len(), 4);

                // 4x
                assert_eq!(rb.dequeue(), Some(3));
                assert_eq!(rb.dequeue(), Some(3));
                assert_eq!(rb.dequeue(), Some(3));
                assert_eq!(rb.dequeue(), Some(3));
            }
        }

        test_fill(AllocRingBuffer::new(4));
        test_fill(GrowableAllocRingBuffer::with_capacity(4));
        test_fill(ConstGenericRingBuffer::<i32, 4>::new());
    }

    mod test_dropping {
        use super::*;
        use std::boxed::Box;
        use std::cell::{RefCell, RefMut};

        struct DropTest {
            flag: bool,
        }

        struct Dropee<'a> {
            parent: Option<RefMut<'a, DropTest>>,
        }

        impl<'a> Drop for Dropee<'a> {
            fn drop(&mut self) {
                if let Some(parent) = &mut self.parent {
                    parent.flag = true;
                }
            }
        }

        macro_rules! test_dropped {
            ($constructor: block) => {{
                let dt = Box::into_raw(Box::new(RefCell::new(DropTest { flag: false })));
                {
                    let d = Dropee {
                        // Safety:
                        // We know the pointer is initialized as it was created just above.
                        // Also no other mutable borrow can exist at this time
                        parent: Some(unsafe { dt.as_ref() }.unwrap().borrow_mut()),
                    };
                    let mut rb = { $constructor };
                    rb.push(d);
                    rb.push(Dropee { parent: None });
                }
                {
                    // Safety:
                    // We know the pointer exists and is no longer borrowed as the block above limited it
                    assert!(unsafe { dt.as_ref() }.unwrap().borrow().flag);
                }
                // Safety:
                // No other references exist to box so we can safely drop it
                unsafe {
                    drop(Box::from_raw(dt));
                }
            }};
        }

        #[test]
        fn run_test_drops_contents_alloc() {
            test_dropped!({ AllocRingBuffer::new(1) });
        }

        #[test]
        fn run_test_drops_contents_const_generic() {
            test_dropped!({ ConstGenericRingBuffer::<_, 1>::new() });
        }

        #[test]
        fn run_test_drops_contents_growable_alloc() {
            test_dropped!({ GrowableAllocRingBuffer::with_capacity(1) });
        }
    }

    #[test]
    fn test_clone() {
        macro_rules! test_clone {
            ($e: expr) => {
                let mut e1 = $e;
                e1.push(1);
                e1.push(2);

                let mut e2 = e1.clone();

                e2.push(11);
                e2.push(12);

                assert_eq!(e1.to_vec(), vec![1, 2]);
                assert_eq!(e2.to_vec(), vec![1, 2, 11, 12]);
            };
        }

        test_clone!(ConstGenericRingBuffer::<_, 4>::new());
        test_clone!(GrowableAllocRingBuffer::<_>::new());
        test_clone!(AllocRingBuffer::<_, _>::new(4));
    }
}
