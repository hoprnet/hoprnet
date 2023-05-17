//! From http://burtleburtle.net/bob/hash/spooky.html
//!
//! Quoted comments are from http://burtleburtle.net/bob/c/SpookyV2.h or
//! http://burtleburtle.net/bob/c/SpookyV2.cpp

use std::hash::Hasher;
use std::mem;
use std::num::Wrapping;
use std::ptr;

/// number of uint64's in internal state
const SC_NUM_VARS: usize = 12;
/// size of the internal state in bytes
const SC_BLOCK_SIZE: usize = SC_NUM_VARS * mem::size_of::<u64>(); // 96
/// size of buffer of unhashed data, in bytes
const SC_BUF_SIZE: usize = 2 * SC_BLOCK_SIZE; // 192
/// > SC_CONST: a constant which:
/// >  * is not zero
/// >  * is odd
/// >  * is a not-very-regular mix of 1's and 0's
/// >  * does not need any other special mathematical properties
const SC_CONST: u64 = 0xdeadbeefdeadbeefu64;

#[inline]
fn offset_to_align<T>(ptr: *const T, align: usize) -> usize {
    align - (ptr as usize & (align - 1))
}

#[inline]
fn rot64(x: Wrapping<u64>, k: usize) -> Wrapping<u64> {
    x << k | x >> (64 - k)
}

/// > This is used if the input is 96 bytes long or longer.
/// >
/// > The internal state is fully overwritten every 96 bytes.
/// > Every input bit appears to cause at least 128 bits of entropy
/// > before 96 other bytes are combined, when run forward or backward
/// >   For every input bit,
/// >   Two inputs differing in just that input bit
/// >   Where "differ" means xor or subtraction
/// >   And the base value is random
/// >   When run forward or backwards one Mix
/// > I tried 3 pairs of each; they all differed by at least 212 bits.
///
/// data indices: 0..11
/// state indices: 0..11
#[inline]
fn mix(data: &[Wrapping<u64>], state: &mut [Wrapping<u64>; 12]) {
    debug_assert!(data.len() >= 12);
    state[0] += data[0];
    state[2] ^= state[10];
    state[11] ^= state[0];
    state[0] = rot64(state[0], 11);
    state[11] += state[1];
    state[1] += data[1];
    state[3] ^= state[11];
    state[0] ^= state[1];
    state[1] = rot64(state[1], 32);
    state[0] += state[2];
    state[2] += data[2];
    state[4] ^= state[0];
    state[1] ^= state[2];
    state[2] = rot64(state[2], 43);
    state[1] += state[3];
    state[3] += data[3];
    state[5] ^= state[1];
    state[2] ^= state[3];
    state[3] = rot64(state[3], 31);
    state[2] += state[4];
    state[4] += data[4];
    state[6] ^= state[2];
    state[3] ^= state[4];
    state[4] = rot64(state[4], 17);
    state[3] += state[5];
    state[5] += data[5];
    state[7] ^= state[3];
    state[4] ^= state[5];
    state[5] = rot64(state[5], 28);
    state[4] += state[6];
    state[6] += data[6];
    state[8] ^= state[4];
    state[5] ^= state[6];
    state[6] = rot64(state[6], 39);
    state[5] += state[7];
    state[7] += data[7];
    state[9] ^= state[5];
    state[6] ^= state[7];
    state[7] = rot64(state[7], 57);
    state[6] += state[8];
    state[8] += data[8];
    state[10] ^= state[6];
    state[7] ^= state[8];
    state[8] = rot64(state[8], 55);
    state[7] += state[9];
    state[9] += data[9];
    state[11] ^= state[7];
    state[8] ^= state[9];
    state[9] = rot64(state[9], 54);
    state[8] += state[10];
    state[10] += data[10];
    state[0] ^= state[8];
    state[9] ^= state[10];
    state[10] = rot64(state[10], 22);
    state[9] += state[11];
    state[11] += data[11];
    state[1] ^= state[9];
    state[10] ^= state[11];
    state[11] = rot64(state[11], 46);
    state[10] += state[0];
}

/// > Mix all 12 inputs together so that h0, h1 are a hash of
/// > them all.
/// >
/// > For two inputs differing in just the input bits Where
/// > "differ" means xor or subtraction And the base value is
/// > random, or a counting value starting at that bit The final
/// > result will have each bit of h0, h1 flip For every input
/// > bit, with probability 50 +- .3% For every pair of input
/// > bits, with probability 50 +- 3%
/// >
/// > This does not rely on the last Mix() call having already
/// > mixed some. Two iterations was almost good enough for a
/// > 64-bit result, but a 128-bit result is reported, so End()
/// > does three iterations.
#[inline]
fn end_partial(state: &mut [Wrapping<u64>; 12]) {
    state[11] += state[1];
    state[2] ^= state[11];
    state[1] = rot64(state[1], 44);
    state[0] += state[2];
    state[3] ^= state[0];
    state[2] = rot64(state[2], 15);
    state[1] += state[3];
    state[4] ^= state[1];
    state[3] = rot64(state[3], 34);
    state[2] += state[4];
    state[5] ^= state[2];
    state[4] = rot64(state[4], 21);
    state[3] += state[5];
    state[6] ^= state[3];
    state[5] = rot64(state[5], 38);
    state[4] += state[6];
    state[7] ^= state[4];
    state[6] = rot64(state[6], 33);
    state[5] += state[7];
    state[8] ^= state[5];
    state[7] = rot64(state[7], 10);
    state[6] += state[8];
    state[9] ^= state[6];
    state[8] = rot64(state[8], 13);
    state[7] += state[9];
    state[10] ^= state[7];
    state[9] = rot64(state[9], 38);
    state[8] += state[10];
    state[11] ^= state[8];
    state[10] = rot64(state[10], 53);
    state[9] += state[11];
    state[0] ^= state[9];
    state[11] = rot64(state[11], 42);
    state[10] += state[0];
    state[1] ^= state[10];
    state[0] = rot64(state[0], 54);
}

#[inline]
fn end(data: &[Wrapping<u64>; 12], state: &mut [Wrapping<u64>; 12]) {
    state[0] += data[0];
    state[1] += data[1];
    state[2] += data[2];
    state[3] += data[3];
    state[4] += data[4];
    state[5] += data[5];
    state[6] += data[6];
    state[7] += data[7];
    state[8] += data[8];
    state[9] += data[9];
    state[10] += data[10];
    state[11] += data[11];
    end_partial(state);
    end_partial(state);
    end_partial(state);
}

/// > The goal is for each bit of the input to expand into 128
/// > bits of apparent entropy before it is fully overwritten. n
/// > trials both set and cleared at least m bits of h0 h1 h2 h3
/// >   n: 2   m: 29
/// >   n: 3   m: 46
/// >   n: 4   m: 57
/// >   n: 5   m: 107
/// >   n: 6   m: 146
/// >   n: 7   m: 152
/// > when run forwards or backwards for all 1-bit and 2-bit
/// > diffs with diffs defined by either xor or subtraction with
/// > a base of all zeros plus a counter, or plus another bit,
/// > or random

#[inline]
fn short_mix(h: &mut [Wrapping<u64>; 4]) {
    h[2] = rot64(h[2], 50);
    h[2] += h[3];
    h[0] ^= h[2];
    h[3] = rot64(h[3], 52);
    h[3] += h[0];
    h[1] ^= h[3];
    h[0] = rot64(h[0], 30);
    h[0] += h[1];
    h[2] ^= h[0];
    h[1] = rot64(h[1], 41);
    h[1] += h[2];
    h[3] ^= h[1];
    h[2] = rot64(h[2], 54);
    h[2] += h[3];
    h[0] ^= h[2];
    h[3] = rot64(h[3], 48);
    h[3] += h[0];
    h[1] ^= h[3];
    h[0] = rot64(h[0], 38);
    h[0] += h[1];
    h[2] ^= h[0];
    h[1] = rot64(h[1], 37);
    h[1] += h[2];
    h[3] ^= h[1];
    h[2] = rot64(h[2], 62);
    h[2] += h[3];
    h[0] ^= h[2];
    h[3] = rot64(h[3], 34);
    h[3] += h[0];
    h[1] ^= h[3];
    h[0] = rot64(h[0], 5);
    h[0] += h[1];
    h[2] ^= h[0];
    h[1] = rot64(h[1], 36);
    h[1] += h[2];
    h[3] ^= h[1];
}

/// > Mix all 4 inputs together so that h0, h1 are a hash of them all.
/// >
/// > For two inputs differing in just the input bits
/// > Where "differ" means xor or subtraction
/// > And the base value is random, or a counting value starting at that bit
/// > The final result will have each bit of h0, h1 flip
/// > For every input bit,
/// > with probability 50 +- .3% (it is probably better than that)
/// > For every pair of input bits,
/// > with probability 50 +- .75% (the worst case is approximately that)

#[inline]
fn short_end(h: &mut [Wrapping<u64>; 4]) {
    h[3] ^= h[2];
    h[2] = rot64(h[2], 15);
    h[3] += h[2];
    h[0] ^= h[3];
    h[3] = rot64(h[3], 52);
    h[0] += h[3];
    h[1] ^= h[0];
    h[0] = rot64(h[0], 26);
    h[1] += h[0];
    h[2] ^= h[1];
    h[1] = rot64(h[1], 51);
    h[2] += h[1];
    h[3] ^= h[2];
    h[2] = rot64(h[2], 28);
    h[3] += h[2];
    h[0] ^= h[3];
    h[3] = rot64(h[3], 9);
    h[0] += h[3];
    h[1] ^= h[0];
    h[0] = rot64(h[0], 47);
    h[1] += h[0];
    h[2] ^= h[1];
    h[1] = rot64(h[1], 54);
    h[2] += h[1];
    h[3] ^= h[2];
    h[2] = rot64(h[2], 32);
    h[3] += h[2];
    h[0] ^= h[3];
    h[3] = rot64(h[3], 25);
    h[0] += h[3];
    h[1] ^= h[0];
    h[0] = rot64(h[0], 63);
    h[1] += h[0];
}

/// > Short is used for messages under 192 bytes in length. Short
/// > has a low startup cost, the normal mode is good for long
/// > keys, the cost crossover is at about 192 bytes. The two modes
/// > were held to the same quality bar.
fn short(message: &[u8], length: usize, hash1: &mut Wrapping<u64>, hash2: &mut Wrapping<u64>) {
    debug_assert!(length <= SC_BUF_SIZE);
    let mut h: [Wrapping<u64>; 4] = [*hash1, *hash2, Wrapping(SC_CONST), Wrapping(SC_CONST)];
    for chunk in message.chunks(4 * mem::size_of::<u64>()) {
        if chunk.len() == 4 * mem::size_of::<u64>() {
            let mut buf = [Wrapping(0u64); 4 * mem::size_of::<u64>()];
            let words: &[Wrapping<u64>];
            if offset_to_align(chunk.as_ptr(), 4) == 0 {
                words = unsafe { mem::transmute(chunk) };
            } else {
                unsafe {
                    ptr::copy_nonoverlapping(
                        chunk.as_ptr(),
                        &mut buf as *mut _ as *mut u8,
                        4 * mem::size_of::<u64>(),
                    );
                }
                words = &buf;
            }
            h[2] += words[0];
            h[3] += words[1];
            short_mix(&mut h);
            h[0] += words[2];
            h[1] += words[3];
        } else if chunk.len() >= 2 * mem::size_of::<u64>() {
            let mut buf = [Wrapping(0u64); 2 * mem::size_of::<u64>()];
            let words: &[Wrapping<u64>];
            if offset_to_align(message.as_ptr(), 4) == 0 {
                words = unsafe { mem::transmute(message) };
            } else {
                unsafe {
                    ptr::copy_nonoverlapping(
                        chunk.as_ptr(),
                        &mut buf as *mut _ as *mut u8,
                        2 * mem::size_of::<u64>(),
                    );
                }
                words = &buf;
            }
            h[2] += words[0];
            h[3] += words[1];
            short_mix(&mut h);
        } else {
            h[3] += Wrapping(length as u64) << 56;
            if chunk.len() >= 12 {
                if chunk.len() > 14 {
                    h[3] += Wrapping(chunk[14] as u64) << 48;
                }
                if chunk.len() > 13 {
                    h[3] += Wrapping(chunk[13] as u64) << 40;
                }
                if chunk.len() > 12 {
                    h[3] += Wrapping(chunk[12] as u64) << 32;
                }
                h[2] += Wrapping(load_int_le!(chunk, 0, u64));
                h[3] += Wrapping(load_int_le!(chunk, 8, u32) as u64);
            } else if chunk.len() >= 8 {
                if chunk.len() > 10 {
                    h[3] += Wrapping(chunk[10] as u64) << 16;
                }
                if chunk.len() > 9 {
                    h[3] += Wrapping(chunk[9] as u64) << 8;
                }
                if chunk.len() > 8 {
                    h[3] += Wrapping(chunk[8] as u64);
                }
                h[2] += Wrapping(load_int_le!(chunk, 0, u64));
            } else if chunk.len() >= 4 {
                if chunk.len() > 6 {
                    h[2] += Wrapping(chunk[6] as u64) << 48;
                }
                if chunk.len() > 5 {
                    h[2] += Wrapping(chunk[5] as u64) << 40;
                }
                if chunk.len() > 4 {
                    h[2] += Wrapping(chunk[4] as u64) << 32;
                }
                h[2] += Wrapping(load_int_le!(chunk, 0, u32) as u64);
            } else if chunk.len() >= 1 {
                if chunk.len() > 2 {
                    h[2] += Wrapping(chunk[2] as u64) << 16;
                }
                if chunk.len() > 1 {
                    h[2] += Wrapping(chunk[1] as u64) << 8;
                }
                h[2] += Wrapping(chunk[0] as u64);
            } else {
                h[2] += Wrapping(SC_CONST);
                h[3] += Wrapping(SC_CONST);
            }
        }
    }
    short_end(&mut h);
    *hash1 = h[0];
    *hash2 = h[1];
}

/// From http://burtleburtle.net/bob/hash/spooky.html
/// > SpookyHash is a public domain noncryptographic hash function producing well-distributed
/// > 128-bit hash values for byte arrays of any length. It can produce 64-bit and 32-bit hash values
/// > too, at the same speed, just use the bottom n bits. The C++ reference implementation is
/// > specific to 64-bit x86 platforms, in particular it assumes the processor is little endian. Long
/// > keys hash in 3 bytes per cycle, short keys take about 1 byte per cycle, and there is a 30 cycle
/// > startup cost. Keys can be supplied in fragments. The function allows a 128-bit seed. It's named
/// > SpookyHash because it was released on Halloween.
pub struct SpookyHasher {
    // unhashed data, for partial messages; 2 * m_state, in bytes
    pub m_data: [u8; SC_BUF_SIZE],
    // internal state of the hash
    pub m_state: [Wrapping<u64>; SC_NUM_VARS],
    // total length of the input so far
    pub m_length: usize,
    // length of unhashed data stashed in m_data
    pub m_remainder: usize,
}

impl SpookyHasher {
    pub fn new(seed1: u64, seed2: u64) -> SpookyHasher {
        let mut sh = SpookyHasher {
            m_data: [0; SC_BUF_SIZE],
            m_state: [Wrapping(0u64); SC_NUM_VARS],
            m_length: 0,
            m_remainder: 0,
        };
        sh.m_state[0] = Wrapping(seed1);
        sh.m_state[3] = Wrapping(seed1);
        sh.m_state[6] = Wrapping(seed1);
        sh.m_state[9] = Wrapping(seed1);
        sh.m_state[1] = Wrapping(seed2);
        sh.m_state[4] = Wrapping(seed2);
        sh.m_state[7] = Wrapping(seed2);
        sh.m_state[10] = Wrapping(seed2);
        sh.m_state[2] = Wrapping(SC_CONST);
        sh.m_state[5] = Wrapping(SC_CONST);
        sh.m_state[8] = Wrapping(SC_CONST);
        sh.m_state[11] = Wrapping(SC_CONST);
        sh
    }

    pub fn finish128(&self) -> (u64, u64) {
        if self.m_length < SC_BUF_SIZE {
            let mut hash1 = self.m_state[0];
            let mut hash2 = self.m_state[1];
            short(&self.m_data, self.m_length, &mut hash1, &mut hash2);
            return (hash1.0, hash2.0);
        }
        let mut state = self.m_state;
        let mut remainder = self.m_remainder;
        let mut processed = 0;
        if self.m_remainder >= SC_BLOCK_SIZE {
            let data = unsafe { mem::transmute::<&[u8], &[Wrapping<u64>]>(&self.m_data) };
            mix(data, &mut state);
            processed = SC_BLOCK_SIZE;
            remainder -= SC_BLOCK_SIZE;
        }
        let mut data = [Wrapping(0u64); SC_NUM_VARS];
        unsafe {
            ptr::copy_nonoverlapping::<u8>(
                (&self.m_data as &_ as *const u8).offset(processed as isize),
                data.as_mut_ptr() as *mut u8,
                remainder,
            );
            ptr::write_bytes(
                (data.as_mut_ptr() as *mut u8).offset((SC_BLOCK_SIZE - 1) as isize),
                remainder as u8,
                1,
            );
        }
        end(&data, &mut state);
        (state[0].0, state[1].0)
    }
}

impl Default for SpookyHasher {
    fn default() -> SpookyHasher {
        SpookyHasher::new(0, 0)
    }
}

impl Hasher for SpookyHasher {
    #[inline]
    fn finish(&self) -> u64 {
        self.finish128().0
    }

    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        let new_length = self.m_remainder + bytes.len();
        // if the fragment is too short, store it for later
        if new_length < SC_BUF_SIZE {
            unsafe {
                ptr::copy_nonoverlapping(
                    bytes.as_ptr(),
                    (self.m_data.as_mut_ptr() as *mut u8).offset(self.m_remainder as isize),
                    bytes.len(),
                );
            }
            self.m_length += bytes.len();
            self.m_remainder = new_length;
            return;
        }
        self.m_length += bytes.len();
        let mut processed = 0;
        // if we've got anything stuffed away, use it now
        if self.m_remainder > 0 {
            // add the prefix of bytes to m_data
            processed = SC_BUF_SIZE - self.m_remainder;
            println!("{}", processed);
            unsafe {
                ptr::copy_nonoverlapping(
                    bytes.as_ptr(),
                    (self.m_data.as_mut_ptr() as *mut u8).offset(self.m_remainder as isize),
                    processed,
                );
            }
            let data: &[Wrapping<u64>] =
                unsafe { mem::transmute::<&[u8], &[Wrapping<u64>]>(&self.m_data) };
            println!("m_data {} data {}", self.m_data.len(), data.len());
            mix(&data, &mut self.m_state);
            mix(&data[SC_NUM_VARS..], &mut self.m_state);
            self.m_remainder = 0;
        }
        // process the rest of the bytes
        for chunk in bytes[processed..].chunks(SC_BLOCK_SIZE) {
            if chunk.len() == SC_BLOCK_SIZE {
                // handle whole blocks of SC_BLOCK_SIZE bytes
                if offset_to_align(&chunk, 8) == 0 {
                    let mut data: &[Wrapping<u64>] =
                        unsafe { mem::transmute::<&[u8], &[Wrapping<u64>]>(chunk) };
                    mix(&data, &mut self.m_state);
                } else {
                    unsafe {
                        ptr::copy_nonoverlapping(
                            chunk.as_ptr(),
                            self.m_data.as_mut_ptr() as *mut u8,
                            SC_BLOCK_SIZE,
                        );
                    }
                    let mut data: &[Wrapping<u64>] =
                        unsafe { mem::transmute::<&[u8], &[Wrapping<u64>]>(&self.m_data) };
                    mix(&data, &mut self.m_state);
                }
            } else {
                // stuff away the last few bytes
                unsafe {
                    ptr::copy_nonoverlapping(
                        chunk.as_ptr(),
                        self.m_data.as_mut_ptr() as *mut u8,
                        chunk.len(),
                    );
                }
                self.m_remainder = chunk.len();
            }
        }
    }
}

hasher_to_fcn!(
    /// Provide access to Lookup3Hasher in a single call.
    spooky,
    SpookyHasher
);

#[cfg(test)]
mod spookyhash_test {
    use super::*;

    #[test]
    fn basic() {
        assert_eq!(spooky(b""), 13905049616862401802);
        assert_eq!(spooky(b"a"), 16932945748884078726);
        assert_eq!(spooky(b"b"), 5781063613870495197);
        assert_eq!(spooky(b"ab"), 13849109452443161137);
        assert_eq!(spooky(b"abcd"), 4142038200967391753);
        assert_eq!(spooky(b"abcdefg"), 2761526316938866980);
        assert_eq!(spooky(b"abcdefghijklmnopqrstuvwxyz"), 16192181224158463141);
    }
}
