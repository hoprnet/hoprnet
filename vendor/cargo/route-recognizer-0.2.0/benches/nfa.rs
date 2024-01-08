#![feature(test)]

extern crate route_recognizer;
extern crate test;

use route_recognizer::nfa::CharSet;
use std::collections::{BTreeSet, HashSet};

#[bench]
fn bench_char_set(b: &mut test::Bencher) {
    let mut set = CharSet::new();
    set.insert('p');
    set.insert('n');
    set.insert('/');

    b.iter(|| {
        assert!(set.contains('p'));
        assert!(set.contains('/'));
        assert!(!set.contains('z'));
    });
}

#[bench]
fn bench_hash_set(b: &mut test::Bencher) {
    let mut set = HashSet::new();
    set.insert('p');
    set.insert('n');
    set.insert('/');

    b.iter(|| {
        assert!(set.contains(&'p'));
        assert!(set.contains(&'/'));
        assert!(!set.contains(&'z'));
    });
}

#[bench]
fn bench_btree_set(b: &mut test::Bencher) {
    let mut set = BTreeSet::new();
    set.insert('p');
    set.insert('n');
    set.insert('/');

    b.iter(|| {
        assert!(set.contains(&'p'));
        assert!(set.contains(&'/'));
        assert!(!set.contains(&'z'));
    });
}
