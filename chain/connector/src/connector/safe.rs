use std::time::Duration;

use blokli_client::api::{BlokliQueryClient, BlokliSubscriptionClient};
use futures::{StreamExt, TryStreamExt};
use futures_time::future::FutureExt as FuturesTimeExt;
use hopr_api::chain::{DeployedSafe, SafeSelector};
use hopr_primitive_types::prelude::*;

use crate::{Backend, HoprBlockchainConnector, connector::utils::model_to_deployed_safe, errors::ConnectorError};

#[async_trait::async_trait]
impl<B, C, P, R> hopr_api::chain::ChainReadSafeOperations for HoprBlockchainConnector<C, B, P, R>
where
    B: Backend + Send + Sync + 'static,
    C: BlokliQueryClient + BlokliSubscriptionClient + Send + Sync + 'static,
    P: Send + Sync + 'static,
    R: Send + Sync,
{
    type Error = ConnectorError;

    async fn safe_allowance<Cy: Currency, A: Into<Address> + Send>(
        &self,
        address: A,
    ) -> Result<Balance<Cy>, Self::Error> {
        let address = address.into();
        if Cy::is::<WxHOPR>() {
            Ok(self
                .client
                .query_safe_allowance(&address.into())
                .await?
                .allowance
                .0
                .parse()?)
        } else if Cy::is::<XDai>() {
            Err(ConnectorError::InvalidState("cannot query allowance on xDai"))
        } else {
            Err(ConnectorError::InvalidState("unsupported currency"))
        }
    }

    async fn safe_info(&self, selector: SafeSelector) -> Result<Option<DeployedSafe>, Self::Error> {
        let selector = match selector {
            SafeSelector::Owner(owner_address) => blokli_client::api::SafeSelector::ChainKey(owner_address.into()),
            SafeSelector::Address(safe_address) => blokli_client::api::SafeSelector::SafeAddress(safe_address.into()),
        };

        if let Some(safe) = self.client.query_safe(selector).await? {
            Ok(Some(model_to_deployed_safe(safe)?))
        } else {
            Ok(None)
        }
    }

    async fn await_safe_deployment(
        &self,
        selector: SafeSelector,
        timeout: Duration,
    ) -> Result<DeployedSafe, Self::Error> {
        if let Some(safe) = self.safe_info(selector).await? {
            return Ok(safe);
        }

        let res = self
            .client
            .subscribe_safe_deployments()?
            .map_err(ConnectorError::from)
            .and_then(|safe| futures::future::ready(model_to_deployed_safe(safe)))
            .try_skip_while(|deployed_safe| futures::future::ok(!selector.satisfies(deployed_safe)))
            .take(1)
            .try_collect::<Vec<_>>()
            .timeout(futures_time::time::Duration::from(timeout))
            .await??;

        res.into_iter()
            .next()
            .ok_or(ConnectorError::InvalidState("safe deployment stream closed"))
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

        let mut connector = create_connector(blokli_client)?;
        connector.connect().await?;

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

        let mut connector = create_connector(blokli_client)?;
        connector.connect().await?;

        assert_eq!(Some(safe), connector.safe_info(SafeSelector::Owner(me)).await?);
        assert_eq!(Some(safe), connector.safe_info(SafeSelector::Address(safe_addr)).await?);

        insta::assert_yaml_snapshot!(*connector.client.snapshot());

        Ok(())
    }
}
