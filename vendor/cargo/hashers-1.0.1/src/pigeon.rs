//! # Hash functions by Steven Pigeon (https://hbfs.wordpress.com/)
//!
//! From:
//!
//! * https://hbfs.wordpress.com/2015/09/29/hash-functions-part-i/
//! * https://hbfs.wordpress.com/2015/10/06/the-anatomy-of-hash-functions-hash-functions-part-ii/
//! * https://hbfs.wordpress.com/2015/10/13/testing-hash-functions-hash-functions-part-iii/
//! * https://hbfs.wordpress.com/2015/10/20/three-bad-functions-hash-functions-part-iv/
//! * https://hbfs.wordpress.com/2015/10/27/three-somewhat-better-functions-hash-functions-part-v/
//! * https://hbfs.wordpress.com/2015/11/17/and-a-good-one-hash-functions-part-vi/
//!
//! as well as
//!
//! * https://hbfs.wordpress.com/2011/11/08/mild-obfuscation/
//!
//! > In the previous entries, we learned that a good hash function for
//! > look-ups should disperse bits as much as possible as well as being
//! > unpredictable, that is, behave more or less like a pseudo-random
//! > number generator. We had a few failed attempts, a few promising ones,
//! > and now, a good one.
//! >
//! > One of the weak operations in the previous hash functions is the
//! > combination operation, which is the addition. We remarked that it
//! > isn’t very good because it is unlikely to provoke a global change in
//! > the hash value. Indeed, if you add an 8-bit quantity, then you’re
//! > reasonably certain that the value changes for the first 8 bits, but
//! > after that, changes are operated only through the carry ripple, which
//! > has only probability \frac{1}{2}^i of being propagated to the ith bit.
//! > That is, it is very unlikely to ripple to the end of the word.
//! >
//! > So we need an operation (as simple as possible) to make sure that the
//! > new bits are spread across, and affect, the hash value. Therefore,
//! > we must scatter input bits. But how? Well, we could design some
//! > bit-wise function that takes 8 bits and spread them, but that function
//! > would be fixed, and independent of the input bits (if we consider a
//! > permutation-type function). So we need a splatter that depends on
//! > the input, but covers more or less all bits. Well, we can do that by
//! > (integer) multiplying the next input byte by a large random-looking
//! > number. A random-looking prime number, in fact. Why prime? It will not
//! > introduce new common factors in the subsequent additions other than
//! > those in the input.
//! >
//! > Let’s pick one:
//! >
//! > 173773926194192273.
//! >
//! > This number is 58 bits long. If you multiply an 8-bit value by a 56-bits
//! > value, you’d get, most of the times, a 64-bits value. This time, it
//! > is a bit bigger to compensate the fact the the 8-bit input doesn’t
//! > necessarily use all of its 8 bits. Plus it’s prime. Why? How?
//! >
//! > ![random-typing](https://hbfs.files.wordpress.com/2015/11/random-typing.gif)
//! >
//! > (Yes. For real. That how I typed it. Not sorry.) Then let’s mix the
//! > product. Let’s use the perfect_shuffle we’ve already used. Then
//! > combine this value with a simple addition. The combination step being
//! > strong enough now, we could use a simple confusion step. Let’s use
//! > cut_deck, a function that swaps the high- and low part of the word,
//! > without exchanging bits in each parts, for a bit more confusion.
//!
//! Unfortunately, although this *looks* like a good hash function, it is
//! very slow, likely because it processes the input one byte at a time. If
//! it were modified to correctly handle a larger block, it might actually
//! be competitive.

use std::hash::Hasher;

#[inline]
fn cut_deck(x: u64) -> u64 {
    (x.wrapping_shl(32) | x.wrapping_shr(32))
}

#[inline]
fn perfect_shuffle_32(mut x: u32) -> u32 {
    x = (x & 0xff0000ffu32) | (x & 0x00ff0000u32).wrapping_shr(8) | (x & 0x0000ff00u32).wrapping_shl(8);
    x = (x & 0xf00ff00fu32) | (x & 0x0f000f00u32).wrapping_shr(4) | (x & 0x00f000f0u32).wrapping_shl(4);
    x = (x & 0xc3c3c3c3u32) | (x & 0x30303030u32).wrapping_shr(2) | (x & 0x0c0c0c0cu32).wrapping_shl(2);
    x = (x & 0x99999999u32) | (x & 0x44444444u32).wrapping_shr(1) | (x & 0x22222222u32).wrapping_shl(1);
    x
}

#[inline]
fn perfect_shuffle_64(mut x: u64) -> u64 {
    x = cut_deck(x);
    let xh = perfect_shuffle_32(x.wrapping_shr(32) as u32) as u64;
    let xl = perfect_shuffle_32(x as u32) as u64;
    xh.wrapping_shl(32) | xl
}

pub struct Bricolage(u64);

default_for_constant!(Bricolage, 0);

const MAGIC: u64 = 173773926194192273u64;

impl Hasher for Bricolage {
    #[inline]
    fn finish(&self) -> u64 {
        self.0
    }

    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        for byte in bytes.iter() {
            let shuffled = perfect_shuffle_64((*byte as u64).wrapping_mul(MAGIC));
            self.0 = cut_deck(self.0.wrapping_add(shuffled));
        }
    }
}

hasher_to_fcn!(
    /// Provide access to Bricolage in a single call.
    bricolage,
    Bricolage
);

// ------------------------------------

#[cfg(test)]
mod bricolage_tests {
    use super::*;

    #[test]
    fn basic() {
        assert_eq!(bricolage(b""),   0);
        assert_eq!(bricolage(b"a"),  17926483712435944715);
        assert_eq!(bricolage(b"b"),  12457347154332739726);
        assert_eq!(bricolage(b"ab"), 16461606921607156355);
    }
}

