//! From http://www.cse.yorku.ca/~oz/hash.html.
//!
//! > A comprehensive collection of hash functions, a hash visualiser
//! > and some test results [see Mckenzie et al. Selecting a Hashing
//! > Algorithm, SP&E 20(2):209-224, Feb 1990] will be available
//! > someday. If you just want to have a good hash function, and cannot
//! > wait, djb2 is one of the best string hash functions i know. it has
//! > excellent distribution and speed on many different sets of keys
//! > and table sizes. you are not likely to do better with one of the
//! > "well known" functions such as PJW, K&R, etc. Also see tpop
//! > pp. 126 for graphing hash functions.
//!
//! "tpop" is *The Practice of Programming*. This page shows three
//! classic hashing algorithms.
use std::hash::Hasher;
use std::num::Wrapping;

// ====================================
// DJB2

/// Dan Bernstein's famous hashing function.
///
/// This Hasher is allegedly good for small tables with lowercase
/// ASCII keys. It is also dirt-simple, although other hash
/// functions are better and almost as simple.
///
/// From http://www.cse.yorku.ca/~oz/hash.html:
///
/// > this algorithm (k=33) was first reported by dan bernstein many
/// > years ago in comp.lang.c. another version of this algorithm (now
/// > favored by bernstein) uses xor: `hash(i) = hash(i - 1) * 33 ^
/// > str[i];` the magic of number 33 (why it works better than many
/// > other constants, prime or not)
/// > has never been adequately explained.
///
/// From http://www.burtleburtle.net/bob/hash/doobs.html:
///
/// > If your keys are lowercase English words, this will fit 6
/// > characters into a 32-bit hash with no collisions (you'd
/// > have to compare all 32 bits). If your keys are mixed case
/// > English words, `65 * hash+key[i]` fits 5 characters into a 32-bit
/// > hash with no collisions. That means this type of hash can
/// > produce (for the right type of keys) fewer collisions than
/// > a hash that gives a more truly random distribution. If your
/// > platform doesn't have fast multiplies, no sweat, 33 * hash =
/// > hash+(hash<<5) and most compilers will figure that out for
/// > you.
/// >
/// > On the down side, if you don't have short text keys, this hash
/// > has a easily detectable flaws. For example, there's a 3-into-2
/// > funnel that 0x0021 and 0x0100 both have the same hash (hex
/// > 0x21, decimal 33) (you saw that one coming, yes?).
pub struct DJB2Hasher(Wrapping<u32>);

impl Hasher for DJB2Hasher {
    #[inline]
    fn finish(&self) -> u64 {
        (self.0).0 as u64
    }

    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        for byte in bytes.iter() {
            self.0 = self.0 + (self.0 << 5) ^ Wrapping(*byte as u32);
        }
    }
}

default_for_constant!(DJB2Hasher, Wrapping(5381));
hasher_to_fcn!(
    /// Provide access to DJB2Hasher in a single call.
    djb2,
    DJB2Hasher
);

// ------------------------------------

#[cfg(test)]
mod djb2_tests {
    use super::*;

    #[test]
    fn basic() {
        assert_eq!(djb2(b""), 5381);
        assert_eq!(djb2(b"a"), 177604);
        assert_eq!(djb2(b"b"), 177607);
        assert_eq!(djb2(b"ab"), 5860902);
    }
}

// ====================================
// sdbm

/// The hash function from SDBM (and gawk?). It might be good for
/// something.
///
/// From http://www.cse.yorku.ca/~oz/hash.html:
///
/// > this algorithm was created for sdbm (a public-domain
/// > reimplementation of ndbm) database library. it was found
/// > to do well in scrambling bits, causing better distribution
/// > of the keys and fewer splits. it also happens to be a good
/// > general hashing function with good distribution. the actual
/// > function is `hash(i) = hash(i - 1) * 65599 + str[i];` what is
/// > included below is the faster version used in gawk. [there is
/// > even a faster, duff-device version] the magic constant 65599
/// > was picked out of thin air while experimenting with different
/// > constants, and turns out to be a prime. this is one of the
/// > algorithms used in berkeley db (see sleepycat) and elsewhere.
pub struct SDBMHasher(Wrapping<u32>);

impl Hasher for SDBMHasher {
    #[inline]
    fn finish(&self) -> u64 {
        (self.0).0 as u64
    }

    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        for byte in bytes.iter() {
            self.0 = Wrapping(*byte as u32) + (self.0 << 6) + (self.0 << 16) - self.0;
        }
    }
}

default_for_constant!(SDBMHasher, Wrapping(0));
hasher_to_fcn!(
    /// Provide access to SDBMHasher in a single call.
    sdbm,
    SDBMHasher
);

// ------------------------------------

#[cfg(test)]
mod sdbm_tests {
    use super::*;

    #[test]
    fn basic() {
        assert_eq!(sdbm(b""), 0);
        assert_eq!(sdbm(b"a"), 97);
        assert_eq!(sdbm(b"b"), 98);
        assert_eq!(sdbm(b"ab"), 6363201);
    }
}

// ====================================
// lose_lose

/// A radically bad hash function from the 1st edition of the K&R C
/// book. Do not use; it's horrible.
///
/// From http://www.cse.yorku.ca/~oz/hash.html
///
/// > This hash function appeared in K&R (1st ed) but at least the
/// > reader was warned: "This is not the best possible algorithm,
/// > but it has the merit of extreme simplicity." This is an
/// > understatement; It is a terrible hashing algorithm, and it
/// > could have been much better without sacrificing its "extreme
/// > simplicity." [see the second edition!] Many C programmers
/// > use this function without actually testing it, or checking
/// > something like Knuth's Sorting and Searching, so it stuck. It
/// > is now found mixed with otherwise respectable code, eg. cnews.
/// > sigh. [see also: tpop]
pub struct LoseLoseHasher(Wrapping<u64>);

impl Hasher for LoseLoseHasher {
    #[inline]
    fn finish(&self) -> u64 {
        (self.0).0
    }

    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        for byte in bytes.iter() {
            self.0 += Wrapping(*byte as u64);
        }
    }
}

default_for_constant!(LoseLoseHasher, Wrapping(0));
hasher_to_fcn!(
    /// Provide access to LoseLoseHasher in a single call.
    loselose,
    LoseLoseHasher
);

// ------------------------------------

#[cfg(test)]
mod loselose_tests {
    use super::*;

    #[test]
    fn basic() {
        assert_eq!(loselose(b""), 0);
        assert_eq!(loselose(b"a"), 97);
        assert_eq!(loselose(b"b"), 98);
        assert_eq!(loselose(b"ab"), 195);
    }
}
