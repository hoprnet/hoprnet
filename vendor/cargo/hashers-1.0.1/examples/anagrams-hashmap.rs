// **WARNING:** This program must be compiled in --release mode, with optimizations, or it will
// take a very, very long time.

extern crate hashers;

use std::collections::{HashMap, HashSet};
use std::hash::{BuildHasher, BuildHasherDefault, Hasher};
use std::io::{BufRead, BufReader};
use std::{fs, time};

use hashers::{builtin, fnv, fx_hash, jenkins, oz, pigeon};

pub mod combinations;

/// Convert a word to a vector of sorted bytes.
fn get_letters(s: &str) -> Vec<u8> {
    let mut t: Vec<char> = s.chars().collect();
    t.sort();
    return t.iter().map(|&ch| ch as u8).collect();
}

/// Split a line into a vector of space-separated words.
fn split_words(s: &str) -> Vec<String> {
    s.split(" ").map(|w| w.to_string()).collect()
}

/// A HashMap using the Hasher provided by BuildHasher BH.
type Dictionary<BH> = HashMap<Vec<u8>, Vec<String>, BH>;
/// A HashSet of strings using the Hasher provided by BuildHasher BH.
/// 
/// Not to be confused with a swing set.
type StringSet<BH> = HashSet<String, BH>;

/// Read the anagram dictionary into a HashMap.
fn load_dictionary<H: Default + Hasher>() -> Dictionary<BuildHasherDefault<H>> {
    let file = match fs::File::open("./data/anadict.txt") {
        Ok(f) => f,
        Err(e) => panic!(e),
    };
    let buffered_file = BufReader::new(file);
    let mut map = HashMap::default();
    for line in buffered_file.lines() {
        let line = line.unwrap();
        let mut words = split_words(&line);
        let key: Vec<u8> = words.remove(0).chars().map(|ch| ch as u8).collect();
        map.insert(key, words);
    }
    return map;
}

/// Search all combinations of letters, returning a set of the matching words.
fn search<H: Default + Hasher, BH: BuildHasher>(
    letters: &[u8],
    dictionary: &Dictionary<BH>,
) -> StringSet<BuildHasherDefault<H>> {
    let mut set = HashSet::default();
    for i in 0..letters.len() + 1 {
        let mut key: Vec<u8> = vec![0; i];
        // note the internal iterator
        combinations::each_combination(letters, i, |combo| {
            for j in 0..combo.len() {
                key[j] = combo[j];
            }
            match dictionary.get(&key) {
                Some(val) => {
                    for word in val.iter() {
                        set.insert(word.clone());
                    }
                }
                None => {}
            }
        });
    }
    return set;
}

/// Return the number of words found using a particular string.
fn do_search<H: Default + Hasher>() -> usize {
    let letters = get_letters("asdwtribnowplfglewhqagnbe");
    let dictionary = load_dictionary::<H>();
    let set = search::<H, BuildHasherDefault<H>>(&letters, &dictionary);
    set.len()
}

/// Measure and print the time used by a search, using Hasher H.
///
/// Also returns the duration, to be used as a baseline for later searchs.
fn time<H: Default + Hasher>(title: &str, baseline: f64) -> f64 {
    let start = time::Instant::now();
    assert_eq!(do_search::<H>(), 7440);
    let duration = time::Instant::now().duration_since(start);
    let secs = duration.as_secs();
    let micros = duration.subsec_micros();
    let time = (secs as f64) + ((micros as f64) / 1_000_000.0);
    if baseline > 0.0 {
        let percent = ((time / baseline) * 1000.0).round() / 10.0;
        println!("{:10} {: >8.3}s ({}%)", title, time, percent);
    } else {
        println!("{:10} {: >8.3}s", title, time);
    }
    time
}

fn main() {
    let baseline = time::<builtin::DefaultHasher>("default", 0.0);
    time::<oz::DJB2Hasher>("djb2", baseline);
    time::<oz::SDBMHasher>("sdbm", baseline);
    time::<jenkins::OAATHasher>("oaat", baseline);
    time::<jenkins::Lookup3Hasher>("lookup3", baseline);
    time::<fnv::FNV1aHasher32>("fnv-1a 32", baseline);
    time::<fnv::FNV1aHasher64>("fnv-1a 64", baseline);
    time::<fx_hash::FxHasher>("fxhash", baseline);
    time::<fx_hash::FxHasher32>("fxhash32", baseline);
    time::<fx_hash::FxHasher64>("fxhash64", baseline);
    time::<jenkins::spooky_hash::SpookyHasher>("spooky", baseline);
    time::<pigeon::Bricolage>("bricolage", baseline);
}
