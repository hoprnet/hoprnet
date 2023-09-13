use crate::errors::CoreEthereumError::{InvalidArguments, NotAWinningTicket, TransactionSubmissionFailed, WrongTicketState};
use crate::errors::Result;
use async_lock::RwLock;
use core_crypto::types::Hash;
use core_ethereum_db::traits::HoprCoreEthereumDbActions;
use core_types::acknowledgement::AcknowledgedTicketStatus::{BeingAggregated, BeingRedeemed, Untouched};
use core_types::acknowledgement::AcknowledgedTicket;
use core_types::channels::ChannelEntry;
use std::ops::DerefMut;
use std::sync::Arc;
use utils_db::errors::DbError;
use utils_log::{debug, error, info};
use utils_types::primitives::Address;
use utils_types::traits::ToHex;

lazy_static::lazy_static! {
    /// Used as a placeholder when the redeem transaction has not yet been published on-chain
    static ref EMPTY_TX_HASH: Hash = Hash::default();
}

pub async fn redeem_all_tickets<Db, F>(
    db: Arc<RwLock<Db>>,
    self_addr: &Address,
    onchain_tx_sender: &impl Fn(AcknowledgedTicket) -> F,
) -> Result<()>
where
    Db: HoprCoreEthereumDbActions,
    F: futures::Future<Output = std::result::Result<String, String>>,
{
    let incoming_channels = db.read().await.get_channels_to(self_addr).await?;
    debug!(
        "starting to redeem all tickets in {} incoming channels to us.",
        incoming_channels.len()
    );

    for channel in incoming_channels.iter() {
        let channel_id = channel.get_id();
        if let Err(e) = redeem_tickets_in_channel(db.clone(), channel, onchain_tx_sender).await {
            error!("failed to redeem tickets in channel {channel_id}: {e}");
        }
    }

    Ok(())
}

async fn set_being_redeemed<Db>(db: &mut Db, ack_ticket: &mut AcknowledgedTicket, tx_hash: Hash) -> Result<()>
where
    Db: HoprCoreEthereumDbActions,
{
    match ack_ticket.status {
        Untouched => {
            let dst = db.get_channels_domain_separator()
                .await
                .and_then(|separator| separator.ok_or(DbError::NotFound))?;

            // Check if we're going to redeem a winning ticket
            if !ack_ticket.is_winning_ticket(&dst) {
                return Err(NotAWinningTicket);
            }
        }
        BeingAggregated { .. } => return Err(WrongTicketState(ack_ticket.to_string())),
        BeingRedeemed { .. } => {} // Just let the TX hash to be updated
    }

    ack_ticket.status = BeingRedeemed { tx_hash };
    debug!("setting a winning {} as being redeemed with TX hash {tx_hash}", ack_ticket.ticket);
    Ok(db.update_acknowledged_ticket(ack_ticket).await?)

}

pub async fn redeem_tickets_with_counterparty<Db, F>(
    db: Arc<RwLock<Db>>,
    counterparty: &Address,
    onchain_tx_sender: &impl Fn(AcknowledgedTicket) -> F,
) -> Result<()>
    where
        Db: HoprCoreEthereumDbActions,
        F: futures::Future<Output = std::result::Result<String, String>>,
{
    let ch = db.read().await.get_channel_from(counterparty).await?;
    if let Some(channel) = ch {
        redeem_tickets_in_channel(db, &channel, onchain_tx_sender).await
    } else {
        Err(InvalidArguments(format!("cannot find channel with {counterparty}")))
    }
}

pub async fn redeem_tickets_in_channel<Db, F>(
    db: Arc<RwLock<Db>>,
    channel: &ChannelEntry,
    onchain_tx_sender: &impl Fn(AcknowledgedTicket) -> F,
) -> Result<()>
where
    Db: HoprCoreEthereumDbActions,
    F: futures::Future<Output = std::result::Result<String, String>>,
{
    let channel_id = channel.get_id();

    // Keep holding the DB write lock until we mark all the eligible tickets as BeginRedeemed
    let mut to_redeem = Vec::new();
    {
        let mut db = db.write().await;
        let ack_tickets = db.get_acknowledged_tickets(Some(channel.clone())).await?;
        debug!(
            "there are {} acknowledged tickets in channel {channel_id}",
            ack_tickets.len()
        );

        for mut avail_to_redeem in ack_tickets.into_iter().filter(|t| Untouched == t.status) {
            if let Err(e) = set_being_redeemed(db.deref_mut(), &mut avail_to_redeem, *EMPTY_TX_HASH).await {
                error!("failed to update state of {}: {e}", avail_to_redeem.ticket)
            } else {
                to_redeem.push(avail_to_redeem);
            }
        }
    }

    info!(
        "{} acknowledged tickets are still available to redeem in {channel_id}",
        to_redeem.len()
    );

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

pub async fn redeem_ticket<Db, F>(
    db: Arc<RwLock<Db>>,
    mut ack_ticket: AcknowledgedTicket,
    on_chain_tx_sender: impl Fn(AcknowledgedTicket) -> F,
) -> Result<Hash>
where
    Db: HoprCoreEthereumDbActions,
    F: futures::Future<Output = std::result::Result<String, String>>,
{
    match ack_ticket.status {
        Untouched => {
            set_being_redeemed(db.write().await.deref_mut(), &mut ack_ticket, *EMPTY_TX_HASH).await?;
        }
        BeingRedeemed { tx_hash } => {
            // Allow sending TX only if there's no TX hash set
            if tx_hash != *EMPTY_TX_HASH {
                return Err(WrongTicketState(ack_ticket.to_string()));
            }
        }
        BeingAggregated { .. } => return Err(WrongTicketState(ack_ticket.to_string())),
    };

    debug!("sending {} for on-chain redemption", ack_ticket.ticket);
    match (on_chain_tx_sender)(ack_ticket.clone()).await {
        Ok(tx_hash_str) => {
            debug!("on-chain redeem tx returned: {tx_hash_str}");
            let tx_hash = Hash::from_hex(&tx_hash_str)?;
            set_being_redeemed(db.write().await.deref_mut(), &mut ack_ticket, tx_hash).await?;
            Ok(tx_hash)
        }
        Err(err) => Err(TransactionSubmissionFailed(err)),
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Deref;
    use async_lock::{Mutex, RwLock};
    use core_crypto::random::random_bytes;
    use core_crypto::types::{Challenge, CurvePoint, HalfKey, Hash};
    use core_types::acknowledgement::{AcknowledgedTicket, AcknowledgedTicketStatus, UnacknowledgedTicket};
    use std::sync::Arc;
    use hex_literal::hex;
    use core_crypto::keypairs::{ChainKeypair, Keypair};
    use core_ethereum_db::db::CoreEthereumDb;
    use core_ethereum_db::traits::HoprCoreEthereumDbActions;
    use core_types::channels::{ChannelEntry, ChannelStatus, Ticket};
    use utils_db::constants::ACKNOWLEDGED_TICKETS_PREFIX;
    use utils_db::db::DB;
    use utils_db::rusty::RustyLevelDbShim;
    use utils_types::primitives::{Address, Balance, BalanceType, EthereumChallenge, Snapshot, U256};
    use utils_types::traits::{BinarySerializable, ToHex};
    use crate::redeem::redeem_all_tickets;

    lazy_static::lazy_static! {
        static ref ALICE: ChainKeypair = ChainKeypair::from_secret(&hex!("492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775")).unwrap();
        static ref BOB: ChainKeypair = ChainKeypair::from_secret(&hex!("48680484c6fc31bc881a0083e6e32b6dc789f9eaba0f8b981429fd346c697f8c")).unwrap();
    }

    fn mock_ticket(
        pk: &ChainKeypair,
        counterparty: &Address,
        domain_separator: Option<Hash>,
        challenge: Option<EthereumChallenge>,
        idx: u32,
    ) -> Ticket {
        let win_prob = 1.0f64; // 100 %
        let price_per_packet: U256 = 10000000000000000u128.into(); // 0.01 HOPR
        let path_pos = 5u64;

        Ticket::new(
            counterparty,
            &Balance::new(
                price_per_packet.divide_f64(win_prob).unwrap() * path_pos.into(),
                BalanceType::HOPR,
            ),
            idx.into(),
            U256::one(),
            1.0f64,
            4u64.into(),
            challenge.unwrap_or_default(),
            pk,
            &domain_separator.unwrap_or_default(),
        )
            .unwrap()
    }

    fn generate_random_ack_ticket(idx: u32) -> AcknowledgedTicket {
        let hk1 = HalfKey::random();
        let hk2 = HalfKey::random();

        let cp1: CurvePoint = hk1.to_challenge().into();
        let cp2: CurvePoint = hk2.to_challenge().into();
        let cp_sum = CurvePoint::combine(&[&cp1, &cp2]);

        let ticket = mock_ticket(
            &BOB,
            &ALICE.public().to_address(),
            None,
            Some(Challenge::from(cp_sum).to_ethereum_challenge()),
            idx
        );

        let unacked_ticket = UnacknowledgedTicket::new(ticket, hk1, BOB.public().to_address());
        unacked_ticket.acknowledge(&hk2, &ALICE, &Hash::default()).unwrap()
    }

    fn to_acknowledged_ticket_key(ack: &AcknowledgedTicket) -> utils_db::db::Key {
        let mut ack_key = Vec::new();

        ack_key.extend_from_slice(&ack.ticket.channel_id.to_bytes());
        ack_key.extend_from_slice(&ack.ticket.channel_epoch.to_be_bytes());
        ack_key.extend_from_slice(&ack.ticket.index.to_be_bytes());

        utils_db::db::Key::new_bytes_with_prefix(&ack_key, ACKNOWLEDGED_TICKETS_PREFIX).unwrap()
    }

    #[async_std::test]
    async fn test_ticket_redeem_flow() {
        let _ = env_logger::builder().is_test(true).try_init();

        let ticket_count: usize = 3;

        let mut inner_db = DB::new(RustyLevelDbShim::new_in_memory());
        let mut input_tickets = Vec::new();

        for i in 0..ticket_count {
            let ack_ticket = generate_random_ack_ticket(i as u32);
            inner_db.set(to_acknowledged_ticket_key(&ack_ticket), &ack_ticket).await.unwrap();
            input_tickets.push(ack_ticket);
        }

        let db = Arc::new(RwLock::new(CoreEthereumDb::new(inner_db, ALICE.public().to_address())));
        let channel = ChannelEntry::new(BOB.public().to_address(),ALICE.public().to_address(),  Balance::zero(BalanceType::HOPR), U256::zero(), ChannelStatus::Open, U256::zero(), U256::zero());
        db.write().await.update_channel_and_snapshot(&channel.get_id(), &channel, &Default::default()).await.unwrap();
        db.write().await.set_channels_domain_separator(&Hash::default(), &Snapshot::default()).await.unwrap();

        let dummy_tx_hash = Hash::new(&random_bytes::<{Hash::SIZE}>());
        let redeemed_tickets = Arc::new(Mutex::new(Vec::new()));

        let rt_clone = redeemed_tickets.clone();
        redeem_all_tickets(db.clone(), &ALICE.public().to_address(), &|ack: AcknowledgedTicket| async {
            rt_clone.lock().await.push(ack);
            Ok(dummy_tx_hash.to_hex())
        }).await.unwrap();

        let db_acks = db.read().await.get_acknowledged_tickets(Some(channel)).await.unwrap();

        assert_eq!(ticket_count, db_acks.len(), "must have {ticket_count} tickets");

        assert!(db_acks.iter().all(|t| match t.status {
            AcknowledgedTicketStatus::BeingRedeemed { tx_hash } => tx_hash == dummy_tx_hash,
            _ => false
        }), "all tickets must be in the BeingRedeemed state with correct tx hash");

        assert_eq!(db_acks.iter().map(|t| t.ticket.clone()).collect::<Vec<_>>(),
                   redeemed_tickets.lock().await.deref().iter().map(|t| t.ticket.clone()).collect::<Vec<_>>(),
                   "on-chain redeemed tickets must be equal");
    }
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use wasm_bindgen::prelude::wasm_bindgen;
    use core_crypto::types::Hash;
    use core_ethereum_db::db::wasm::Database;
    use core_types::acknowledgement::wasm::AcknowledgedTicket;
    use core_types::channels::ChannelEntry;
    use utils_misc::utils::wasm::JsResult;
    use utils_types::primitives::Address;

    macro_rules! make_js_on_chain_sender {
        ($on_chain_tx_sender:expr) => {
            |ack: core_types::acknowledgement::AcknowledgedTicket|
            -> std::pin::Pin<Box<dyn futures::Future<Output = Result<String, String>>>> {
            Box::pin(async move {
                let serialized_ack: core_types::acknowledgement::wasm::AcknowledgedTicket = ack.into();
                match $on_chain_tx_sender.call1(&wasm_bindgen::JsValue::null(), &wasm_bindgen::JsValue::from(serialized_ack)) {
                    Ok(res) => {
                        let promise = js_sys::Promise::from(res);
                        let ret = wasm_bindgen_futures::JsFuture::from(promise)
                            .await
                            .map(|v| v.as_string().expect("on-chain ticket redeem did not yield a string"))
                            .map_err(|v| utils_misc::utils::wasm::js_value_to_error_msg(v).unwrap_or("unknown error".to_string()))?;
                        Ok(ret)
                    }
                    Err(_) => Err("failed to call on-chain redeem TX closure".to_string())
                }
            })
        }
        };
    }

    #[wasm_bindgen]
    pub async fn redeem_all_tickets(
        db: &Database,
        self_addr: &Address,
        on_chain_tx_sender: &js_sys::Function,
    ) -> JsResult<()> {
        let js_on_chain_tx_sender = make_js_on_chain_sender!(on_chain_tx_sender);
        super::redeem_all_tickets(db.as_ref_counted().clone(), self_addr, &js_on_chain_tx_sender).await?;
        Ok(())
    }

    #[wasm_bindgen]
    pub async fn redeem_tickets_with_counterparty(db: &Database, counterparty: &Address, on_chain_tx_sender: &js_sys::Function) -> JsResult<()> {
        let js_on_chain_tx_sender = make_js_on_chain_sender!(on_chain_tx_sender);
        super::redeem_tickets_with_counterparty(db.as_ref_counted(), counterparty, &js_on_chain_tx_sender).await?;
        Ok(())
    }

    #[wasm_bindgen]
    pub async fn redeem_tickets_in_channel(
        db: &Database,
        channel: &ChannelEntry,
        on_chain_tx_sender: &js_sys::Function,
    ) -> JsResult<()> {
        let js_on_chain_tx_sender = make_js_on_chain_sender!(on_chain_tx_sender);
        super::redeem_tickets_in_channel(db.as_ref_counted().clone(), channel, &js_on_chain_tx_sender).await?;
        Ok(())
    }

    #[wasm_bindgen]
    pub async fn redeem_ticket(
        db: &Database,
        ack_ticket: &AcknowledgedTicket,
        on_chain_tx_sender: &js_sys::Function,
    ) -> JsResult<Hash> {
        let js_on_chain_tx_sender = make_js_on_chain_sender!(on_chain_tx_sender);
        Ok(super::redeem_ticket(db.as_ref_counted().clone(), ack_ticket.into(), &js_on_chain_tx_sender).await?)
    }
}
