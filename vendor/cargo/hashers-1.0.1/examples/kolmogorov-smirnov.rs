extern crate rand;
extern crate hashers;

mod samples;

// See
// - https://www.itl.nist.gov/div898/handbook/eda/section3/eda35g.htm
// - https://onlinecourses.science.psu.edu/stat414/node/322/

/// Hash a sequence of values, returning the hashes sorted.
#[inline]
fn do_hashes(fcn: fn(&[u8]) -> u64, data: &[Vec<u8>]) -> Vec<u64> {
    let mut res: Vec<u64> = data.iter().map(|elt| fcn(elt)).collect();
    res.sort();
    res
}

/// Cumulative Distribution Function for the Uniform Distribution.
fn cdf_uniform(x: u64) -> f64 {
    // Wish we had f128s. Gonna be issues here.
    (x as f64) / (std::u64::MAX as f64)
}

/// Compute the Kolmogorov-Smirnov test.
///
/// ECDF: Experimental Cumulative Distribution Function. The distribution represented by the
/// samples.
///
/// TCDF: Theoretical Cumulative Distribution Function. The theoretical distribution to be tested
/// against; in this case the uniform distribution.
fn ks(samples: &[u64]) -> f64 {
    let n = samples.len() as f64;
    let mut last_ecdf = 0.0f64;
    let mut ks = std::f64::MIN;
    for (i, x) in samples.iter().enumerate() {
        let tcdf = (i as f64) / n;
        let next_ecdf = cdf_uniform(*x);
        let d1 = (last_ecdf - tcdf).abs();
        let d2 = (tcdf - next_ecdf).abs();
        ks = ks.max(d1.max(d2));
        last_ecdf = next_ecdf;
    }
    ks
}

fn print_ks(sample: &str, hash: &str, d: f64) {
    println!("{:10} {:10} {: <10.4}", sample, hash, d);
}

fn run_sample(name: &str, samples: &[Vec<u8>]) {
    print_ks(name, "null     ", ks(&do_hashes(hashers::null::null, samples)));
    print_ks(name, "passthru ", ks(&do_hashes(hashers::null::passthrough, samples)));
    print_ks(name, "default  ", ks(&do_hashes(hashers::builtin::default, samples)));
    print_ks(name, "loselose ", ks(&do_hashes(hashers::oz::loselose, samples)));
    print_ks(name, "sdbm     ", ks(&do_hashes(hashers::oz::sdbm, samples)));
    print_ks(name, "djb2     ", ks(&do_hashes(hashers::oz::djb2, samples)));
    print_ks(name, "oaat     ", ks(&do_hashes(hashers::jenkins::oaat, samples)));
    print_ks(name, "lookup3  ", ks(&do_hashes(hashers::jenkins::lookup3, samples)));
    print_ks(name, "fnv1a 64 ", ks(&do_hashes(hashers::fnv::fnv1a64, samples)));
    print_ks(name, "fxhash   ", ks(&do_hashes(hashers::fx_hash::fxhash, samples)));
    print_ks(name, "fxhash32 ", ks(&do_hashes(hashers::fx_hash::fxhash32, samples)));
    print_ks(name, "fxhash64 ", ks(&do_hashes(hashers::fx_hash::fxhash64, samples)));
    print_ks(name, "spooky   ", ks(&do_hashes(hashers::jenkins::spooky_hash::spooky, samples)));
    print_ks(name, "bricolage", ks(&do_hashes(hashers::pigeon::bricolage, samples)));
}

fn main() {
    run_sample("random      ", &samples::random_samples(&mut samples::uniform(), 1000, 6));
    run_sample("alphanumeric", &samples::alphanumeric_samples(10000, 6));
    run_sample("generated   ", &samples::generated_samples(10000, 6));
    run_sample("word_samples", &samples::word_samples());
}
