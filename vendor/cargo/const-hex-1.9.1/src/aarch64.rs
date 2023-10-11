#![allow(unsafe_op_in_unsafe_fn)]

use crate::generic;
use core::arch::aarch64::*;

const CHUNK_SIZE: usize = core::mem::size_of::<uint8x16_t>();

/// Hex encoding function using aarch64 intrisics.
///
/// # Safety
///
/// `output` must be a valid pointer to at least `2 * input.len()` bytes.
// SAFETY: this is only compiled when the target feature is enabled.
#[target_feature(enable = "neon")]
pub(super) unsafe fn encode<const UPPER: bool>(input: &[u8], output: *mut u8) {
    if input.len() < CHUNK_SIZE {
        return generic::encode::<UPPER>(input, output);
    }

    // Load table.
    let hex_table = vld1q_u8(super::get_chars_table::<UPPER>().as_ptr());

    let input_chunks = input.chunks_exact(CHUNK_SIZE);
    let input_remainder = input_chunks.remainder();

    let mut i = 0;
    for input_chunk in input_chunks {
        // Load input bytes and mask to nibbles.
        let input_bytes = vld1q_u8(input_chunk.as_ptr());
        let mut lo = vandq_u8(input_bytes, vdupq_n_u8(0x0F));
        let mut hi = vshrq_n_u8(input_bytes, 4);

        // Lookup the corresponding ASCII hex digit for each nibble.
        lo = vqtbl1q_u8(hex_table, lo);
        hi = vqtbl1q_u8(hex_table, hi);

        // Interleave the nibbles ([hi[0], lo[0], hi[1], lo[1], ...]).
        let hex_lo = vzip1q_u8(hi, lo);
        let hex_hi = vzip2q_u8(hi, lo);

        // Store result into the output buffer.
        vst1q_u8(output.add(i), hex_lo);
        vst1q_u8(output.add(i + CHUNK_SIZE), hex_hi);
        i += CHUNK_SIZE * 2;
    }

    if !input_remainder.is_empty() {
        generic::encode::<UPPER>(input_remainder, output.add(i));
    }
}

pub(super) use generic::decode;
