//! Constant-time software implementation of POLYVAL for 64-bit architectures.
//! Adapted from BearSSL's `ghash_ctmul64.c`:
//!
//! <https://bearssl.org/gitweb/?p=BearSSL;a=blob;f=src/hash/ghash_ctmul64.c;hb=4b6046412>
//!
//! Copyright (c) 2016 Thomas Pornin <pornin@bolet.org>

// TODO(tarcieri): fix zeroize when we switch to ManuallyDrop on MSRV 1.49+

use crate::{Block, Key};
use core::{
    convert::TryInto,
    num::Wrapping,
    ops::{Add, Mul},
};
use universal_hash::{consts::U16, NewUniversalHash, Output, UniversalHash};

/// **POLYVAL**: GHASH-like universal hash over GF(2^128).
#[allow(non_snake_case)]
#[derive(Clone)]
#[repr(align(16))]
#[cfg_attr(
    all(
        any(target_arch = "x86", target_arch = "x86_64"),
        not(feature = "force-soft")
    ),
    derive(Copy)
)] // TODO(tarcieri): switch to ManuallyDrop on MSRV 1.49+
pub struct Polyval {
    /// GF(2^128) field element input blocks are multiplied by
    H: U64x2,

    /// Field element representing the computed universal hash
    S: U64x2,
}

impl NewUniversalHash for Polyval {
    type KeySize = U16;

    /// Initialize POLYVAL with the given `H` field element
    fn new(h: &Key) -> Self {
        Self {
            H: h.into(),
            S: U64x2::default(),
        }
    }
}

impl UniversalHash for Polyval {
    type BlockSize = U16;

    /// Input a field element `X` to be authenticated
    fn update(&mut self, x: &Block) {
        let x = U64x2::from(x);
        self.S = (self.S + x) * self.H;
    }

    /// Reset internal state
    fn reset(&mut self) {
        self.S = U64x2::default();
    }

    /// Get POLYVAL result (i.e. computed `S` field element)
    fn finalize(self) -> Output<Self> {
        let mut block = Block::default();

        for (chunk, i) in block.chunks_mut(8).zip(&[self.S.0, self.S.1]) {
            chunk.copy_from_slice(&i.to_le_bytes());
        }

        Output::new(block)
    }
}

/// 2 x `u64` values
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
struct U64x2(u64, u64);

impl From<&Block> for U64x2 {
    fn from(bytes: &Block) -> U64x2 {
        U64x2(
            u64::from_le_bytes(bytes[..8].try_into().unwrap()),
            u64::from_le_bytes(bytes[8..].try_into().unwrap()),
        )
    }
}

#[allow(clippy::suspicious_arithmetic_impl)]
impl Add for U64x2 {
    type Output = Self;

    /// Adds two POLYVAL field elements.
    fn add(self, rhs: Self) -> Self::Output {
        U64x2(self.0 ^ rhs.0, self.1 ^ rhs.1)
    }
}

#[allow(clippy::suspicious_arithmetic_impl)]
impl Mul for U64x2 {
    type Output = Self;

    /// Computes carryless POLYVAL multiplication over GF(2^128) in constant time.
    ///
    /// Method described at:
    /// <https://www.bearssl.org/constanttime.html#ghash-for-gcm>
    ///
    /// POLYVAL multiplication is effectively the little endian equivalent of
    /// GHASH multiplication, aside from one small detail described here:
    ///
    /// <https://crypto.stackexchange.com/questions/66448/how-does-bearssls-gcm-modular-reduction-work/66462#66462>
    ///
    /// > The product of two bit-reversed 128-bit polynomials yields the
    /// > bit-reversed result over 255 bits, not 256. The BearSSL code ends up
    /// > with a 256-bit result in zw[], and that value is shifted by one bit,
    /// > because of that reversed convention issue. Thus, the code must
    /// > include a shifting step to put it back where it should
    ///
    /// This shift is unnecessary for POLYVAL and has been removed.
    fn mul(self, rhs: Self) -> Self {
        let h0 = self.0;
        let h1 = self.1;
        let h0r = rev64(h0);
        let h1r = rev64(h1);
        let h2 = h0 ^ h1;
        let h2r = h0r ^ h1r;

        let y0 = rhs.0;
        let y1 = rhs.1;
        let y0r = rev64(y0);
        let y1r = rev64(y1);
        let y2 = y0 ^ y1;
        let y2r = y0r ^ y1r;
        let z0 = bmul64(y0, h0);
        let z1 = bmul64(y1, h1);

        let mut z2 = bmul64(y2, h2);
        let mut z0h = bmul64(y0r, h0r);
        let mut z1h = bmul64(y1r, h1r);
        let mut z2h = bmul64(y2r, h2r);

        z2 ^= z0 ^ z1;
        z2h ^= z0h ^ z1h;
        z0h = rev64(z0h) >> 1;
        z1h = rev64(z1h) >> 1;
        z2h = rev64(z2h) >> 1;

        let v0 = z0;
        let mut v1 = z0h ^ z2;
        let mut v2 = z1 ^ z2h;
        let mut v3 = z1h;

        v2 ^= v0 ^ (v0 >> 1) ^ (v0 >> 2) ^ (v0 >> 7);
        v1 ^= (v0 << 63) ^ (v0 << 62) ^ (v0 << 57);
        v3 ^= v1 ^ (v1 >> 1) ^ (v1 >> 2) ^ (v1 >> 7);
        v2 ^= (v1 << 63) ^ (v1 << 62) ^ (v1 << 57);

        U64x2(v2, v3)
    }
}

/// Multiplication in GF(2)[X], truncated to the low 64-bits, with “holes”
/// (sequences of zeroes) to avoid carry spilling.
///
/// When carries do occur, they wind up in a "hole" and are subsequently masked
/// out of the result.
fn bmul64(x: u64, y: u64) -> u64 {
    let x0 = Wrapping(x & 0x1111_1111_1111_1111);
    let x1 = Wrapping(x & 0x2222_2222_2222_2222);
    let x2 = Wrapping(x & 0x4444_4444_4444_4444);
    let x3 = Wrapping(x & 0x8888_8888_8888_8888);
    let y0 = Wrapping(y & 0x1111_1111_1111_1111);
    let y1 = Wrapping(y & 0x2222_2222_2222_2222);
    let y2 = Wrapping(y & 0x4444_4444_4444_4444);
    let y3 = Wrapping(y & 0x8888_8888_8888_8888);

    let mut z0 = ((x0 * y0) ^ (x1 * y3) ^ (x2 * y2) ^ (x3 * y1)).0;
    let mut z1 = ((x0 * y1) ^ (x1 * y0) ^ (x2 * y3) ^ (x3 * y2)).0;
    let mut z2 = ((x0 * y2) ^ (x1 * y1) ^ (x2 * y0) ^ (x3 * y3)).0;
    let mut z3 = ((x0 * y3) ^ (x1 * y2) ^ (x2 * y1) ^ (x3 * y0)).0;

    z0 &= 0x1111_1111_1111_1111;
    z1 &= 0x2222_2222_2222_2222;
    z2 &= 0x4444_4444_4444_4444;
    z3 &= 0x8888_8888_8888_8888;

    z0 | z1 | z2 | z3
}

/// Bit-reverse a `u64` in constant time
fn rev64(mut x: u64) -> u64 {
    x = ((x & 0x5555_5555_5555_5555) << 1) | ((x >> 1) & 0x5555_5555_5555_5555);
    x = ((x & 0x3333_3333_3333_3333) << 2) | ((x >> 2) & 0x3333_3333_3333_3333);
    x = ((x & 0x0f0f_0f0f_0f0f_0f0f) << 4) | ((x >> 4) & 0x0f0f_0f0f_0f0f_0f0f);
    x = ((x & 0x00ff_00ff_00ff_00ff) << 8) | ((x >> 8) & 0x00ff_00ff_00ff_00ff);
    x = ((x & 0xffff_0000_ffff) << 16) | ((x >> 16) & 0xffff_0000_ffff);
    (x << 32) | (x >> 32)
}
