use std::ops::DerefMut;
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

pub async fn redeem_all_tickets<Db, F>(db: Arc<RwLock<Db>>, self_addr: &Address, onchain_tx_sender: &impl Fn(AcknowledgedTicket) -> F) -> Result<()>
    where Db: HoprCoreEthereumDbActions, F: futures::Future<Output = std::result::Result<String, String>> {
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

lazy_static::lazy_static! {
    static ref EMPTY_TX_HASH: Hash = Hash::default();
}

async fn set_being_redeemed<Db>(db: &mut Db, ack_ticket: &mut AcknowledgedTicket, tx_hash: Hash) -> Result<()>
where Db: HoprCoreEthereumDbActions {
    ack_ticket.status = BeingRedeemed { tx_hash };
    debug!("setting {} as being redeemed with TX hash {tx_hash}", ack_ticket.ticket);
    Ok(db.update_acknowledged_ticket(ack_ticket).await?)
}

pub async fn redeem_tickets_in_channel<Db, F>(db: Arc<RwLock<Db>>, channel: ChannelEntry, onchain_tx_sender: &impl Fn(AcknowledgedTicket) -> F) -> Result<()>
where Db: HoprCoreEthereumDbActions, F: futures::Future<Output = std::result::Result<String, String>> {
    let channel_id = channel.get_id();
    let mut to_redeem = Vec::new();
    {
        let mut db = db.write().await;
        let ack_tickets = db.get_acknowledged_tickets(Some(channel)).await?;
        debug!("there are {} acknowledged tickets in channel {channel_id}", ack_tickets.len());

        for mut avail_to_redeem in ack_tickets.into_iter().filter(|t| Untouched == t.status) {
            if let Err(e) = set_being_redeemed(db.deref_mut(), &mut avail_to_redeem, *EMPTY_TX_HASH).await {
                error!("failed to update state of {}: {e}", avail_to_redeem.ticket)
            } else {
                to_redeem.push(avail_to_redeem);
            }
        }
    }
    info!("{} acknowledged tickets are still available to redeem in {channel_id}", to_redeem.len());

    let mut redeemed = 0;
    for (i, ack_ticket) in to_redeem.into_iter().enumerate() {
        let ticket_id = ack_ticket.to_string();
        if let Err(e) = redeem_ticket(db.clone(), ack_ticket, onchain_tx_sender).await {
            error!("#{i} failed to redeem {ticket_id}: {e}");
        } else {
            redeemed += 1;
        }
    }

    info!("{redeemed} tickets have been sent for redeeming in channel {channel_id}");
    Ok(())
}

pub async fn redeem_ticket<Db, F>(db: Arc<RwLock<Db>>, mut ack_ticket: AcknowledgedTicket, on_chain_tx_sender: impl Fn(AcknowledgedTicket) -> F) -> Result<Hash>
where Db: HoprCoreEthereumDbActions, F: futures::Future<Output = std::result::Result<String, String>> {
    match ack_ticket.status {
        Untouched => {
            set_being_redeemed(db.write().await.deref_mut(), &mut ack_ticket, *EMPTY_TX_HASH).await?;
        },
        BeingRedeemed { tx_hash } => if tx_hash != *EMPTY_TX_HASH {
            return Err(WrongTicketState(ack_ticket))
        }
        AcknowledgedTicketStatus::BeingAggregated { .. } => return Err(WrongTicketState(ack_ticket))
    };

    debug!("sending {} for on-chain redemption", ack_ticket.ticket);
    match (on_chain_tx_sender)(ack_ticket.clone()).await {
        Ok(tx_hash_str) => {
            let tx_hash = hex::decode(tx_hash_str)
                .map_err(|_| ParseError)
                .and_then(|s| Hash::from_bytes(&s))?;

            set_being_redeemed(db.write().await.deref_mut(), &mut ack_ticket, tx_hash).await?;
            Ok(tx_hash)
        }
        Err(err) => Err(TransactionSubmissionFailed(err))
    }
}

#[cfg(test)]
mod tests {
    fn test_ticket_redeem_flow() {

    }
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use core_crypto::types::Hash;
    use core_ethereum_db::db::wasm::Database;
    use core_types::acknowledgement::wasm::AcknowledgedTicket;
    use core_types::channels::ChannelEntry;
    use utils_misc::utils::wasm::JsResult;
    use utils_types::primitives::Address;

    macro_rules! make_js_on_chain_sender {
        ($on_chain_tx_sender:expr) => {
            |ack: core_types::acknowledgement::AcknowledgedTicket| -> std::pin::Pin<Box<dyn futures::Future<Output = Result<String, String>>>> {
            Box::pin(async move {
                let serialized_ack: core_types::acknowledgement::wasm::AcknowledgedTicket = ack.into();
                match $on_chain_tx_sender.call1(&wasm_bindgen::JsValue::null(), &wasm_bindgen::JsValue::from(serialized_ack)) {
                    Ok(res) => {
                        let promise = js_sys::Promise::from(res);
                        let ret = wasm_bindgen_futures::JsFuture::from(promise)
                            .await
                            .map(|v| v.as_string().expect("on-chain ticket redeem did not yield a string"))
                            .map_err(|v| v.as_string().unwrap_or("unknown error".to_string()))?;
                        Ok(ret)
                    }
                    Err(_) => Err("failed to call on-chain redeem TX closure".to_string())
                }
            })
        }
        };
    }

    pub async fn redeem_all_tickets(db: &Database, self_addr: &Address, on_chain_tx_sender: &js_sys::Function) -> JsResult<()> {
        let js_on_chain_tx_sender = make_js_on_chain_sender!(on_chain_tx_sender);
        super::redeem_all_tickets(db.as_ref_counted().clone(), self_addr, &js_on_chain_tx_sender).await?;
        Ok(())
    }

    pub async fn redeem_tickets_in_channel(db: &Database, channel: &ChannelEntry, on_chain_tx_sender: &js_sys::Function) -> JsResult<()> {
        let js_on_chain_tx_sender = make_js_on_chain_sender!(on_chain_tx_sender);
        super::redeem_tickets_in_channel(db.as_ref_counted().clone(), channel.clone(), &js_on_chain_tx_sender).await?;
        Ok(())
    }

    pub async fn redeem_ticket(db: &Database, ack_ticket: &AcknowledgedTicket, on_chain_tx_sender: &js_sys::Function) -> JsResult<Hash> {
        let js_on_chain_tx_sender = make_js_on_chain_sender!(on_chain_tx_sender);
        Ok(super::redeem_ticket(db.as_ref_counted().clone(), ack_ticket.into(), &js_on_chain_tx_sender).await?)
    }

}