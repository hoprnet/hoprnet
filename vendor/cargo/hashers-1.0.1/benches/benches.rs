#![feature(test)]

extern crate test;

extern crate hashers;

use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;
use test::{black_box, Bencher};

use hashers::fnv::*;
use hashers::builtin::*;
use hashers::fx_hash::*;
use hashers::jenkins::spooky_hash::*;
use hashers::jenkins::*;
use hashers::null::*;
use hashers::pigeon::*;
use hashers::oz::*;

macro_rules! tiny_bench {
    ($name:ident, $fcn:ident, $hasher:ident) => {
        // hasher_to_fcn!($fcn, $hasher);
        #[bench]
        fn $name(b: &mut Bencher) {
            b.iter(|| black_box($fcn(b"abcd")))
        }
    };
}

tiny_bench!(tiny_default, default, DefaultHasher);
tiny_bench!(tiny_bricolage, bricolage, Bricolage);
tiny_bench!(tiny_djb2, djb2, DJB2Hasher);
tiny_bench!(tiny_fnv1a64, fnv1a64, FNV1aHasher64);
tiny_bench!(tiny_fxhash, fxhash, FxHasher);
tiny_bench!(tiny_fxhash32, fxhash32, FxHasher32);
tiny_bench!(tiny_fxhash64, fxhash64, FxHasher64);
tiny_bench!(tiny_lookup3, lookup3, Lookup3Hasher);
tiny_bench!(tiny_loselose, loselose, LoseLoseHasher);
tiny_bench!(tiny_oaat, oaat, OAATHasher);
tiny_bench!(tiny_passthrough, passthrough, PassThroughHasher);
tiny_bench!(tiny_sdbm, sdbm, SDBMHasher);
tiny_bench!(tiny_spooky, spooky, SpookyHasher);

macro_rules! w32_bench {
    ($name:ident, $hasher:ident, $count:expr) => {
        #[bench]
        fn $name(b: &mut Bencher) {
            b.iter(|| {
                let mut h = $hasher::default();
                for i in 0..$count {
                    h.write_i32(i);
                }
                black_box(h.finish())
            })
        }
    };
}

w32_bench!(w32_10_default, DefaultHasher, 10);
w32_bench!(w32_10_djb2, DJB2Hasher, 10);
w32_bench!(w32_10_sdbm, SDBMHasher, 10);
w32_bench!(w32_10_loselose, LoseLoseHasher, 10);
w32_bench!(w32_10_oaat, OAATHasher, 10);
w32_bench!(w32_10_lookup3, Lookup3Hasher, 10);
w32_bench!(w32_10_passthrough, PassThroughHasher, 10);
w32_bench!(w32_10_fnv1a64, FNV1aHasher64, 10);
w32_bench!(w32_10_fxhash, FxHasher, 10);
w32_bench!(w32_10_spooky, SpookyHasher, 10);
w32_bench!(w32_10_bricolage, Bricolage, 10);

w32_bench!(w32_100_default, DefaultHasher, 100);
w32_bench!(w32_100_djb2, DJB2Hasher, 100);
w32_bench!(w32_100_sdbm, SDBMHasher, 100);
w32_bench!(w32_100_loselose, LoseLoseHasher, 100);
w32_bench!(w32_100_oaat, OAATHasher, 100);
w32_bench!(w32_100_lookup3, Lookup3Hasher, 100);
w32_bench!(w32_100_passthrough, PassThroughHasher, 100);
w32_bench!(w32_100_fnv1a64, FNV1aHasher64, 100);
w32_bench!(w32_100_fxhash, FxHasher, 100);
w32_bench!(w32_100_spooky, SpookyHasher, 100);
w32_bench!(w32_100_bricolage, Bricolage, 100);

w32_bench!(w32_1000_default, DefaultHasher, 1000);
w32_bench!(w32_1000_djb2, DJB2Hasher, 1000);
w32_bench!(w32_1000_sdbm, SDBMHasher, 1000);
w32_bench!(w32_1000_loselose, LoseLoseHasher, 1000);
w32_bench!(w32_1000_oaat, OAATHasher, 1000);
w32_bench!(w32_1000_lookup3, Lookup3Hasher, 1000);
w32_bench!(w32_1000_passthrough, PassThroughHasher, 1000);
w32_bench!(w32_1000_fnv1a64, FNV1aHasher64, 1000);
w32_bench!(w32_1000_fxhash, FxHasher, 1000);
w32_bench!(w32_1000_spooky, SpookyHasher, 1000);
w32_bench!(w32_1000_bricolage, Bricolage, 1000);

macro_rules! w64_bench {
    ($name:ident, $hasher:ident, $count:expr) => {
        #[bench]
        fn $name(b: &mut Bencher) {
            b.iter(|| {
                let mut h = $hasher::default();
                for i in 0..$count {
                    h.write_i64(i);
                }
                black_box(h.finish())
            })
        }
    };
}

w64_bench!(w64_10_default, DefaultHasher, 10);
w64_bench!(w64_10_djb2, DJB2Hasher, 10);
w64_bench!(w64_10_sdbm, SDBMHasher, 10);
w64_bench!(w64_10_loselose, LoseLoseHasher, 10);
w64_bench!(w64_10_oaat, OAATHasher, 10);
w64_bench!(w64_10_lookup3, Lookup3Hasher, 10);
w64_bench!(w64_10_passthrough, PassThroughHasher, 10);
w64_bench!(w64_10_fnv1a64, FNV1aHasher64, 10);
w64_bench!(w64_10_fxhash, FxHasher, 10);
w64_bench!(w64_10_spooky, SpookyHasher, 10);
w64_bench!(w64_10_bricolage, Bricolage, 10);

w64_bench!(w64_100_default, DefaultHasher, 100);
w64_bench!(w64_100_djb2, DJB2Hasher, 100);
w64_bench!(w64_100_sdbm, SDBMHasher, 100);
w64_bench!(w64_100_loselose, LoseLoseHasher, 100);
w64_bench!(w64_100_oaat, OAATHasher, 100);
w64_bench!(w64_100_lookup3, Lookup3Hasher, 100);
w64_bench!(w64_100_passthrough, PassThroughHasher, 100);
w64_bench!(w64_100_fnv1a64, FNV1aHasher64, 100);
w64_bench!(w64_100_fxhash, FxHasher, 100);
w64_bench!(w64_100_spooky, SpookyHasher, 100);
w64_bench!(w64_100_bricolage, Bricolage, 100);

w64_bench!(w64_1000_default, DefaultHasher, 1000);
w64_bench!(w64_1000_djb2, DJB2Hasher, 1000);
w64_bench!(w64_1000_sdbm, SDBMHasher, 1000);
w64_bench!(w64_1000_loselose, LoseLoseHasher, 1000);
w64_bench!(w64_1000_oaat, OAATHasher, 1000);
w64_bench!(w64_1000_lookup3, Lookup3Hasher, 1000);
w64_bench!(w64_1000_passthrough, PassThroughHasher, 1000);
w64_bench!(w64_1000_fnv1a64, FNV1aHasher64, 1000);
w64_bench!(w64_1000_fxhash, FxHasher, 1000);
w64_bench!(w64_1000_spooky, SpookyHasher, 1000);
w64_bench!(w64_1000_bricolage, Bricolage, 1000);

fn read_words() -> Vec<String> {
    use std::fs::File;
    use std::io::prelude::*;
    use std::io::BufReader;

    let file = File::open("./data/words.txt").expect("cannot open words.txt");
    return BufReader::new(file)
        .lines()
        .map(|l| l.expect("bad read"))
        .collect();
}

macro_rules! words_bench {
    ($name:ident, $hasher:ident, $count:expr) => {
        #[bench]
        fn $name(b: &mut Bencher) {
            let words = read_words();
            b.iter(|| {
                let mut h = $hasher::default();
                for i in words.iter().take($count) {
                    h.write(i.as_bytes());
                }
                black_box(h.finish())
            })
        }
    };
}

words_bench!(words1000_default, DefaultHasher, 1000);
words_bench!(words1000_djb2, DJB2Hasher, 1000);
words_bench!(words1000_sdbm, SDBMHasher, 1000);
words_bench!(words1000_loselose, LoseLoseHasher, 1000);
words_bench!(words1000_oaat, OAATHasher, 1000);
words_bench!(words1000_lookup3, Lookup3Hasher, 1000);
words_bench!(words1000_passthrough, PassThroughHasher, 1000);
words_bench!(words1000_fnv1a64, FNV1aHasher64, 1000);
words_bench!(words1000_fxhash, FxHasher, 1000);
words_bench!(words1000_spooky, SpookyHasher, 1000);
words_bench!(words1000_bricolage, Bricolage, 1000);

macro_rules! file_bench {
    ($name:ident, $fcn:ident) => {
        #[bench]
        fn $name(b: &mut Bencher) {
            use std::fs::read;
            let file: Vec<u8> = read("./data/words.txt").expect("cannot read words.txt");
            b.iter(|| black_box($fcn(&file)))
        }
    };
}

file_bench!(file_default, default);
file_bench!(file_djb2, djb2);
file_bench!(file_sdbm, sdbm);
file_bench!(file_loselose, loselose);
file_bench!(file_oaat, oaat);
file_bench!(file_lookup3, lookup3);
file_bench!(file_passthrough, passthrough);
file_bench!(file_fnv1a64, fnv1a64);
file_bench!(file_fnv1a32, fnv1a32);
file_bench!(file_fxhash, fxhash);
file_bench!(file_spooky, spooky);
file_bench!(file_bricolage, bricolage);
