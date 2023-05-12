//! From http://www.burtleburtle.net/bob/hash/doobs.html.
//!
//! This module mostly comes from his survey of hash functions. See also
//! https://en.wikipedia.org/wiki/Jenkins_hash_function.
//!
//! This module includes a sub-module implementing SpookyHash.

use std::hash::Hasher;
use std::num::Wrapping;
use std::{mem, ptr};

pub mod spooky_hash;

// ================================
// one_at_a_time

/// The one-at-a-time Hasher. Relatively simple, but superseded by
/// later algorithms.
///
/// From http://www.burtleburtle.net/bob/hash/doobs.html:
///
/// > This is similar to the rotating hash, but it actually mixes
/// > the internal state. It takes 9n+9 instructions and produces a
/// > full 4-byte result. Preliminary analysis suggests there are no
/// > funnels.
/// >
/// > This hash was not in the original Dr. Dobb's article. I
/// > implemented it to fill a set of requirements posed by Colin
/// > Plumb. Colin ended up using an even simpler (and weaker) hash
/// > that was sufficient for his purpose.
pub struct OAATHasher(Wrapping<u64>);

impl Hasher for OAATHasher {
    #[inline]
    fn finish(&self) -> u64 {
        let mut hash = self.0;
        hash += hash << 3;
        hash ^= hash >> 11;
        hash += hash << 15;
        hash.0
    }

    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        for byte in bytes.iter() {
            self.0 += Wrapping(*byte as u64);
            self.0 += self.0 << 10;
            self.0 ^= self.0 >> 6;
        }
    }
}

default_for_constant!(OAATHasher, Wrapping(0));
hasher_to_fcn!(
    /// Provide access to OAATHasher in a single call.
    oaat,
    OAATHasher
);

// ------------------------------------

#[cfg(test)]
mod oaat_tests {
    use super::*;

    #[test]
    fn basic() {
        assert_eq!(oaat(b""), 0);
        assert_eq!(oaat(b"a"), 29161854018);
        assert_eq!(oaat(b"b"), 30079156635);
        assert_eq!(oaat(b"ab"), 30087418617432);
        assert_eq!(oaat(b"abcdefg"), 3103867595652801641);
    }
}

// ================================
// lookup3

/// Another Hasher from the inventor of SpookyHash. Fancy bit-mixing. *Very fancy.*
///
/// From http://www.burtleburtle.net/bob/hash/doobs.html:
///
/// > ...http://burtleburtle.net/bob/c/lookup3.c (2006) is about 2
/// > cycles/byte, works well on 32-bit platforms, and can produce a
/// > 32 or 64 bit hash.
///
/// > A hash I wrote nine years later designed along the same lines
/// > as "My Hash", see http://burtleburtle.net/bob/c/lookup3.c.
/// > It takes 2n instructions per byte for mixing instead of 3n.
/// > When fitting bytes into registers (the other 3n instructions),
/// > it takes advantage of alignment when it can (a trick learned
/// > from Paul Hsieh's hash). It doesn't bother to reserve a byte
/// > for the length. That allows zero-length strings to require no
/// > mixing. More generally, the length that requires additional
/// > mixes is now 13-25-37 instead of 12-24-36.
/// >
/// > One theoretical insight was that the last mix doesn't need to
/// > do well in reverse (though it has to affect all output bits).
/// > And the middle mixing steps don't have to affect all output
/// > bits (affecting some 32 bits is enough), though it does have
/// > to do well in reverse. So it uses different mixes for those
/// > two cases. "My Hash" (lookup2.c) had a single mixing operation
/// > that had to satisfy both sets of requirements, which is why it
/// > was slower.
/// >
/// > On a Pentium 4 with gcc 3.4.?, Paul's hash was usually faster
/// > than lookup3.c. On a Pentium 4 with gcc 3.2.?, they were about
/// > the same speed. On a Pentium 4 with icc -O2, lookup3.c was a
/// > little faster than Paul's hash. I don't know how it would play
/// > out on other chips and other compilers. lookup3.c is slower
/// > than the additive hash pretty much forever, but it's faster
/// > than the rotating hash for keys longer than 5 bytes.
/// >
/// > lookup3.c does a much more thorough job of mixing than any of
/// > my previous hashes (lookup2.c, lookup.c, One-at-a-time). All
/// > my previous hashes did a more thorough job of mixing than Paul
/// > Hsieh's hash. Paul's hash does a good enough job of mixing for
/// > most practical purposes.
/// >
/// > The most evil set of keys I know of are sets of keys that are
/// > all the same length, with all bytes zero, except with a few
/// > bits set. This is tested by frog.c.. To be even more evil, I
/// > had my hashes return b and c instead of just c, yielding a
/// > 64-bit hash value. Both lookup.c and lookup2.c start seeing
/// > collisions after 253 frog.c keypairs. Paul Hsieh's hash sees
/// > collisions after 217 keypairs, even if we take two hashes
/// > with different seeds. lookup3.c is the only one of the batch
/// > that passes this test. It gets its first collision somewhere
/// > beyond 263 keypairs, which is exactly what you'd expect from a
/// > completely random mapping to 64-bit values.
///
/// This structure implements hashlittle2:
///
/// > You probably want to use hashlittle(). hashlittle() and
/// > hashbig() hash byte arrays. hashlittle() is is faster than
/// > hashbig() on little-endian machines. Intel and AMD are
/// > little-endian machines. On second thought, you probably want
/// > hashlittle2(), which is identical to hashlittle() except it
/// > returns two 32-bit hashes for the price of one. You could
/// > implement hashbig2() if you wanted but I haven't bothered
/// > here.
///
/// See http://www.burtleburtle.net/bob/c/lookup3.c.
pub struct Lookup3Hasher {
    pc: Wrapping<u32>, // primary initval / primary hash
    pb: Wrapping<u32>, // secondary initval / secondary hash
}

impl Default for Lookup3Hasher {
    fn default() -> Lookup3Hasher {
        Lookup3Hasher {
            pc: Wrapping(0),
            pb: Wrapping(0),
        }
    }
}

#[inline]
fn rot(x: Wrapping<u32>, k: usize) -> Wrapping<u32> {
    x << k | x >> (32 - k)
}

/// > mix -- mix 3 32-bit values reversibly.
/// >
/// > This is reversible, so any information in (a,b,c) before mix() is
/// > still in (a,b,c) after mix().
/// >
/// > If four pairs of (a,b,c) inputs are run through mix(), or through
/// > mix() in reverse, there are at least 32 bits of the output that
/// > are sometimes the same for one pair and different for another pair.
/// > This was tested for:
/// > * pairs that differed by one bit, by two bits, in any combination
/// >   of top bits of (a,b,c), or in any combination of bottom bits of
/// >   (a,b,c).
/// > * "differ" is defined as +, -, ^, or ~^.  For + and -, I transformed
/// >   the output delta to a Gray code (a^(a>>1)) so a string of 1's (as
/// >   is commonly produced by subtraction) look like a single 1-bit
/// >   difference.
/// > * the base values were pseudorandom, all zero but one bit set, or
/// >   all zero plus a counter that starts at zero.
/// >
/// > Some k values for my "a-=c; a^=rot(c,k); c+=b;" arrangement that
/// > satisfy this are
/// >     4  6  8 16 19  4
/// >     9 15  3 18 27 15
/// >    14  9  3  7 17  3
/// > Well, "9 15 3 18 27 15" didn't quite get 32 bits diffing
/// > for "differ" defined as + with a one-bit base and a two-bit delta.  I
/// > used http://burtleburtle.net/bob/hash/avalanche.html to choose
/// > the operations, constants, and arrangements of the variables.
/// >
/// > This does not achieve avalanche.  There are input bits of (a,b,c)
/// > that fail to affect some output bits of (a,b,c), especially of a.  The
/// > most thoroughly mixed value is c, but it doesn't really even achieve
/// > avalanche in c.
/// >
/// > This allows some parallelism.  Read-after-writes are good at doubling
/// > the number of bits affected, so the goal of mixing pulls in the opposite
/// > direction as the goal of parallelism.  I did what I could.  Rotates
/// > seem to cost as much as shifts on every machine I could lay my hands
/// > on, and rotates are much kinder to the top and bottom bits, so I used
/// > rotates.
#[inline]
fn mix(a: &mut Wrapping<u32>, b: &mut Wrapping<u32>, c: &mut Wrapping<u32>) {
    *a -= *c;
    *a ^= rot(*c, 4);
    *c += *b;
    *b -= *a;
    *b ^= rot(*a, 6);
    *a += *c;
    *c -= *b;
    *c ^= rot(*b, 8);
    *b += *a;
    *a -= *c;
    *a ^= rot(*c, 16);
    *c += *b;
    *b -= *a;
    *b ^= rot(*a, 19);
    *a += *c;
    *c -= *b;
    *c ^= rot(*b, 4);
    *b += *a;
}

/// > final -- final mixing of 3 32-bit values (a,b,c) into c
/// >
/// > Pairs of (a,b,c) values differing in only a few bits will usually
/// > produce values of c that look totally different.  This was tested for
/// > - pairs that differed by one bit, by two bits, in any combination
/// >   of top bits of (a,b,c), or in any combination of bottom bits of
/// >   (a,b,c).
/// > - "differ" is defined as +, -, ^, or ~^.  For + and -, I transformed
/// >   the output delta to a Gray code (a^(a>>1)) so a string of 1's (as
/// >   is commonly produced by subtraction) look like a single 1-bit
/// >   difference.
/// > - the base values were pseudorandom, all zero but one bit set, or
/// >   all zero plus a counter that starts at zero.
/// >
/// > These constants passed:
/// >
/// >  14 11 25 16 4 14 24
/// >  12 14 25 16 4 14 24
/// >
/// > and these came close:
/// >
/// >  4  8 15 26 3 22 24
/// >  10  8 15 26 3 22 24
/// >  11  8 15 26 3 22 24
#[inline]
fn final_mix(a: &mut Wrapping<u32>, b: &mut Wrapping<u32>, c: &mut Wrapping<u32>) {
    *c ^= *b;
    *c -= rot(*b, 14);
    *a ^= *c;
    *a -= rot(*c, 11);
    *b ^= *a;
    *b -= rot(*a, 25);
    *c ^= *b;
    *c -= rot(*b, 16);
    *a ^= *c;
    *a -= rot(*c, 4);
    *b ^= *a;
    *b -= rot(*a, 14);
    *c ^= *b;
    *c -= rot(*b, 24);
}

/// Turn 0-4 bytes into an unsigned 32-bit number.
#[inline]
fn shift_add(s: &[u8]) -> Wrapping<u32> {
    Wrapping(match s.len() {
        1 => s[0] as u32,
        2 => (s[0] as u32) + ((s[1] as u32) << 8),
        3 => (s[0] as u32) + ((s[1] as u32) << 8) + ((s[2] as u32) << 16),
        4 => (s[0] as u32) + ((s[1] as u32) << 8) + ((s[2] as u32) << 16) + ((s[3] as u32) << 24),
        _ => 0 as u32,
    })
}

#[inline]
fn offset_to_align<T>(ptr: *const T, align: usize) -> usize {
    align - (ptr as usize & (align - 1))
}

impl Hasher for Lookup3Hasher {
    #[inline]
    fn finish(&self) -> u64 {
        (self.pc.0 as u64) + ((self.pb.0 as u64) << 32)
    }

    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        if bytes.len() == 0 {
            return;
        }
        let initial = Wrapping(0xdeadbeefu32) + Wrapping(bytes.len() as u32) + self.pc;
        let mut a: Wrapping<u32> = initial;
        let mut b: Wrapping<u32> = initial;
        let mut c: Wrapping<u32> = initial;
        c += self.pb;

        if cfg!(target_endian = "little") && offset_to_align(bytes.as_ptr(), 4) == 0 {
            for chunk in bytes.chunks(12) {
                if chunk.len() == 12 {
                    // true for all chunks except the last
                    a += Wrapping(load_int_le!(chunk, 0, u32));
                    b += Wrapping(load_int_le!(chunk, 4, u32));
                    c += Wrapping(load_int_le!(chunk, 8, u32));
                    mix(&mut a, &mut b, &mut c);
                } else if chunk.len() >= 8 {
                    a += Wrapping(load_int_le!(chunk, 0, u32));
                    b += Wrapping(load_int_le!(chunk, 4, u32));
                    c += shift_add(&chunk[8..]);
                } else if chunk.len() >= 4 {
                    a += Wrapping(load_int_le!(chunk, 0, u32));
                    b += shift_add(&chunk[4..]);
                } else {
                    a += shift_add(chunk);
                }
            }
        } else if cfg!(target_endian = "little") && offset_to_align(bytes.as_ptr(), 2) == 0 {
            for chunk in bytes.chunks(12) {
                if chunk.len() == 12 {
                    // true for all chunks except the last
                    a += Wrapping(load_int_le!(chunk, 0, u16) as u32)
                        + Wrapping(load_int_le!(chunk, 0, u16) as u32)
                        << 16;
                    b += Wrapping(load_int_le!(chunk, 4, u16) as u32)
                        + Wrapping(load_int_le!(chunk, 6, u16) as u32)
                        << 16;
                    c += Wrapping(load_int_le!(chunk, 8, u16) as u32)
                        + Wrapping(load_int_le!(chunk, 10, u16) as u32)
                        << 16;
                    mix(&mut a, &mut b, &mut c);
                } else if chunk.len() >= 8 {
                    a += Wrapping(load_int_le!(chunk, 0, u16) as u32)
                        + Wrapping(load_int_le!(chunk, 0, u16) as u32)
                        << 16;
                    b += Wrapping(load_int_le!(chunk, 4, u16) as u32)
                        + Wrapping(load_int_le!(chunk, 6, u16) as u32)
                        << 16;
                    c += shift_add(&chunk[8..]);
                } else if chunk.len() >= 4 {
                    a += Wrapping(load_int_le!(chunk, 0, u16) as u32)
                        + Wrapping(load_int_le!(chunk, 0, u16) as u32)
                        << 16;
                    b += shift_add(&chunk[4..]);
                } else {
                    a += shift_add(chunk);
                }
            }
        } else {
            // For big endian machines and unaligned slices: hash bytes.
            // "You could implement hashbig2() if you wanted but I
            // haven't bothered here."
            for chunk in bytes.chunks(12) {
                if chunk.len() == 12 {
                    a += shift_add(&chunk[..4]);
                    b += shift_add(&chunk[4..8]);
                    c += shift_add(&chunk[8..]);
                    mix(&mut a, &mut b, &mut c);
                } else if chunk.len() >= 8 {
                    a += shift_add(&chunk[..4]);
                    b += shift_add(&chunk[4..8]);
                    c += shift_add(&chunk[8..]);
                } else if chunk.len() >= 4 {
                    a += shift_add(&chunk[..4]);
                    b += shift_add(&chunk[4..]);
                } else {
                    a += shift_add(chunk);
                }
            }
        }
        final_mix(&mut a, &mut b, &mut c);
        self.pb = b;
        self.pc = c;
    }
}

hasher_to_fcn!(
    /// Provide access to Lookup3Hasher in a single call.
    lookup3,
    Lookup3Hasher
);

// ------------------------------------

#[cfg(test)]
mod lookup3_tests {
    use super::*;

    #[test]
    fn basic() {
        assert_eq!(lookup3(b""), 0);
        assert_eq!(lookup3(b"a"), 6351843130003064584);
        assert_eq!(lookup3(b"b"), 5351957087540069269);
        assert_eq!(lookup3(b"ab"), 7744397999705663711);
        assert_eq!(lookup3(b"abcd"), 16288908501016938652);
        assert_eq!(lookup3(b"abcdefg"), 6461572128488215717);
    }
}
