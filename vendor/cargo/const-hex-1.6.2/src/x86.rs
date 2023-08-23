use crate::encode_default;

#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

const CHUNK_SIZE: usize = core::mem::size_of::<__m128i>();

cpufeatures::new!(cpuid_ssse3, "sse2", "ssse3");

/// Hex encoding function using x86 intrisics.
///
/// # Safety
///
/// Assumes `output.len() == 2 * input.len()`.
pub(super) unsafe fn _encode(input: &[u8], output: &mut [u8], table: &[u8; 16]) {
    if input.len() < CHUNK_SIZE || !cpuid_ssse3::get() {
        return encode_default(input, output, table);
    }

    // Load table and construct masks.
    let hex_table = _mm_loadu_si128(table.as_ptr().cast());
    let mask_lo = _mm_set1_epi8(0x0F);
    #[allow(clippy::cast_possible_wrap)]
    let mask_hi = _mm_set1_epi8(0xF0u8 as i8);

    let input_chunks = input.chunks_exact(CHUNK_SIZE);
    let input_remainder = input_chunks.remainder();

    let mut i = 0;
    for input_chunk in input_chunks {
        // Load input bytes and mask to nibbles.
        let input_bytes = _mm_loadu_si128(input_chunk.as_ptr().cast());
        let mut lo = _mm_and_si128(input_bytes, mask_lo);
        let mut hi = _mm_srli_epi32::<4>(_mm_and_si128(input_bytes, mask_hi));

        // Lookup the corresponding ASCII hex digit for each nibble.
        lo = _mm_shuffle_epi8(hex_table, lo);
        hi = _mm_shuffle_epi8(hex_table, hi);

        // Interleave the nibbles ([hi[0], lo[0], hi[1], lo[1], ...]).
        let hex_lo = _mm_unpacklo_epi8(hi, lo);
        let hex_hi = _mm_unpackhi_epi8(hi, lo);

        // Store result into the output buffer.
        let ptr = output.as_mut_ptr().add(i);
        i = i.checked_add(CHUNK_SIZE).unwrap_unchecked();
        _mm_storeu_si128(ptr.cast(), hex_lo);

        let ptr = output.as_mut_ptr().add(i);
        i = i.checked_add(CHUNK_SIZE).unwrap_unchecked();
        _mm_storeu_si128(ptr.cast(), hex_hi);
    }

    if !input_remainder.is_empty() {
        encode_default(input_remainder, output.get_unchecked_mut(i..), table);
    }
}
