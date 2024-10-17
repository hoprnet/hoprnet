use libp2p_identity::PeerId;

use crate::keypairs::OffchainKeypair;

pub fn seal_data<T: serde::Serialize>(_data: T, _peer_id: PeerId) -> crate::errors::Result<Box<[u8]>> {
    todo!()
}

pub fn unseal_data<T: for<'a> serde::Deserialize<'a>>(
    _data: &[u8],
    _key: &OffchainKeypair,
) -> crate::errors::Result<T> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keypairs::Keypair;

    #[ignore]
    #[test]
    fn seal_unseal_should_work() -> anyhow::Result<()> {
        let data = "some test data".to_string();

        let keypair = OffchainKeypair::random();

        let sealed = seal_data(data.clone(), keypair.public().into())?;

        let unsealed: String = unseal_data(&sealed, &keypair)?;

        assert_eq!(data, unsealed);
        Ok(())
    }
}
