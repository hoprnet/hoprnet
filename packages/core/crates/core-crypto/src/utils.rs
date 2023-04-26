/// Convenience method to XOR one slice onto other.
pub fn xor_inplace(a: &mut [u8], b: &[u8]) {
    let bound = a.len().min(b.len());

    // TODO: use portable_simd here
    for i in 0..bound {
        a[i] ^= b[i];
    }
}

/// Convenience function to efficiently copy slices of unequal sizes.
#[allow(dead_code)]
pub fn copy_nonequal(target: &mut [u8], source: &[u8]) {
    let sz = target.len().min(source.len());
    target[0..sz].copy_from_slice(&source[0..sz]);
}
