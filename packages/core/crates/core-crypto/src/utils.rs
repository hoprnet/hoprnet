pub fn xor_inplace(a: &mut [u8], b: &[u8]) {
    let bound = if a.len() > b.len() { b.len() } else { a.len() };

    // TODO: use portable_simd here
    for i in 0..bound {
        a[i] = a[i] ^ b[i];
    }
}