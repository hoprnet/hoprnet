use std::time::Duration;

use blokli_client::api::{BlokliQueryClient, BlokliSubscriptionClient};
use hopr_api::chain::{DeployedSafe, SafeSelector};
use hopr_primitive_types::prelude::*;

use crate::{Backend, HoprBlockchainConnector, HoprBlockchainReader, errors::ConnectorError};

#[async_trait::async_trait]
impl<B, C, P, R> hopr_api::chain::ChainReadSafeOperations for HoprBlockchainConnector<C, B, P, R>
where
    B: Backend + Send + Sync + 'static,
    C: BlokliQueryClient + BlokliSubscriptionClient + Send + Sync + 'static,
    P: Send + Sync + 'static,
    R: Send + Sync,
{
    type Error = ConnectorError;

    // NOTE: these APIs can be called without calling `connect` first

    #[inline]
    async fn safe_allowance<Cy: Currency, A: Into<Address> + Send>(
        &self,
        safe_address: A,
    ) -> Result<Balance<Cy>, Self::Error> {
        HoprBlockchainReader(self.client.clone())
            .safe_allowance(safe_address)
            .await
    }

    #[inline]
    async fn safe_info(&self, selector: SafeSelector) -> Result<Option<DeployedSafe>, Self::Error> {
        HoprBlockchainReader(self.client.clone()).safe_info(selector).await
    }

    #[inline]
    async fn await_safe_deployment(
        &self,
        selector: SafeSelector,
        timeout: Duration,
    ) -> Result<DeployedSafe, Self::Error> {
        HoprBlockchainReader(self.client.clone())
            .await_safe_deployment(selector, timeout)
            .await
    }

    #[inline]
    async fn predict_module_address(
        &self,
        nonce: u64,
        owner: &Address,
        safe_address: &Address,
    ) -> Result<Address, Self::Error> {
        HoprBlockchainReader(self.client.clone())
            .predict_module_address(nonce, owner, safe_address)
            .await
    }
}

#[cfg(test)]
mod tests {
    use hopr_api::chain::ChainReadSafeOperations;
    use hopr_crypto_types::prelude::*;
    use hopr_internal_types::prelude::*;

    use super::*;
    use crate::{
        connector::tests::{MODULE_ADDR, PRIVATE_KEY_1, create_connector},
        testing::BlokliTestStateBuilder,
    };

    #[tokio::test]
    async fn connector_should_safe_allowance() -> anyhow::Result<()> {
        let account = AccountEntry {
            public_key: *OffchainKeypair::random().public(),
            chain_addr: [1u8; Address::SIZE].into(),
            entry_type: AccountType::NotAnnounced,
            safe_address: Some([2u8; Address::SIZE].into()),
            key_id: 1.into(),
        };

        let blokli_client = BlokliTestStateBuilder::default()
            .with_accounts([(account.clone(), HoprBalance::new_base(100), XDaiBalance::new_base(1))])
            .with_safe_allowances([(account.safe_address.unwrap(), HoprBalance::new_base(10000))])
            .build_static_client();

        let connector = create_connector(blokli_client)?;

        assert_eq!(
            connector.safe_allowance(account.safe_address.unwrap()).await?,
            HoprBalance::new_base(10000)
        );

        Ok(())
    }

    #[tokio::test]
    async fn connector_should_query_existing_safe() -> anyhow::Result<()> {
        let me = ChainKeypair::from_secret(&PRIVATE_KEY_1)?.public().to_address();
        let safe_addr = [1u8; Address::SIZE].into();
        let safe = DeployedSafe {
            address: safe_addr,
            owner: me,
            module: MODULE_ADDR.into(),
        };
        let blokli_client = BlokliTestStateBuilder::default()
            .with_balances([(me, XDaiBalance::new_base(10))])
            .with_deployed_safes([safe])
            .with_hopr_network_chain_info("rotsee")
            .build_dynamic_client(MODULE_ADDR.into());

        let connector = create_connector(blokli_client)?;

        assert_eq!(Some(safe), connector.safe_info(SafeSelector::Owner(me)).await?);
        assert_eq!(Some(safe), connector.safe_info(SafeSelector::Address(safe_addr)).await?);

        insta::assert_yaml_snapshot!(*connector.client.snapshot());

        Ok(())
    }

    #[tokio::test]
    async fn connector_should_predict_module_address() -> anyhow::Result<()> {
        let me = ChainKeypair::from_secret(&PRIVATE_KEY_1)?.public().to_address();
        let safe_addr = [1u8; Address::SIZE].into();
        let blokli_client = BlokliTestStateBuilder::default()
            .with_hopr_network_chain_info("rotsee")
            .build_dynamic_client(MODULE_ADDR.into());

        let connector = create_connector(blokli_client)?;

        assert_eq!(
            "0xff3dae517c13a59014c79c397de258c9557c04b8",
            connector.predict_module_address(0, &me, &safe_addr).await?.to_string()
        );

        Ok(())
    }
}
