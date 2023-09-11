use std::future::Future;
use std::sync::Arc;
use async_lock::RwLock;
use core_crypto::types::Hash;
use core_ethereum_db::traits::HoprCoreEthereumDbActions;
use core_types::acknowledgement::{AcknowledgedTicket, AcknowledgedTicketStatus};
use core_types::acknowledgement::AcknowledgedTicketStatus::{BeingRedeemed, Untouched};
use core_types::channels::ChannelEntry;
use utils_log::{debug, error, info};
use utils_types::errors::GeneralError::ParseError;
use utils_types::primitives::Address;
use utils_types::traits::BinarySerializable;
use crate::errors::CoreEthereumError::{TransactionSubmissionFailed, WrongTicketState};
use crate::errors::Result;

pub async fn redeem_all_tickets<Db, F>(db: Arc<RwLock<Db>>, self_addr: &Address, onchain_tx_sender: &impl Fn(&AcknowledgedTicket) -> F) -> Result<()>
    where Db: HoprCoreEthereumDbActions, F: Future<Output = std::result::Result<String, String>> {
    let incoming_channels = db.read().await.get_channels_to(self_addr).await?;
    debug!("starting to redeem all tickets in {} incoming channels to us.", incoming_channels.len());

    for channel in incoming_channels {
        let channel_id = channel.get_id();
        if let Err(e) = redeem_tickets_in_channel(db.clone(), channel, onchain_tx_sender).await {
            error!("failed to redeem tickets in channel {channel_id}: {e}");
        }
    }

    Ok(())
}

pub async fn redeem_tickets_in_channel<Db, F>(db: Arc<RwLock<Db>>, channel: ChannelEntry, onchain_tx_sender: &impl Fn(&AcknowledgedTicket) -> F) -> Result<()>
where Db: HoprCoreEthereumDbActions, F: Future<Output = std::result::Result<String, String>> {
    let channel_id = channel.get_id();
    let mut to_redeem = Vec::new();
    {
        let mut db = db.write().await;
        let ack_tickets = db.get_acknowledged_tickets(Some(channel)).await?;
        debug!("there are {} acknowledged tickets in channel {channel_id}", ack_tickets.len());

        for mut avail_to_redeem in ack_tickets.into_iter().filter(|t| Untouched == t.status) {
            avail_to_redeem.status = BeingRedeemed { tx_hash: Hash::default() };
            if let Err(e) = db.update_acknowledged_ticket(&avail_to_redeem).await {
                error!("failed to update state of ack ticket {:?}: {e}", avail_to_redeem.ticket)
            } else {
                to_redeem.push(avail_to_redeem);
            }
        }
    }
    info!("{} acknowledged tickets are still available to redeem in {channel_id}", to_redeem.len());

    let mut redeemed = 0;
    for (i, ack_ticket) in to_redeem.into_iter().enumerate() {
        if let Err(e) = redeem_ticket(db.clone(), ack_ticket, onchain_tx_sender).await {
            error!("failed to redeem ticket #{i} in channel {channel_id}: {e}");
        } else {
            redeemed += 1;
        }
    }

    info!("{redeemed} tickets have been sent for redeeming in channel {channel_id}");
    Ok(())
}

pub async fn redeem_ticket<Db, F>(db: Arc<RwLock<Db>>, mut ticket: AcknowledgedTicket, onchain_tx_sender: impl Fn(&AcknowledgedTicket) -> F) -> Result<()>
where Db: HoprCoreEthereumDbActions, F: Future<Output = std::result::Result<String, String>> {
    let tkt_tx_hash = match ticket.status {
        Untouched => Hash::default(),
        BeingRedeemed { tx_hash } => tx_hash,
        AcknowledgedTicketStatus::BeingAggregated { .. } => return Err(WrongTicketState(ticket))
    };

    if tkt_tx_hash == Hash::default() {
        match (onchain_tx_sender)(&ticket).await {
            Ok(tx_hash) => {
                ticket.status = BeingRedeemed {
                    tx_hash: hex::decode(tx_hash)
                        .map_err(|_| ParseError)
                        .and_then(|s| Hash::from_bytes(&s))?
                };
                db.write().await.update_acknowledged_ticket(&ticket).await?;
                Ok(())
            }
            Err(err) => Err(TransactionSubmissionFailed(err))
        }
    } else {
         Err(WrongTicketState(ticket))
    }
}

#[cfg(feature = "wasm")]
pub mod wasm {

}