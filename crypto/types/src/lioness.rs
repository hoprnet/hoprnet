//! This module implements a generic Lioness wide-block cipher.
//!
//! It is based on the
//! [Lioness wide-block cipher](https://www.cl.cam.ac.uk/archive/rja14/Papers/bear-lion.pdf)
//! as proposed by Ross Anderson and Eli Biham.
use std::{fmt::Formatter, marker::PhantomData, ops::Sub};

#[allow(deprecated)] // Until the crate updates to newer versions of `generic-array`
use cipher::{
    AlgorithmName, ArrayLength, Block, BlockSizeUser, Iv, IvSizeUser, Key, KeyInit, KeyIvInit, KeySizeUser,
    StreamCipher, generic_array::GenericArray, inout::InOut,
};
use digest::{Digest, OutputSizeUser};
use typenum::{B1, Diff, IsEqual, IsGreater, Unsigned};

use crate::crypto_traits::PRP;

/// Implementation of [Lioness wide-block cipher](https://www.cl.cam.ac.uk/archive/rja14/Papers/bear-lion.pdf) over a keyed [`Digest`] and a [`StreamCipher`].
///
/// ## Requirements
/// - The output size of the `Digest` `H` must match the key size of the `StreamCipher`.
/// - The key size of the keyed digest `H` must be equal to the key size of the `StreamCipher`.
/// - The block size `B` can be arbitrary but must be strictly greater than the key size of the `StreamCipher`.
/// - However, for cryptographic security, `B` must be at least twice the key size of the `StreamCipher`.
///
/// The key size of the Lioness cipher is 4-times the size of `StreamCipher`'s
/// key.
///
/// The IV size of the Lioness cipher is 2-times the size of the `StreamCipher`'s
/// IV size.
#[derive(Clone, zeroize::ZeroizeOnDrop)]
pub struct Lioness<H: KeySizeUser + OutputSizeUser, S: KeySizeUser + IvSizeUser, B: ArrayLength<u8>>
where
    // OutputSize of the digest must be
    // equal to the KeySize of the
    // stream cipher
    H::OutputSize: IsEqual<S::KeySize, Output = B1>,
    // KeySize of the digest must be equal to the KeySize of the stream cipher
    H::KeySize: IsEqual<S::KeySize, Output = B1>,
    // BlockSize must be greater or equal to the KeySize of the stream cipher
    B: IsGreater<<S as KeySizeUser>::KeySize, Output = B1>,
{
    k1: GenericArray<u8, S::KeySize>,
    k2: GenericArray<u8, H::KeySize>,
    k3: GenericArray<u8, S::KeySize>,
    k4: GenericArray<u8, H::KeySize>,
    iv1: GenericArray<u8, S::IvSize>,
    iv2: GenericArray<u8, S::IvSize>,
    _phantom: PhantomData<(H, S, B)>,
}

impl<H: KeySizeUser + OutputSizeUser, S: KeySizeUser + IvSizeUser, B: ArrayLength<u8>> KeySizeUser for Lioness<H, S, B>
where
    H::OutputSize: IsEqual<S::KeySize, Output = B1>,
    H::KeySize: IsEqual<S::KeySize, Output = B1>,
    B: IsGreater<<S as KeySizeUser>::KeySize, Output = B1>,
    // OutputSize must allow multiplication by U4
    H::OutputSize: std::ops::Mul<cipher::consts::U4>,
    // The product of OutputSize and U4 must be an array length
    <H::OutputSize as std::ops::Mul<cipher::consts::U4>>::Output: ArrayLength<u8>,
{
    type KeySize = typenum::Prod<H::OutputSize, cipher::consts::U4>;
}

impl<H: KeySizeUser + OutputSizeUser, S: KeySizeUser + IvSizeUser, B: ArrayLength<u8>> IvSizeUser for Lioness<H, S, B>
where
    H::OutputSize: IsEqual<S::KeySize, Output = B1>,
    H::KeySize: IsEqual<S::KeySize, Output = B1>,
    B: IsGreater<<S as KeySizeUser>::KeySize, Output = B1>,
    // IvSize must allow multiplication by U2
    S::IvSize: std::ops::Mul<cipher::consts::U2>,
    // The product of IvSize with U2 must be an array length
    <S::IvSize as std::ops::Mul<cipher::consts::U2>>::Output: ArrayLength<u8>,
{
    type IvSize = typenum::Prod<S::IvSize, cipher::consts::U2>;
}

impl<H: KeySizeUser + OutputSizeUser, S: KeySizeUser + IvSizeUser, B: ArrayLength<u8>> BlockSizeUser
    for Lioness<H, S, B>
where
    H::OutputSize: IsEqual<S::KeySize, Output = B1>,
    H::KeySize: IsEqual<S::KeySize, Output = B1>,
    B: IsGreater<<S as KeySizeUser>::KeySize, Output = B1>,
    H::OutputSize: std::ops::Mul<cipher::consts::U4>,
    S::IvSize: std::ops::Mul<cipher::consts::U2>,
    <H::OutputSize as std::ops::Mul<cipher::consts::U4>>::Output: ArrayLength<u8>,
    <S::IvSize as std::ops::Mul<cipher::consts::U2>>::Output: ArrayLength<u8>,
{
    type BlockSize = B;
}

impl<H: KeySizeUser + OutputSizeUser, S: KeySizeUser + IvSizeUser, B: ArrayLength<u8>> KeyIvInit for Lioness<H, S, B>
where
    H::OutputSize: IsEqual<S::KeySize, Output = B1>,
    H::KeySize: IsEqual<S::KeySize, Output = B1>,
    B: IsGreater<<S as KeySizeUser>::KeySize, Output = B1>,
    H::OutputSize: std::ops::Mul<cipher::consts::U4>,
    S::IvSize: std::ops::Mul<cipher::consts::U2>,
    <H::OutputSize as std::ops::Mul<cipher::consts::U4>>::Output: ArrayLength<u8>,
    <S::IvSize as std::ops::Mul<cipher::consts::U2>>::Output: ArrayLength<u8>,
{
    fn new(key: &Key<Self>, iv: &Iv<Self>) -> Self {
        let k = H::OutputSize::to_usize();
        let i = S::IvSize::to_usize();
        Self {
            k1: GenericArray::clone_from_slice(&key[0..k]),
            k2: GenericArray::clone_from_slice(&key[k..2 * k]),
            k3: GenericArray::clone_from_slice(&key[2 * k..3 * k]),
            k4: GenericArray::clone_from_slice(&key[3 * k..4 * k]),
            iv1: GenericArray::clone_from_slice(&iv[0..i]),
            iv2: GenericArray::clone_from_slice(&iv[i..2 * i]),
            _phantom: Default::default(),
        }
    }
}

impl<H: KeySizeUser + OutputSizeUser + AlgorithmName, S: AlgorithmName + KeySizeUser + IvSizeUser, B: ArrayLength<u8>>
    AlgorithmName for Lioness<H, S, B>
where
    H::OutputSize: IsEqual<<S as KeySizeUser>::KeySize, Output = B1>,
    H::KeySize: IsEqual<S::KeySize, Output = B1>,
    B: IsGreater<<S as KeySizeUser>::KeySize, Output = B1>,
{
    fn write_alg_name(f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("Lioness<")?;
        H::write_alg_name(f)?;
        f.write_str(", ")?;
        S::write_alg_name(f)?;
        f.write_str(">")
    }
}

impl<H: Digest + KeyInit, S: StreamCipher + KeyIvInit, B: ArrayLength<u8>> Lioness<H, S, B>
where
    H::OutputSize: IsEqual<<S as KeySizeUser>::KeySize, Output = B1>,
    H::KeySize: IsEqual<S::KeySize, Output = B1>,
    // BlockSize must be greater than KeySize of the stream cipher, and they must be subtractable
    B: IsGreater<<S as KeySizeUser>::KeySize, Output = B1> + Sub<<S as KeySizeUser>::KeySize>,
    // The difference of BlockSize minus KeySize must be an array length
    <B as Sub<<S as KeySizeUser>::KeySize>>::Output: ArrayLength<u8>,
    H::OutputSize: std::ops::Mul<cipher::consts::U4>,
    <S as IvSizeUser>::IvSize: std::ops::Mul<cipher::consts::U2>,
    <H::OutputSize as std::ops::Mul<cipher::consts::U4>>::Output: ArrayLength<u8>,
    <<S as IvSizeUser>::IvSize as std::ops::Mul<cipher::consts::U2>>::Output: ArrayLength<u8>,
{
    const K: usize = <S as KeySizeUser>::KeySize::USIZE;

    /// Performs encryption of the given `block`.
    pub fn encrypt_block(&self, mut block: InOut<'_, '_, Block<Self>>) {
        // L' = L ^ K1
        let mut left_prime =
            GenericArray::<u8, <S as KeySizeUser>::KeySize>::clone_from_slice(&block.get_in()[0..Self::K]);
        for i in 0..Self::K {
            left_prime[i] ^= self.k1[i];
        }

        // R = R ^ S(L', IV1)
        let r_cpy = GenericArray::<u8, Diff<B, <S as KeySizeUser>::KeySize>>::clone_from_slice(
            &block.get_in()[Self::K..B::USIZE],
        );
        S::new(&left_prime, &self.iv1)
            .apply_keystream_b2b(&r_cpy, &mut block.get_out()[Self::K..B::USIZE])
            .expect("slices have always equal sizes");

        // R' = H_K2(R)
        let r_prime = <H as KeyInit>::new(&self.k2)
            .chain_update(&block.get_out()[Self::K..B::USIZE])
            .finalize();

        // L = L ^ R'
        for i in 0..Self::K {
            block.get_out()[i] = block.get_in()[i] ^ r_prime[i];
        }

        // L' = L ^ K3
        let mut left_prime =
            GenericArray::<u8, <S as KeySizeUser>::KeySize>::clone_from_slice(&block.get_out()[0..Self::K]);
        for i in 0..Self::K {
            left_prime[i] ^= self.k3[i];
        }
        // R = R ^ S(L', IV2)
        S::new(&left_prime, &self.iv2).apply_keystream(&mut block.get_out()[Self::K..B::USIZE]);

        // R' = H_K4(R)
        let r_prime = <H as KeyInit>::new(&self.k4)
            .chain_update(&block.get_out()[Self::K..B::USIZE])
            .finalize();

        // L = L ^ R'
        for i in 0..Self::K {
            block.get_out()[i] ^= r_prime[i];
        }
    }

    /// Performs decryption of the given `block`.
    pub fn decrypt_block(&self, mut block: InOut<'_, '_, Block<Self>>) {
        // R' = H(K4 || R)
        let r_prime = <H as KeyInit>::new(&self.k4)
            .chain_update(&block.get_in()[Self::K..B::USIZE])
            .finalize();

        // L = L ^ R'
        for i in 0..Self::K {
            block.get_out()[i] = block.get_in()[i] ^ r_prime[i];
        }

        // L' = L ^ K3
        let mut left_prime =
            GenericArray::<u8, <S as KeySizeUser>::KeySize>::clone_from_slice(&block.get_out()[0..Self::K]);
        for i in 0..Self::K {
            left_prime[i] ^= self.k3[i];
        }

        // R = R ^ S(L', IV2)
        let r_cpy = GenericArray::<u8, Diff<B, <S as KeySizeUser>::KeySize>>::clone_from_slice(
            &block.get_in()[Self::K..B::USIZE],
        );
        S::new(&left_prime, &self.iv2)
            .apply_keystream_b2b(&r_cpy, &mut block.get_out()[Self::K..B::USIZE])
            .expect("slices have always equal sizes");

        // R' = H(K2 || R)
        let r_prime = <H as KeyInit>::new(&self.k2)
            .chain_update(&block.get_out()[Self::K..B::USIZE])
            .finalize();

        // L = L ^ R'
        for i in 0..Self::K {
            block.get_out()[i] ^= r_prime[i];
        }

        // L' = L ^ K1
        let mut left_prime =
            GenericArray::<u8, <S as KeySizeUser>::KeySize>::clone_from_slice(&block.get_out()[0..Self::K]);
        for i in 0..Self::K {
            left_prime[i] ^= self.k1[i];
        }

        // R = R ^ S(L', IV1)
        S::new(&left_prime, &self.iv1).apply_keystream(&mut block.get_out()[Self::K..B::USIZE]);
    }
}

impl<H: Digest + KeyInit, S: StreamCipher + KeyIvInit, B: ArrayLength<u8>> PRP for Lioness<H, S, B>
where
    H::OutputSize: IsEqual<<S as KeySizeUser>::KeySize, Output = B1>,
    H::KeySize: IsEqual<S::KeySize, Output = B1>,
    B: IsGreater<<S as KeySizeUser>::KeySize, Output = B1> + Sub<<S as KeySizeUser>::KeySize>,
    <B as Sub<<S as KeySizeUser>::KeySize>>::Output: ArrayLength<u8>,
    H::OutputSize: std::ops::Mul<cipher::consts::U4>,
    <S as IvSizeUser>::IvSize: std::ops::Mul<cipher::consts::U2>,
    <H::OutputSize as std::ops::Mul<cipher::consts::U4>>::Output: ArrayLength<u8>,
    <<S as IvSizeUser>::IvSize as std::ops::Mul<cipher::consts::U2>>::Output: ArrayLength<u8>,
{
    fn forward(&self, data: &mut Block<Self>) {
        self.encrypt_block(data.into());
    }

    fn inverse(&self, data: &mut Block<Self>) {
        self.decrypt_block(data.into());
    }
}

/// Type-alias for Lioness wide-block cipher instantiated using Blake3 cryptographic hash function and ChaCha20 stream
/// cipher.
pub type LionessBlake3ChaCha20<B> = Lioness<blake3::Hasher, chacha20::ChaCha20, B>;

#[cfg(test)]
mod tests {
    use hex_literal::hex;
    use typenum::{U33, U1024};

    use super::*;

    #[test]
    fn lioness_sizes() {
        assert_eq!(
            <blake3::Hasher as OutputSizeUser>::output_size(),
            chacha20::ChaCha20::key_size()
        );

        let key_sz = LionessBlake3ChaCha20::<U33>::key_size();
        let iv_sz = LionessBlake3ChaCha20::<U33>::iv_size();
        let block_sz = LionessBlake3ChaCha20::<U33>::block_size();

        assert_eq!(key_sz, <blake3::Hasher as OutputSizeUser>::output_size() * 4);
        assert_eq!(iv_sz, chacha20::ChaCha20::iv_size() * 2);
        assert_eq!(block_sz, U33::USIZE);
    }

    #[test]
    fn lioness_forward_inverse() {
        let lioness = LionessBlake3ChaCha20::<U1024>::new(&Default::default(), &Default::default());

        let mut data = GenericArray::<u8, U1024>::default();
        let data_clone = data;
        assert_eq!(data, data_clone);

        lioness.encrypt_block((&mut data).into());
        assert_ne!(data, data_clone);

        lioness.decrypt_block((&mut data).into());
        assert_eq!(data, data_clone);
    }

    #[test]
    fn lioness_forward_kat() {
        let lioness = LionessBlake3ChaCha20::<U33>::new(&Default::default(), &Default::default());

        let mut data = GenericArray::<u8, U33>::default();

        lioness.encrypt_block((&mut data).into());
        let ka = hex!("36690b60686f3c997a7bfb3808aa18a1b5808b750587ed04a01ebd836dd3ea97b4");
        assert_eq!(data.as_slice(), &ka);
    }

    #[test]
    fn lioness_inverse_kat() {
        let lioness = LionessBlake3ChaCha20::<U33>::new(&Default::default(), &Default::default());

        let mut data = GenericArray::<u8, U33>::default();

        lioness.decrypt_block((&mut data).into());
        let ka = hex!("7857b5bb58995ac8c59eff412dad35af72a7d1e1ff1caba132aef382b15789a6cb");
        assert_eq!(data.as_slice(), &ka);
    }

    #[test]
    fn lioness_forward_inverse_random() {
        // let (k, iv) = LionessBlake3ChaCha20::<U1024>::generate_key_iv(hopr_crypto_random::rng());
        let mut k = Key::<LionessBlake3ChaCha20<U1024>>::default();
        let mut iv = Iv::<LionessBlake3ChaCha20<U1024>>::default();
        hopr_crypto_random::random_fill(&mut k);
        hopr_crypto_random::random_fill(&mut iv);

        let lioness = LionessBlake3ChaCha20::<U1024>::new(&k, &iv);

        let mut data = GenericArray::<u8, U1024>::default();
        hopr_crypto_random::random_fill(&mut data);
        let data_clone = data;
        assert_eq!(data, data_clone);

        lioness.encrypt_block((&mut data).into());
        assert_ne!(data, data_clone);

        lioness.decrypt_block((&mut data).into());
        assert_eq!(data, data_clone);
    }

    #[test]
    fn lioness_forward_inverse_random_separate_buffers() {
        // let (k, iv) = LionessBlake3ChaCha20::<U1024>::generate_key_iv(hopr_crypto_random::rng());
        let mut k = Key::<LionessBlake3ChaCha20<U1024>>::default();
        let mut iv = Iv::<LionessBlake3ChaCha20<U1024>>::default();
        hopr_crypto_random::random_fill(&mut k);
        hopr_crypto_random::random_fill(&mut iv);

        let lioness = LionessBlake3ChaCha20::<U1024>::new(&k, &iv);

        let mut data_in = GenericArray::<u8, U1024>::default();
        let mut data_out = GenericArray::<u8, U1024>::default();
        hopr_crypto_random::random_fill(&mut data_in);
        let data_orig = data_in;
        assert_eq!(data_in, data_orig);

        lioness.encrypt_block((&data_in, &mut data_out).into());
        assert_ne!(data_out, data_orig);

        let data_in = data_out;
        let mut data_out = GenericArray::<u8, U1024>::default();
        lioness.decrypt_block((&data_in, &mut data_out).into());
        assert_eq!(data_out, data_orig);
    }
}
