use blokli_client::api::{BlokliQueryClient, BlokliTransactionClient};
use futures::{FutureExt, TryFutureExt, future::BoxFuture};
use hopr_api::chain::{ChainReceipt, TicketRedeemError};
use hopr_chain_types::prelude::*;
use hopr_crypto_types::prelude::*;
use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::HoprBalance;

use crate::{backend::Backend, connector::HoprBlockchainConnector, errors::ConnectorError};

impl<B, C, P> HoprBlockchainConnector<C, B, P, P::TxRequest>
where
    B: Backend + Send + Sync + 'static,
    C: BlokliTransactionClient + BlokliQueryClient + Send + Sync + 'static,
    P: PayloadGenerator + Send + Sync + 'static,
    P::TxRequest: Send + Sync + 'static,
{
    async fn prepare_ticket_redeem_payload(&self, ticket: RedeemableTicket) -> Result<P::TxRequest, ConnectorError> {
        self.check_connection_state()?;

        let channel_id = *ticket.ticket.channel_id();
        if generate_channel_id(ticket.ticket.verified_issuer(), self.chain_key.public().as_ref()) != channel_id {
            tracing::error!(%channel_id, "redeemed ticket is not ours");
            return Err(ConnectorError::InvalidTicket);
        }

        // Do a fresh Blokli query to ensure the channel is not closed.
        let channel = self
            .client
            .query_channels(blokli_client::api::ChannelSelector {
                filter: blokli_client::api::ChannelFilter::ChannelId(ticket.ticket.channel_id().into()),
                status: None,
            })
            .await?
            .first()
            .cloned()
            .ok_or_else(|| {
                tracing::error!(%channel_id, "trying to redeem a ticket on a channel that does not exist");
                ConnectorError::ChannelDoesNotExist(channel_id)
            })?;

        if channel.status == blokli_client::api::types::ChannelStatus::Closed {
            tracing::error!(%channel_id, "trying to redeem a ticket on a closed channel");
            return Err(ConnectorError::ChannelClosed(channel_id));
        }

        if channel.epoch as u32 != ticket.verified_ticket().channel_epoch {
            tracing::error!(
                channel_epoch = channel.epoch,
                ticket_epoch = ticket.verified_ticket().channel_epoch,
                "invalid redeemed ticket epoch"
            );
            return Err(ConnectorError::InvalidTicket);
        }

        let channel_index: u64 = channel
            .ticket_index
            .0
            .parse()
            .map_err(|e| ConnectorError::TypeConversion(format!("unparseable channel index at redemption: {e}")))?;

        if channel_index > ticket.verified_ticket().index {
            tracing::error!(
                channel_index,
                ticket_index = ticket.verified_ticket().index,
                "invalid redeemed ticket index"
            );
            return Err(ConnectorError::InvalidTicket);
        }

        let channel_stake: HoprBalance = channel
            .balance
            .0
            .parse()
            .map_err(|e| ConnectorError::TypeConversion(format!("unparseable channel stake at redemption: {e}")))?;

        if channel_stake < ticket.verified_ticket().amount {
            tracing::error!(
                %channel_stake,
                ticket_amount = %ticket.verified_ticket().amount,
                "insufficient stake in channel to redeem ticket"
            );
            return Err(ConnectorError::InvalidTicket);
        }

        Ok(self.payload_generator.redeem_ticket(ticket)?)
    }
}

#[async_trait::async_trait]
impl<B, C, P> hopr_api::chain::ChainWriteTicketOperations for HoprBlockchainConnector<C, B, P, P::TxRequest>
where
    B: Backend + Send + Sync + 'static,
    C: BlokliTransactionClient + BlokliQueryClient + Send + Sync + 'static,
    P: PayloadGenerator + Send + Sync + 'static,
    P::TxRequest: Send + Sync + 'static,
{
    type Error = ConnectorError;

    async fn redeem_ticket<'a>(
        &'a self,
        ticket: RedeemableTicket,
    ) -> Result<
        BoxFuture<'a, Result<(VerifiedTicket, ChainReceipt), TicketRedeemError<Self::Error>>>,
        TicketRedeemError<Self::Error>,
    > {
        match self.prepare_ticket_redeem_payload(ticket).await {
            Ok(tx_req) => {
                Ok(self
                    .send_tx(tx_req)
                    .await
                    .map_err(|e| TicketRedeemError::ProcessingError(ticket.ticket, e))?
                    .map_err(move |tx_tracking_error|
                        // For ticket redemption, certain errors are to be handled differently
                        if let Some(reject_error) = tx_tracking_error.as_transaction_rejection_error() {
                            TicketRedeemError::Rejected(ticket.ticket, format!("on-chain rejection: {reject_error:?}"))
                        } else {
                            TicketRedeemError::ProcessingError(ticket.ticket, tx_tracking_error)
                        })
                    .and_then(move |receipt| futures::future::ok((ticket.ticket, receipt)))
                    .boxed())
            }
            Err(e @ ConnectorError::InvalidTicket)
            | Err(e @ ConnectorError::ChannelDoesNotExist(_))
            | Err(e @ ConnectorError::ChannelClosed(_)) => {
                Err(TicketRedeemError::Rejected(ticket.ticket, e.to_string()))
            }
            Err(e) => Err(TicketRedeemError::ProcessingError(ticket.ticket, e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use blokli_client::BlokliTestClient;
    use hex_literal::hex;
    use hopr_api::chain::{ChainWriteChannelOperations, ChainWriteTicketOperations};
    use hopr_primitive_types::prelude::*;

    use super::*;
    use crate::{
        connector::tests::*,
        testing::{BlokliTestStateBuilder, FullStateEmulator},
    };

    fn prepare_client() -> anyhow::Result<BlokliTestClient<FullStateEmulator>> {
        let offchain_key_1 = OffchainKeypair::from_secret(&hex!(
            "60741b83b99e36aa0c1331578156e16b8e21166d01834abb6c64b103f885734d"
        ))?;
        let account_1 = AccountEntry {
            public_key: *offchain_key_1.public(),
            chain_addr: ChainKeypair::from_secret(&PRIVATE_KEY_1)?.public().to_address(),
            entry_type: AccountType::NotAnnounced,
            safe_address: Some([1u8; Address::SIZE].into()),
            key_id: 1.into(),
        };
        let offchain_key_2 = OffchainKeypair::from_secret(&hex!(
            "71bf1f42ebbfcd89c3e197a3fd7cda79b92499e509b6fefa0fe44d02821d146a"
        ))?;
        let account_2 = AccountEntry {
            public_key: *offchain_key_2.public(),
            chain_addr: ChainKeypair::from_secret(&PRIVATE_KEY_2)?.public().to_address(),
            entry_type: AccountType::NotAnnounced,
            safe_address: Some([2u8; Address::SIZE].into()),
            key_id: 2.into(),
        };

        let channel_1 = ChannelEntry::new(
            ChainKeypair::from_secret(&PRIVATE_KEY_2)?.public().to_address(),
            ChainKeypair::from_secret(&PRIVATE_KEY_1)?.public().to_address(),
            10.into(),
            1,
            ChannelStatus::Open,
            1,
        );

        Ok(BlokliTestStateBuilder::default()
            .with_accounts([
                (account_1, HoprBalance::new_base(100), XDaiBalance::new_base(1)),
                (account_2, HoprBalance::new_base(100), XDaiBalance::new_base(1)),
            ])
            .with_channels([channel_1])
            .with_hopr_network_chain_info("rotsee")
            .build_dynamic_client(MODULE_ADDR.into())
            .with_tx_simulation_delay(Duration::from_millis(100)))
    }

    #[tokio::test]
    async fn connector_should_redeem_ticket() -> anyhow::Result<()> {
        let blokli_client = prepare_client()?;

        let mut connector = create_connector(blokli_client)?;
        connector.connect().await?;

        let hkc1 = ChainKeypair::from_secret(&hex!(
            "e17fe86ce6e99f4806715b0c9412f8dad89334bf07f72d5834207a9d8f19d7f8"
        ))?;
        let hkc2 = ChainKeypair::from_secret(&hex!(
            "492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775"
        ))?;

        let ticket = TicketBuilder::default()
            .counterparty(&ChainKeypair::from_secret(&PRIVATE_KEY_1)?)
            .amount(1)
            .index(1)
            .channel_epoch(1)
            .eth_challenge(
                Challenge::from_hint_and_share(
                    &HalfKeyChallenge::new(hkc1.public().as_ref()),
                    &HalfKeyChallenge::new(hkc2.public().as_ref()),
                )?
                .to_ethereum_challenge(),
            )
            .build_signed(&ChainKeypair::from_secret(&PRIVATE_KEY_2)?, &Hash::default())?
            .into_acknowledged(Response::from_half_keys(
                &HalfKey::try_from(hkc1.secret().as_ref())?,
                &HalfKey::try_from(hkc2.secret().as_ref())?,
            )?)
            .into_redeemable(&ChainKeypair::from_secret(&PRIVATE_KEY_1)?, &Hash::default())?;

        connector.redeem_ticket(ticket).await?.await?;

        insta::assert_yaml_snapshot!(*connector.client().snapshot());

        Ok(())
    }

    #[tokio::test]
    async fn connector_should_not_redeem_ticket_on_non_existing_channel() -> anyhow::Result<()> {
        let blokli_client = prepare_client()?;

        let mut connector = create_connector(blokli_client)?;
        connector.connect().await?;

        let hkc1 = ChainKeypair::from_secret(&hex!(
            "e17fe86ce6e99f4806715b0c9412f8dad89334bf07f72d5834207a9d8f19d7f8"
        ))?;
        let hkc2 = ChainKeypair::from_secret(&hex!(
            "492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775"
        ))?;

        let ticket = TicketBuilder::default()
            .counterparty(&ChainKeypair::from_secret(&PRIVATE_KEY_2)?)
            .amount(1)
            .index(1)
            .channel_epoch(1)
            .eth_challenge(
                Challenge::from_hint_and_share(
                    &HalfKeyChallenge::new(hkc1.public().as_ref()),
                    &HalfKeyChallenge::new(hkc2.public().as_ref()),
                )?
                .to_ethereum_challenge(),
            )
            .build_signed(&ChainKeypair::from_secret(&PRIVATE_KEY_1)?, &Hash::default())?
            .into_acknowledged(Response::from_half_keys(
                &HalfKey::try_from(hkc1.secret().as_ref())?,
                &HalfKey::try_from(hkc2.secret().as_ref())?,
            )?)
            .into_redeemable(&ChainKeypair::from_secret(&PRIVATE_KEY_2)?, &Hash::default())?;

        assert!(matches!(
            connector.redeem_ticket(ticket).await,
            Err(TicketRedeemError::Rejected(_, _))
        ));

        insta::assert_yaml_snapshot!(*connector.client().snapshot());

        Ok(())
    }

    #[tokio::test]
    async fn connector_should_not_redeem_ticket_on_closed_channel() -> anyhow::Result<()> {
        let blokli_client = prepare_client()?;

        let mut connector = create_connector(blokli_client)?;
        connector.connect().await?;

        let hkc1 = ChainKeypair::from_secret(&hex!(
            "e17fe86ce6e99f4806715b0c9412f8dad89334bf07f72d5834207a9d8f19d7f8"
        ))?;
        let hkc2 = ChainKeypair::from_secret(&hex!(
            "492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775"
        ))?;

        let ticket = TicketBuilder::default()
            .counterparty(&ChainKeypair::from_secret(&PRIVATE_KEY_1)?)
            .amount(1)
            .index(1)
            .channel_epoch(1)
            .eth_challenge(
                Challenge::from_hint_and_share(
                    &HalfKeyChallenge::new(hkc1.public().as_ref()),
                    &HalfKeyChallenge::new(hkc2.public().as_ref()),
                )?
                .to_ethereum_challenge(),
            )
            .build_signed(&ChainKeypair::from_secret(&PRIVATE_KEY_2)?, &Hash::default())?
            .into_acknowledged(Response::from_half_keys(
                &HalfKey::try_from(hkc1.secret().as_ref())?,
                &HalfKey::try_from(hkc2.secret().as_ref())?,
            )?)
            .into_redeemable(&ChainKeypair::from_secret(&PRIVATE_KEY_1)?, &Hash::default())?;

        // Close the channel from the incoming side
        connector.close_channel(ticket.ticket.channel_id()).await?.await?;

        assert!(matches!(
            connector.redeem_ticket(ticket).await,
            Err(TicketRedeemError::Rejected(_, _))
        ));

        insta::assert_yaml_snapshot!(*connector.client().snapshot());

        Ok(())
    }

    #[tokio::test]
    async fn connector_should_not_redeem_ticket_with_old_index() -> anyhow::Result<()> {
        let blokli_client = prepare_client()?;

        let mut connector = create_connector(blokli_client)?;
        connector.connect().await?;

        let hkc1 = ChainKeypair::from_secret(&hex!(
            "e17fe86ce6e99f4806715b0c9412f8dad89334bf07f72d5834207a9d8f19d7f8"
        ))?;
        let hkc2 = ChainKeypair::from_secret(&hex!(
            "492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775"
        ))?;

        let ticket = TicketBuilder::default()
            .counterparty(&ChainKeypair::from_secret(&PRIVATE_KEY_1)?)
            .amount(1)
            .index(0)
            .channel_epoch(1)
            .eth_challenge(
                Challenge::from_hint_and_share(
                    &HalfKeyChallenge::new(hkc1.public().as_ref()),
                    &HalfKeyChallenge::new(hkc2.public().as_ref()),
                )?
                .to_ethereum_challenge(),
            )
            .build_signed(&ChainKeypair::from_secret(&PRIVATE_KEY_2)?, &Hash::default())?
            .into_acknowledged(Response::from_half_keys(
                &HalfKey::try_from(hkc1.secret().as_ref())?,
                &HalfKey::try_from(hkc2.secret().as_ref())?,
            )?)
            .into_redeemable(&ChainKeypair::from_secret(&PRIVATE_KEY_1)?, &Hash::default())?;

        assert!(matches!(
            connector.redeem_ticket(ticket).await,
            Err(TicketRedeemError::Rejected(_, _))
        ));

        insta::assert_yaml_snapshot!(*connector.client().snapshot());

        Ok(())
    }

    #[tokio::test]
    async fn connector_should_not_redeem_ticket_with_amount_higher_than_channel_stake() -> anyhow::Result<()> {
        let blokli_client = prepare_client()?;

        let mut connector = create_connector(blokli_client)?;
        connector.connect().await?;

        let hkc1 = ChainKeypair::from_secret(&hex!(
            "e17fe86ce6e99f4806715b0c9412f8dad89334bf07f72d5834207a9d8f19d7f8"
        ))?;
        let hkc2 = ChainKeypair::from_secret(&hex!(
            "492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775"
        ))?;

        let ticket = TicketBuilder::default()
            .counterparty(&ChainKeypair::from_secret(&PRIVATE_KEY_1)?)
            .amount(100000)
            .index(1)
            .channel_epoch(1)
            .eth_challenge(
                Challenge::from_hint_and_share(
                    &HalfKeyChallenge::new(hkc1.public().as_ref()),
                    &HalfKeyChallenge::new(hkc2.public().as_ref()),
                )?
                .to_ethereum_challenge(),
            )
            .build_signed(&ChainKeypair::from_secret(&PRIVATE_KEY_2)?, &Hash::default())?
            .into_acknowledged(Response::from_half_keys(
                &HalfKey::try_from(hkc1.secret().as_ref())?,
                &HalfKey::try_from(hkc2.secret().as_ref())?,
            )?)
            .into_redeemable(&ChainKeypair::from_secret(&PRIVATE_KEY_1)?, &Hash::default())?;

        assert!(matches!(
            connector.redeem_ticket(ticket).await,
            Err(TicketRedeemError::Rejected(_, _))
        ));

        insta::assert_yaml_snapshot!(*connector.client().snapshot());

        Ok(())
    }

    #[tokio::test]
    async fn connector_should_not_redeem_ticket_with_previous_epoch() -> anyhow::Result<()> {
        let blokli_client = prepare_client()?;

        let mut connector = create_connector(blokli_client)?;
        connector.connect().await?;

        let hkc1 = ChainKeypair::from_secret(&hex!(
            "e17fe86ce6e99f4806715b0c9412f8dad89334bf07f72d5834207a9d8f19d7f8"
        ))?;
        let hkc2 = ChainKeypair::from_secret(&hex!(
            "492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775"
        ))?;

        let ticket = TicketBuilder::default()
            .counterparty(&ChainKeypair::from_secret(&PRIVATE_KEY_1)?)
            .amount(1)
            .index(1)
            .channel_epoch(0)
            .eth_challenge(
                Challenge::from_hint_and_share(
                    &HalfKeyChallenge::new(hkc1.public().as_ref()),
                    &HalfKeyChallenge::new(hkc2.public().as_ref()),
                )?
                .to_ethereum_challenge(),
            )
            .build_signed(&ChainKeypair::from_secret(&PRIVATE_KEY_2)?, &Hash::default())?
            .into_acknowledged(Response::from_half_keys(
                &HalfKey::try_from(hkc1.secret().as_ref())?,
                &HalfKey::try_from(hkc2.secret().as_ref())?,
            )?)
            .into_redeemable(&ChainKeypair::from_secret(&PRIVATE_KEY_1)?, &Hash::default())?;

        assert!(matches!(
            connector.redeem_ticket(ticket).await,
            Err(TicketRedeemError::Rejected(_, _))
        ));

        insta::assert_yaml_snapshot!(*connector.client().snapshot());

        Ok(())
    }
}
