use rand;
use rand::distributions::{Alphanumeric, Distribution, Uniform};

// Helper to create a uniform distribution
pub fn uniform() -> Uniform<u8> {
    Uniform::new_inclusive(1u8, 255u8)
}

// Generate n samples of size s bytes from distribution dst.
pub fn random_samples<D: Distribution<u8>>(dist: &mut D, n: usize, s: usize) -> Vec<Vec<u8>> {
    (0..n)
        .map(|_| dist.sample_iter(&mut rand::thread_rng()).take(s).collect())
        .collect()
}

// Generate n samples of size s bytes from a alphanumeric uniform distribution
pub fn alphanumeric_samples(n: usize, s: usize) -> Vec<Vec<u8>> {
    (0..n)
        .map(|_| {
            Alphanumeric
                .sample_iter(&mut rand::thread_rng())
                .take(s)
                .map(|s| s as u8)
                .collect()
        })
        .collect()
}

// Generate n samples of s bytes, of the form 'an...'
pub fn generated_samples(n: usize, s: usize) -> Vec<Vec<u8>> {
    (0..n)
        .map(|v| format!("a{:0width$}", v, width = s).as_bytes().to_vec())
        .collect()
}

// Read samples from dictionary
pub fn word_samples() -> Vec<Vec<u8>> {
    use std::fs::File;
    use std::io::prelude::*;
    use std::io::BufReader;

    let file = File::open("./data/words.txt").expect("cannot open words.txt");
    return BufReader::new(file)
        .lines()
        .map(|l| l.expect("bad read"))
        .map(|l| l.as_bytes().to_vec())
        .collect();
}

