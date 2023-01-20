# Hashers

This crate is a collection of hashing-related functionality, for use
with Rust's `std::collections::HashMap`, `HashSet`, and so forth.

Additionally, there are benchmarks of the hash functions and a
couple of statistical tests for hash quality.

## Disclaimer

**None** of this is *cryptographically secure*. Attempting to use this
for cryptographic purposes is not recommended. I am not a cryptographer;
I don't even play one on TV.

Many (most? all?) of these functions are not designed to prevent
collision-based denial of service attacks. Rust's default hash function
is SipHash (1-3?), which is designed for that. Many of these functions
are intended to be used for performance purposes where that form of
security is not needed.

## What's a Hasher?

A hash function, for our purposes here, is a function that takes as
input another, general, value, and returns a number that is
ideally unique to that value. This number can be used to
store the value in an array and then locate it again later
without searching the array; in other words, in O(1) time. More or
less: there are a lot of other details. For more information, see
Rust's HashMap and various information sources online.

In Rust specifically, std::hash::Hasher is a trait:

```text
pub trait Hasher {
    fn finish(&self) -> u64;
    fn write(&mut self, bytes: &[u8]);

    fn write_u8(&mut self, i: u8) { ... }
    fn write_u16(&mut self, i: u16) { ... }
    ...
}
```

Hasher has two required methods, `finish` and `write`, and default implementations of other
useful methods like `write_u8` and `write_u16`, implemented by calling `write`. In use, an
implementation of Hasher is created, data is fed to it using the various `write` methods, then
the result is returned using the `finish` method to get the hash number out.

## Using a custom hash function in Rust

Using a custom hash function with Rust's HashMap or HashSet has long been regarded as a deep
mystery. Now, I will strip back the curtains of ignorance and reveal the secrets in all their
unholy glory!

Or something like that. It's not really a big deal.

There are two ways to create a HashMap that uses a custom Hasher implementation: setting the
hasher on a hash-map instance, and type-level hackery.

### Explicitly telling a HashMap what Hasher to use

Everytime a value needs to be hashed, when inserting or querying the HashMap for example, a new
Hasher instance is created. (Remember that the only methods in the Hasher trait update its
state or return the final value.)

As a result, instead of passing in a Hasher, we have to pass an instance of another trait,
`std::hash::BuildHash`. Rust's standard library currently has two implementations of that
trait: 
- `std::collections::hash_map::RandomState`, which creates instances of DefaultHasher,
  Rust's implementation of SIP-something using cryptographic keys to prevent denial-of-service
  attacks. 
- `std::hash::BuildHasherDefault`, which can create instances of any Hasher implementation that
  also implements the Default trait.

All of the Hashers in this collection also implement Default.

```rust
use std::collections::HashMap;
use std::hash::BuildHasherDefault;

use hashers::fx_hash::FxHasher;

// BuildHasherDefault also implements Default---it's not really interesting.
let mut map =
  HashMap::with_hasher( BuildHasherDefault::<FxHasher>::default() );

map.insert(1, 2);
assert_eq!(map.get(&1), Some(&2));
```

### Using types to specify what Hasher to use

As an alternative, HashMap has three type-level parameters: the type of keys, the type of
values, and a type implementing `std::hash::BuildHash`. By default, the latter is
`RandomState`, which securely creates DefaultHashers. By replacing RandomState, the Hashers
used by the map can be determined by the HashMap's concrete type.
`std::hash::BuildHasherDefault` is useful here, as well.

```rust
use std::collections::HashMap;
use std::hash::BuildHasherDefault;

use hashers::fnv::FNV1aHasher64;

// This could be more complicated.
fn gimmie_a_map() -> HashMap<i32,i32,BuildHasherDefault<FNV1aHasher64>> {
    HashMap::default()
}

let mut map = gimmie_a_map();

map.insert(1,2);
assert_eq!(map.get(&1), Some(&2));
```

A more complicated example is the anagrams-hashmap.rs example program included with this
module.

## About this crate

This collection of Hashers is based on:
- http://www.cse.yorku.ca/~oz/hash.html Oz's Hash functions. (oz)
- http://www.burtleburtle.net/bob/hash/doobs.html Bob Jenkins'
  (updated) 1997 Dr. Dobbs article. (jenkins)
- http://burtleburtle.net/bob/hash/spooky.html Jenkin's SpookyHash. (jenkins::spooky_hash)
- Rust's builtin DefaultHasher (SIP 1-3?) (default)
- https://github.com/cbreeden/fxhash A fast, non-secure, hashing algorithm derived from an
  internal hasher in FireFox. (fx_hash)
- http://www.isthe.com/chongo/tech/comp/fnv/ The Fowler/Noll/Vo hash algorithm. (fnv)
- https://hbfs.wordpress.com/2015/11/17/and-a-good-one-hash-functions-part vi/
  Steven Pigeon's Bricolage hash algorithm.
- Two "null" hashers: NullHasher returns 0 for all inputs and PassThroughHasher returns the
  last 8 bytes of the data.

Each sub-module implements one or more Hashers plus a minimal testing module. As well, the
module has a benchmarking module for comparing the Hashers and some example programs using
statistical tests to prod the various Hashers.

## Example programs

### chi2

> The chi-squared test is used to determine whether there is a significant difference between
> the expected frequencies and the observed frequencies in one or more categories. --
> [Chi-squared test](https://en.wikipedia.org/wiki/Chi-squared_test)

This program attempts to compute the hash values for one of a number of data sets, then
simulates using those values in a 128-bucket hash table (a 2^7 - 1 mask) and tries to determine
if the hash buckets are uniformly distributed. I think. I'm not a statistician and apparently
not much of a programmer any more. Sorry.

Anyway, it shows what may be the chi2 test of the lower bits of the hash values for a number of
samples and for each Hasher. Numbers closer to 0 are better, and between 3.0 and -3.0 are
apparently "ok." Maybe.

The samples are:
- 1000 uniformly distributed 6-byte binary values.
- 1000 uniformly distributed 6-byte alphanumeric (ASCII) values.
- 1000 generated identifiers of the form 'annnnn'.
- The words from data/words.txt

### kolmogorov-smirnov

> The Kolmogorov–Smirnov statistic quantifies a distance
> between the empirical distribution function of the
> sample and the cumulative distribution function of
> the reference distribution. -- [Kolmogorov–Smirnov
> test](https://en.wikipedia.org/wiki/Kolmogorov%E2%80%93Smirnov_test).

Ok, this one I have a bit more confidence in. It hashes the same samples as the chi2 program,
then attempts to determine how far from uniformly distributed the 64-bit hash values are,
reporting values between 0.0 and 1.0. Lower values are better. 32-bit hashes like DJB2
trivially fail this test, though, although they may be fine for HashMaps with much less than 2^32
entries.

### anagrams-hashmap

This program finds the number of words that can be made from the letters
"asdwtribnowplfglewhqagnbe", based on the anagrams dictionary in data/anadict.txt. (There are
7440 of them.) It uses implementations of HashMap and HashSet parameterized by Hashers, and
reports the time taken by each hasher as well as a comparison with DefaultHasher.

For more information, check out my ancient series of blog posts:
- https://maniagnosis.crsr.net/2013/02/creating-letterpress-cheating-program.html
- https://maniagnosis.crsr.net/2014/01/letterpress-cheating-in-rust-09.html
- https://maniagnosis.crsr.net/2016/01/letterpress-cheating-in-rust-16-how.html
And others.
