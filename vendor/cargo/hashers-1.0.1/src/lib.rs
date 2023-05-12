//! # hashers
//!
//! This module contains implementations and re-exports of a number of
//! (non-cryptographic) hashing functions suitable for use with Rust's
//! HashMap and HashSet.
//!
//! Additionally, there are benchmarks of the hash functions and a
//! couple of statistical tests for hash quality.
//!
//! # Disclaimer
//!
//! To quote [fxhash](https://github.com/cbreeden/fxhash),
//!
//! > [None of these are] cryptographically secure hash, so it is strongly
//! > recommended that you do not use this hash for cryptographic
//! > purproses. Furthermore, this hashing algorithm was not designed to
//! > prevent any attacks for determining collisions which could be used
//! > to potentially cause quadratic behavior in HashMaps. So it is not
//! > recommended to expose this hash in places where collissions or DDOS
//! > attacks may be a concern.
//!
//! # What's a Hasher?
//!
//! A hash function, for our purposes here, is a function that takes as
//! input another, general, value, and returns a number that is
//! ideally unique to that value. This number can be used to
//! store the value in an array and then locate it again later
//! without searching the array; in other words, in O(1) time. More or
//! less: there are a lot of other details. For more information, see
//! Rust's HashMap and various information sources online.
//!
//! In Rust specifically, std::hash::Hasher is a trait:
//!
//! ```text
//! pub trait Hasher {
//!     fn finish(&self) -> u64;
//!     fn write(&mut self, bytes: &[u8]);
//! 
//!     fn write_u8(&mut self, i: u8) { ... }
//!     fn write_u16(&mut self, i: u16) { ... }
//!     ...
//! }
//! ```
//!
//! Hasher has two required methods, `finish` and `write`, and default implementations of other
//! useful methods like `write_u8` and `write_u16`, implemented by calling `write`. In use, an
//! implementation of Hasher is created, data is fed to it using the various `write` methods, then
//! the result is returned using the `finish` method to get the hash number out.
//!
//! # Using a custom hash function in Rust
//!
//! Using a custom hash function with Rust's HashMap or HashSet has long been regarded as a deep
//! mystery. Now, I will strip back the curtains of ignorance and reveal the secrets in all their
//! unholy glory!
//!
//! Or something like that. It's not really a big deal.
//!
//! There are two ways to create a HashMap that uses a custom Hasher implementation: setting the
//! hasher on a hash-map instance, and type-level hackery.
//!
//! ## Explicitly telling a HashMap what Hasher to use
//!
//! Everytime a value needs to be hashed, when inserting or querying the HashMap for example, a new
//! Hasher instance is created. (Remember that the only methods in the Hasher trait update its
//! state or return the final value.)
//!
//! As a result, instead of passing in a Hasher, we have to pass an instance of another trait,
//! `std::hash::BuildHash`. Rust's standard library currently has two implementations of that
//! trait: 
//! - `std::collections::hash_map::RandomState`, which creates instances of DefaultHasher,
//!   Rust's implementation of SIP-something using cryptographic keys to prevent denial-of-service
//!   attacks. 
//! - `std::hash::BuildHasherDefault`, which can create instances of any Hasher implementation that
//!   also implements the Default trait.
//!
//! All of the Hashers in this collection also implement Default.
//!
//! ```rust
//! use std::collections::HashMap;
//! use std::hash::BuildHasherDefault;
//!
//! use hashers::fx_hash::FxHasher;
//!
//! // BuildHasherDefault also implements Default---it's not really interesting.
//! let mut map =
//!   HashMap::with_hasher( BuildHasherDefault::<FxHasher>::default() );
//!
//! map.insert(1, 2);
//! assert_eq!(map.get(&1), Some(&2));
//! ```
//!
//! ## Using types to specify what Hasher to use
//!
//! As an alternative, HashMap has three type-level parameters: the type of keys, the type of
//! values, and a type implementing `std::hash::BuildHash`. By default, the latter is
//! `RandomState`, which securely creates DefaultHashers. By replacing RandomState, the Hashers
//! used by the map can be determined by the HashMap's concrete type.
//! `std::hash::BuildHasherDefault` is useful here, as well.
//!
//! ```rust
//! use std::collections::HashMap;
//! use std::hash::BuildHasherDefault;
//!
//! use hashers::fnv::FNV1aHasher64;
//!
//! // This could be more complicated.
//! fn gimmie_a_map() -> HashMap<i32,i32,BuildHasherDefault<FNV1aHasher64>> {
//!     HashMap::default()
//! }
//!
//! let mut map = gimmie_a_map();
//!
//! map.insert(1,2);
//! assert_eq!(map.get(&1), Some(&2));
//! ```
//!
//! A more complicated example is the anagrams-hashmap.rs example program included with this
//! module.
//!
//! # About this crate
//!
//! This collection of Hashers is based on:
//! - http://www.cse.yorku.ca/~oz/hash.html Oz's Hash functions. (oz)
//! - http://www.burtleburtle.net/bob/hash/doobs.html Bob Jenkins'
//!   (updated) 1997 Dr. Dobbs article. (jenkins)
//! - http://burtleburtle.net/bob/hash/spooky.html Jenkin's SpookyHash. (jenkins::spooky_hash)
//! - Rust's builtin DefaultHasher (SIP 1-3?) (default)
//! - https://github.com/cbreeden/fxhash A fast, non-secure, hashing algorithm derived from an
//!   internal hasher in FireFox. (fx_hash)
//! - http://www.isthe.com/chongo/tech/comp/fnv/ The Fowler/Noll/Vo hash algorithm. (fnv)
//! - Two "null" hashers: NullHasher returns 0 for all inputs and PassThroughHasher returns the
//!   last 8 bytes of the data.
//!
//! Each sub-module implements one or more Hashers plus a minimal testing module. As well, the
//! module has a benchmarking module for comparing the Hashers and some example programs using
//! statistical tests to prod the various Hashers.
//!
//! # Example programs
//!
//! ## chi2
//!
//! > The chi-squared test is used to determine whether there is a significant difference between
//! > the expected frequencies and the observed frequencies in one or more categories. --
//! > [Chi-squared test](https://en.wikipedia.org/wiki/Chi-squared_test)
//!
//! This program attempts to compute the hash values for one of a number of data sets, then
//! simulates using those values in a 128-bucket hash table (a 2^7 - 1 mask) and tries to determine
//! if the hash buckets are uniformly distributed. I think. I'm not a statistician and apparently
//! not much of a programmer any more. Sorry.
//!
//! Anyway, it shows what may be the chi2 test of the lower bits of the hash values for a number of
//! samples and for each Hasher. Numbers closer to 0 are better, and between 3.0 and -3.0 are
//! apparently "ok." Maybe.
//!
//! The samples are:
//! - 1000 uniformly distributed 6-byte binary values.
//! - 1000 uniformly distributed 6-byte alphanumeric (ASCII) values.
//! - 1000 generated identifiers of the form 'annnnn'.
//! - The words from data/words.txt
//!
//! ## kolmogorov-smirnov
//!
//! > The Kolmogorov–Smirnov statistic quantifies a distance
//! > between the empirical distribution function of the
//! > sample and the cumulative distribution function of
//! > the reference distribution. -- [Kolmogorov–Smirnov
//! > test](https://en.wikipedia.org/wiki/Kolmogorov%E2%80%93Smirnov_test).
//!
//! Ok, this one I have a bit more confidence in. It hashes the same samples as the chi2 program,
//! then attempts to determine how far from uniformly distributed the 64-bit hash values are,
//! reporting values between 0.0 and 1.0. Lower values are better. 32-bit hashes like DJB2
//! trivially fail this test, though, although they may be fine for HashMaps with much less than 2^32
//! entries.
//!
//! ## anagrams-hashmap
//!
//! This program finds the number of words that can be made from the letters
//! "asdwtribnowplfglewhqagnbe", based on the anagrams dictionary in data/anadict.txt. (There are
//! 7440 of them.) It uses implementations of HashMap and HashSet parameterized by Hashers, and
//! reports the time taken by each hasher as well as a comparison with DefaultHasher.
//!
//! For more information, check out my ancient series of blog posts:
//! - https://maniagnosis.crsr.net/2013/02/creating-letterpress-cheating-program.html
//! - https://maniagnosis.crsr.net/2014/01/letterpress-cheating-in-rust-09.html
//! - https://maniagnosis.crsr.net/2016/01/letterpress-cheating-in-rust-16-how.html
//! And others.

extern crate fxhash;

// ====================================
// Utilities

/// Load an integer of the desired type from a byte stream, in LE order. Uses
/// `copy_nonoverlapping` to let the compiler generate the most efficient way
/// to load it from a possibly unaligned address.
///
/// Unsafe because: unchecked indexing at `i..i+size_of<int_ty>`.
///
/// **WARNING:** The user is responsible for ensuring that
/// `$buf[$i..$i+size_of<$int_ty>]` is valid.
///
/// Found this on the 'net somewhere.
macro_rules! load_int_le {
    ($buf:expr, $i:expr, $int_ty:ident) => {{
        unsafe {
            debug_assert!($i + mem::size_of::<$int_ty>() <= $buf.len());
            let mut data = 0 as $int_ty;
            ptr::copy_nonoverlapping(
                $buf.get_unchecked($i),
                &mut data as *mut _ as *mut u8,
                mem::size_of::<$int_ty>(),
            );
            data.to_le()
        }
    }};
}

// This is how I might have done it.
// macro_rules! bytes_to {
//     ($slice:ident, $offset:expr, $dst_ty:ident) => {
//         unsafe {
//             *mem::transmute::<*const u8, &$dst_ty>(
//                 $slice
//                     .get_unchecked($offset..($offset + mem::size_of::<$dst_ty>()))
//                     .as_ptr(),
//             )
//         }
//     };
// }

// Create an implementation of Default for a simple type initialized
// with a constant value.
macro_rules! default_for_constant {

    ($(#[$attr:meta])* $name:ident, $default:expr) => {
        $(#[$attr])*
        impl Default for $name {
            #[inline]
            fn default() -> $name {
                $name($default)
            }
        }
    };

}

// Given a Hasher, create a single-use hash function.
macro_rules! hasher_to_fcn {

    ($(#[$attr:meta])* $name:ident, $hasher:ident) => {
        $(#[$attr])*
        #[inline]
        pub fn $name(bytes: &[u8]) -> u64 {
            let mut hasher = $hasher::default();
            hasher.write(bytes);
            hasher.finish()
        }
    };

}

// ====================================
// Hashing modules

pub mod jenkins;
pub mod pigeon;
pub mod oz;

/// For easy access, reexport the built-in hash map's DefaultHasher,
/// including a matching one-stop function.
///
/// See std::collections::hash_map::DefaultHasher.
pub mod builtin {
    use std::hash::Hasher;

    pub use std::collections::hash_map::DefaultHasher;

    hasher_to_fcn!(
        /// Provide access to the DefaultHasher in a single function.
        default,
        DefaultHasher
    );
}

/// From https://github.com/cbreeden/fxhash
/// > This hashing algorithm was extracted from the Rustc compiler. This
/// > is the same hashing algorithm used for some internal operations in
/// > FireFox. The strength of this algorithm is in hashing 8 bytes at
/// > a time on 64-bit platforms, where the FNV algorithm works on one
/// > byte at a time.
///
/// This Hasher is imported from the fxhash crate.
///
/// Ok, its is a weird one. It chomps the data in 32- or 64-
/// (or system-specific) bit bites, and is otherwise very, very
/// simple. Literally, the algorithm is based around hashing a word:
/// `rotate_left(5).bitxor(word).wrapping_mul($key)`
///
/// The complexity must be the `$key` value, right. In 64-bits, it is 0x517cc1b727220a95. What's
/// that, you ask?
///
/// ```sh
/// $ bc
/// ibase = 16
/// 517CC1B727220A95
/// 5871781006564002453
/// ...
/// scale = 15
/// (2^64) / 5871781006564002453
/// 3.141592653589793
/// ```
///
/// For those not in the bc inner circle, 0x517cc1b727220a95 = 5871781006564002453, which when
/// divided into 2^64 is 3.14159, i.e. π.
///
/// So, yeah.
///
/// The fxhash crate provides both 32- and 64-bit versions, as well as FxHasher, which uses the
/// system bit-width.
pub mod fx_hash {
    pub use fxhash::{FxHasher, FxHasher32, FxHasher64};
    use std::hash::Hasher;

    hasher_to_fcn!(fxhash, FxHasher);
    hasher_to_fcn!(fxhash32, FxHasher32);
    hasher_to_fcn!(fxhash64, FxHasher64);
}

/// Poor Hashers used for testing purposes.
///
/// These are not expected to be used. Really. They're not good.
pub mod null {
    use std::hash::Hasher;

    /// Always returns 0.
    pub struct NullHasher;

    impl Hasher for NullHasher {
        #[inline]
        fn finish(&self) -> u64 {
            0u64
        }

        #[inline]
        fn write(&mut self, _bytes: &[u8]) {
            // data, you say?
        }
    }

    impl Default for NullHasher {
        fn default() -> NullHasher {
            NullHasher
        }
    }

    hasher_to_fcn!(
        /// Provide access to NullHasher in a single call.
        null,
        NullHasher
    );

    // --------------------------------

    /// Returns the last 8 bytes of the data, as a u64.
    pub struct PassThroughHasher(u64);

    impl Hasher for PassThroughHasher {
        #[inline]
        fn finish(&self) -> u64 {
            self.0
        }

        #[inline]
        fn write(&mut self, bytes: &[u8]) {
            for byte in bytes.iter() {
                self.0 = self.0.wrapping_shl(8) + (*byte as u64);
            }
        }
    }

    /// Provide a default PassThroughHasher initialized to 0.
    default_for_constant!(PassThroughHasher, 0);

    hasher_to_fcn!(
        /// Provide access to PassThroughHasher in a single call.
        passthrough,
        PassThroughHasher
    );
}

// ====================================
// FNV-1a (64-bit)

/// The [Fowler–Noll–Vo hash function](https://en.wikipedia.org/wiki/Fowler%E2%80%93Noll%E2%80%93Vo_hash_function).
///
/// http://www.isthe.com/chongo/tech/comp/fnv/
///
/// > The basis of the FNV hash algorithm was taken from an idea sent as reviewer comments to the
/// > IEEE POSIX P1003.2 committee by Glenn Fowler and Phong Vo back in 1991. In a subsequent ballot
/// > round: Landon Curt Noll improved on their algorithm. Some people tried this hash and found that
/// > it worked rather well. In an EMail message to Landon, they named it the ``Fowler/Noll/Vo'' or
/// > FNV hash.
/// >
/// > FNV hashes are designed to be fast while maintaining a low collision rate. The FNV speed allows
/// > one to quickly hash lots of data while maintaining a reasonable collision rate. The high
/// > dispersion of the FNV hashes makes them well suited for hashing nearly identical strings such
/// > as URLs, hostnames, filenames, text, IP addresses, etc.
/// >
/// > The IETF has an informational draft on The FNV Non-Cryptographic Hash Algorithm 
///
/// This module provides both 32- and 64-bit versions of FNV-1a.
pub mod fnv {
    use std::hash::Hasher;

    macro_rules! fnv1a {
        ($name:ident, $size:ty, $fnv_prime:expr, $offset_basis:expr) => {
            pub struct $name($size);
            impl Hasher for $name {
                #[inline]
                fn finish(&self) -> u64 {
                    self.0 as u64
                }
                #[inline]
                fn write(&mut self, bytes: &[u8]) {
                    for byte in bytes.iter() {
                        self.0 = self.0 ^ (*byte as $size);
                        self.0 = self.0.wrapping_mul($fnv_prime);
                    }
                }
            }
            default_for_constant!($name, $offset_basis);
        };
    }

    fnv1a!(FNV1aHasher32, u32, 16777619, 0x811c9dc5);
    fnv1a!(FNV1aHasher64, u64, 1099511628211, 0xcbf29ce484222325);

    hasher_to_fcn!(
        /// Provide access to FNV1aHasher32 in a single call.
        fnv1a32,
        FNV1aHasher32
    );

    hasher_to_fcn!(
        /// Provide access to FNV1aHasher64 in a single call.
        fnv1a64,
        FNV1aHasher64
    );

    #[cfg(test)]
    mod fnv1a_tests {
        use super::*;

        #[test]
        fn basic() {
            assert_eq!(fnv1a64(b""), 14695981039346656037);
            assert_eq!(fnv1a64(b"a"), 12638187200555641996);
            assert_eq!(fnv1a64(b"b"), 12638190499090526629);
            assert_eq!(fnv1a64(b"ab"), 620445648566982762);
            assert_eq!(fnv1a64(b"abcd"), 18165163011005162717);
            assert_eq!(fnv1a64(b"abcdefg"), 4642726675185563447);
        }
    }
}
