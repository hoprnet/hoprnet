use hopr_protocol_pix::{PixGroup, PixScalar, PixSpec};
use hopr_types::{crypto::prelude::*, primitive::prelude::Address};

#[allow(unused)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TestSpecK256;

impl PixSpec for TestSpecK256 {
    type AddressPrivateKey = ChainKeypair;
    type Cipher = ChaCha20;
    type Curve = Secp256k1;
    type DepositAddress = Address;
    type Digest = Sha3_256;
    type Pseudonym = SimplePseudonym;

    const HASH_TO_SCALAR_SUITE_ID: &'static [u8] = b"Secp256k1_XMD:SHA3-256_SSWU_RO_";

    fn group_to_deposit_address(group: PixGroup<Self>) -> Option<Self::DepositAddress> {
        PublicKey::try_from(group.to_affine()).ok().map(|pk| pk.to_address())
    }

    fn scalar_to_private_key(scalar: PixScalar<Self>) -> Option<Self::AddressPrivateKey> {
        ChainKeypair::from_secret(scalar.to_bytes().as_ref()).ok()
    }
}

#[allow(unused)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TestSpecBjj;

impl PixSpec for TestSpecBjj {
    type AddressPrivateKey = BjjKeypair;
    type Cipher = ChaCha20;
    type Curve = BabyJubJub;
    type DepositAddress = BjjPublicKey;
    type Digest = Blake3;
    type Pseudonym = SimplePseudonym;

    const HASH_TO_SCALAR_SUITE_ID: &'static [u8] = b"BabyJubJub_XMD:BLAKE3_SSWU_RO_";

    fn group_to_deposit_address(group: PixGroup<Self>) -> Option<Self::DepositAddress> {
        BjjPublicKey::try_from(group).ok()
    }

    fn scalar_to_private_key(scalar: PixScalar<Self>) -> Option<Self::AddressPrivateKey> {
        BjjKeypair::from_secret(scalar.to_bytes().as_ref()).ok()
    }
}

// Change this to run tests and benchmarks against different PixSpec implementations
pub type TestSpec = TestSpecBjj;
