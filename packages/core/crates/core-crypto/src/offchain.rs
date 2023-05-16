use curve25519_dalek::edwards::CompressedEdwardsY;
use ed25519_dalek::VerifyingKey;
use utils_types::traits::BinarySerializable;
use crate::errors::CryptoError;
use crate::errors::Result;

pub struct OffchainCurvePoint {
    point: CompressedEdwardsY
}

impl BinarySerializable<'_> for OffchainCurvePoint {
    const SIZE: usize = 0;

    fn from_bytes(data: &[u8]) -> utils_types::errors::Result<Self> {
        todo!()
    }

    fn to_bytes(&self) -> Box<[u8]> {
        self.point.to_bytes().into()
    }
}

impl From<OffchainPublicKey> for OffchainCurvePoint {
    fn from(value: OffchainPublicKey) -> Self {
        todo!()
    }
}

pub struct OffchainPublicKey {
    key: VerifyingKey
}

