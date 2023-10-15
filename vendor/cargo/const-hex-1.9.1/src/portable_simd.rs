use crate::generic;
use core::simd::u8x16;
use core::slice;

const CHUNK_SIZE: usize = core::mem::size_of::<u8x16>();

/// Hex encoding function using [`std::simd`][core::simd].
///
/// # Safety
///
/// `output` must be a valid pointer to at least `2 * input.len()` bytes.
pub(super) unsafe fn encode<const UPPER: bool>(input: &[u8], output: *mut u8) {
    let mut i = 0;
    let (prefix, chunks, suffix) = input.as_simd::<CHUNK_SIZE>();

    // SAFETY: ensured by caller.
    unsafe { generic::encode::<UPPER>(prefix, output) };
    i += prefix.len() * 2;

    let hex_table = u8x16::from_array(*crate::get_chars_table::<UPPER>());
    for &chunk in chunks {
        // Load input bytes and mask to nibbles.
        let mut lo = chunk & u8x16::splat(15);
        let mut hi = chunk >> u8x16::splat(4);

        // Lookup the corresponding ASCII hex digit for each nibble.
        lo = hex_table.swizzle_dyn(lo);
        hi = hex_table.swizzle_dyn(hi);

        // Interleave the nibbles ([hi[0], lo[0], hi[1], lo[1], ...]).
        let (hex_lo, hex_hi) = u8x16::interleave(hi, lo);

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

pub(super) use generic::decode;
