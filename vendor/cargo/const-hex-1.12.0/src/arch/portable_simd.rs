use super::generic;
use crate::get_chars_table;
use core::simd::prelude::*;
use core::slice;

type Simd = u8x16;

pub(crate) const USE_CHECK_FN: bool = true;
const CHUNK_SIZE: usize = core::mem::size_of::<Simd>();

pub(crate) unsafe fn encode<const UPPER: bool>(input: &[u8], output: *mut u8) {
    let mut i = 0;
    let (prefix, chunks, suffix) = input.as_simd::<CHUNK_SIZE>();

    // SAFETY: ensured by caller.
    unsafe { generic::encode::<UPPER>(prefix, output) };
    i += prefix.len() * 2;

    let hex_table = Simd::from_array(*get_chars_table::<UPPER>());
    for &chunk in chunks {
        // Load input bytes and mask to nibbles.
        let mut lo = chunk & Simd::splat(15);
        let mut hi = chunk >> Simd::splat(4);

        // Lookup the corresponding ASCII hex digit for each nibble.
        lo = hex_table.swizzle_dyn(lo);
        hi = hex_table.swizzle_dyn(hi);

        // Interleave the nibbles ([hi[0], lo[0], hi[1], lo[1], ...]).
        let (hex_lo, hex_hi) = Simd::interleave(hi, lo);

        // Store result into the output buffer.
        // SAFETY: ensured by caller.
        unsafe {
            hex_lo.copy_to_slice(slice::from_raw_parts_mut(output.add(i), CHUNK_SIZE));
            i += CHUNK_SIZE;
            hex_hi.copy_to_slice(slice::from_raw_parts_mut(output.add(i), CHUNK_SIZE));
            i += CHUNK_SIZE;
        }
    }

    // SAFETY: ensured by caller.
    unsafe { generic::encode::<UPPER>(suffix, output.add(i)) };
}

pub(crate) fn check(input: &[u8]) -> bool {
    let (prefix, chunks, suffix) = input.as_simd::<CHUNK_SIZE>();
    generic::check(prefix)
        && chunks.iter().all(|&chunk| {
            let valid_digit = chunk.simd_ge(Simd::splat(b'0')) & chunk.simd_le(Simd::splat(b'9'));
            let valid_upper = chunk.simd_ge(Simd::splat(b'A')) & chunk.simd_le(Simd::splat(b'F'));
            let valid_lower = chunk.simd_ge(Simd::splat(b'a')) & chunk.simd_le(Simd::splat(b'f'));
            let valid = valid_digit | valid_upper | valid_lower;
            valid.all()
        })
        && generic::check(suffix)
}

pub(crate) use generic::decode_checked;
pub(crate) use generic::decode_unchecked;
