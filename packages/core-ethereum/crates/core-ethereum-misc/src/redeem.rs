use crate::errors::CoreEthereumError::{
    InvalidArguments, NotAWinningTicket, TransactionSubmissionFailed, WrongTicketState,
};
use crate::errors::Result;
use async_lock::RwLock;
use core_crypto::types::Hash;
use core_ethereum_db::traits::HoprCoreEthereumDbActions;
use core_types::acknowledgement::AcknowledgedTicket;
use core_types::acknowledgement::AcknowledgedTicketStatus::{BeingAggregated, BeingRedeemed, Untouched};
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

/// Redeems all redeemable tickets in all channels.
pub async fn redeem_all_tickets<Db, F>(
    db: Arc<RwLock<Db>>,
    self_addr: &Address,
    onchain_tx_sender: &impl Fn(AcknowledgedTicket) -> F,
) -> Result<usize>
where
    Db: HoprCoreEthereumDbActions,
    F: futures::Future<Output = std::result::Result<String, String>>,
{
    let incoming_channels = db.read().await.get_channels_to(self_addr).await?;
    debug!(
        "starting to redeem all tickets in {} incoming channels to us.",
        incoming_channels.len()
    );

    let redeemed = futures::future::join_all(incoming_channels.iter().map(|channel| async {
        let channel_id = channel.get_id();
        let redeem_res = redeem_tickets_in_channel(db.clone(), channel, onchain_tx_sender).await;
        match redeem_res {
            Ok(count) => count,
            Err(e) => {
                error!("failed to redeem tickets in channel {channel_id}: {e}");
                0
            }
        }
    }))
    .await
    .into_iter()
    .sum();

    Ok(redeemed)
}

async fn set_being_redeemed<Db>(db: &mut Db, ack_ticket: &mut AcknowledgedTicket, tx_hash: Hash) -> Result<()>
where
    Db: HoprCoreEthereumDbActions,
{
    match ack_ticket.status {
        Untouched => {
            let dst = db
                .get_channels_domain_separator()
                .await
                .and_then(|separator| separator.ok_or(DbError::NotFound))?;

            // Check if we're going to redeem a winning ticket
            if !ack_ticket.is_winning_ticket(&dst) {
                return Err(NotAWinningTicket);
            }
        }
        BeingAggregated { .. } => return Err(WrongTicketState(ack_ticket.to_string())),
        BeingRedeemed { tx_hash: txh } => {
            // If there's already some hash set for this ticket, do not allow unsetting it
            if txh != Hash::default() && tx_hash == Hash::default() {
                return Err(InvalidArguments(format!("cannot unset tx hash of {ack_ticket}")));
            }
        }
    }

    ack_ticket.status = BeingRedeemed { tx_hash };
    debug!(
        "setting a winning {} as being redeemed with TX hash {tx_hash}",
        ack_ticket.ticket
    );
    Ok(db.update_acknowledged_ticket(ack_ticket).await?)
}

/// Redeems all redeemable tickets in the incoming channel from the given counterparty.
pub async fn redeem_tickets_with_counterparty<Db, F>(
    db: Arc<RwLock<Db>>,
    counterparty: &Address,
    onchain_tx_sender: &impl Fn(AcknowledgedTicket) -> F,
) -> Result<usize>
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

/// Redeems all redeemable tickets in the given channel.
pub async fn redeem_tickets_in_channel<Db, F>(
    db: Arc<RwLock<Db>>,
    channel: &ChannelEntry,
    onchain_tx_sender: &impl Fn(AcknowledgedTicket) -> F,
) -> Result<usize>
where
    Db: HoprCoreEthereumDbActions,
    F: futures::Future<Output = std::result::Result<String, String>>,
{
    let channel_id = channel.get_id();

    let count_redeemable_tickets = db
        .read()
        .await
        .get_acknowledged_tickets(Some(*channel))
        .await?
        .iter()
        .filter(|t| t.status == Untouched)
        .count();
    info!("there are {count_redeemable_tickets} acknowledged tickets in channel {channel_id} which can be redeemed");

    // Return fast if there are no redeemable tickets
    if count_redeemable_tickets == 0 {
        return Ok(0);
    }

    // Keep holding the DB write lock until we mark all the eligible tickets as BeginRedeemed
    let mut to_redeem = Vec::new();
    {
        // Lock the database and retrieve again all the redeemable tickets
        let mut db = db.write().await;
        let redeemable = db
            .get_acknowledged_tickets(Some(*channel))
            .await?
            .into_iter()
            .filter(|t| Untouched == t.status);

        for mut avail_to_redeem in redeemable {
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

    let redeemed = futures::future::join_all(to_redeem.into_iter().map(|ack_ticket| async {
        let ticket_id = ack_ticket.to_string();
        if let Err(e) = unchecked_ticket_redeem(db.clone(), ack_ticket, onchain_tx_sender).await {
            error!("failed to redeem {ticket_id}: {e}");
            false
        } else {
            true
        }
    }))
    .await
    .into_iter()
    .filter(|r| *r)
    .count();

    info!("{redeemed} tickets have been redeemed in channel {channel_id}");
    Ok(redeemed)
}

async fn unchecked_ticket_redeem<Db, F>(
    db: Arc<RwLock<Db>>,
    ack_ticket: AcknowledgedTicket,
    on_chain_tx_sender: impl Fn(AcknowledgedTicket) -> F,
) -> Result<Hash>
where
    Db: HoprCoreEthereumDbActions,
    F: futures::Future<Output = std::result::Result<String, String>>,
{
    debug!("sending {} for on-chain redemption", ack_ticket.ticket);
    match (on_chain_tx_sender)(ack_ticket.clone()).await {
        Ok(tx_hash_str) => {
            let tx_hash = Hash::from_hex(&tx_hash_str)?;
            debug!("{ack_ticket} tx {tx_hash} completed");

            db.write().await.mark_redeemed(&ack_ticket).await?;
            info!("redemption of {ack_ticket} completed successfully in {tx_hash} !");
            Ok(tx_hash)
        }
        Err(err) => Err(TransactionSubmissionFailed(err)),
    }
}

/// Tries to redeem the given ticket. If the ticket is not redeemable, returns an error.
/// Otherwise, the transaction hash of the on-chain redemption is returned.
pub async fn redeem_ticket<Db, F>(
    db: Arc<RwLock<Db>>,
    ack_ticket: AcknowledgedTicket,
    on_chain_tx_sender: impl Fn(AcknowledgedTicket) -> F,
) -> Result<Hash>
where
    Db: HoprCoreEthereumDbActions,
    F: futures::Future<Output = std::result::Result<String, String>>,
{
    if let Untouched = ack_ticket.status {
        unchecked_ticket_redeem(db, ack_ticket, on_chain_tx_sender).await
    } else {
        Err(WrongTicketState(ack_ticket.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use crate::redeem::{
        redeem_all_tickets, redeem_ticket, redeem_tickets_in_channel, redeem_tickets_with_counterparty,
    };
    use async_lock::{Mutex, RwLock};
    use core_crypto::keypairs::{ChainKeypair, Keypair};
    use core_crypto::random::random_bytes;
    use core_crypto::types::{Challenge, CurvePoint, HalfKey, Hash};
    use core_ethereum_db::db::CoreEthereumDb;
    use core_ethereum_db::traits::HoprCoreEthereumDbActions;
    use core_types::acknowledgement::AcknowledgedTicketStatus::{BeingAggregated, BeingRedeemed, Untouched};
    use core_types::acknowledgement::{AcknowledgedTicket, UnacknowledgedTicket};
    use core_types::channels::{ChannelEntry, ChannelStatus, Ticket};
    use hex_literal::hex;
    use std::ops::Deref;
    use std::sync::Arc;
    use utils_db::constants::ACKNOWLEDGED_TICKETS_PREFIX;
    use utils_db::db::DB;
    use utils_db::rusty::RustyLevelDbShim;
    use utils_types::primitives::{Balance, BalanceType, Snapshot, U256};
    use utils_types::traits::{BinarySerializable, ToHex};

    lazy_static::lazy_static! {
        static ref ALICE: ChainKeypair = ChainKeypair::from_secret(&hex!("492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775")).unwrap();
        static ref BOB: ChainKeypair = ChainKeypair::from_secret(&hex!("48680484c6fc31bc881a0083e6e32b6dc789f9eaba0f8b981429fd346c697f8c")).unwrap();
        static ref CHARLIE: ChainKeypair = ChainKeypair::from_secret(&hex!("d39a926980d6fa96a9eba8f8058b2beb774bc11866a386e9ddf9dc1152557c26")).unwrap();
    }

    fn generate_random_ack_ticket(idx: u32, counterparty: &ChainKeypair) -> AcknowledgedTicket {
        let hk1 = HalfKey::random();
        let hk2 = HalfKey::random();

        let cp1: CurvePoint = hk1.to_challenge().into();
        let cp2: CurvePoint = hk2.to_challenge().into();
        let cp_sum = CurvePoint::combine(&[&cp1, &cp2]);

        let price_per_packet: U256 = 10000000000000000u128.into(); // 0.01 HOPR

        let ticket = Ticket::new(
            &ALICE.public().to_address(),
            &Balance::new(
                price_per_packet.divide_f64(1.0f64).unwrap() * 5u64.into(),
                BalanceType::HOPR,
            ),
            idx.into(),
            U256::one(),
            1.0f64,
            4u64.into(),
            Challenge::from(cp_sum).to_ethereum_challenge(),
            counterparty,
            &Hash::default(),
        )
        .unwrap();

        let unacked_ticket = UnacknowledgedTicket::new(ticket, hk1, counterparty.public().to_address());
        unacked_ticket.acknowledge(&hk2, &ALICE, &Hash::default()).unwrap()
    }

    fn to_acknowledged_ticket_key(ack: &AcknowledgedTicket) -> utils_db::db::Key {
        let mut ack_key = Vec::new();

        ack_key.extend_from_slice(&ack.ticket.channel_id.to_bytes());
        ack_key.extend_from_slice(&ack.ticket.channel_epoch.to_be_bytes());
        ack_key.extend_from_slice(&ack.ticket.index.to_be_bytes());

        utils_db::db::Key::new_bytes_with_prefix(&ack_key, ACKNOWLEDGED_TICKETS_PREFIX).unwrap()
    }

    async fn create_channel_with_ack_tickets(
        rdb: RustyLevelDbShim,
        ticket_count: usize,
        counterparty: &ChainKeypair,
    ) -> (ChannelEntry, Vec<AcknowledgedTicket>) {
        let mut inner_db = DB::new(rdb);
        let mut input_tickets = Vec::new();

        for i in 0..ticket_count {
            let ack_ticket = generate_random_ack_ticket(i as u32, counterparty);
            inner_db
                .set(to_acknowledged_ticket_key(&ack_ticket), &ack_ticket)
                .await
                .unwrap();
            input_tickets.push(ack_ticket);
        }

        let mut db = CoreEthereumDb::new(inner_db, ALICE.public().to_address());
        let channel = ChannelEntry::new(
            counterparty.public().to_address(),
            ALICE.public().to_address(),
            Balance::zero(BalanceType::HOPR),
            U256::zero(),
            ChannelStatus::Open,
            U256::zero(),
            U256::zero(),
        );
        db.update_channel_and_snapshot(&channel.get_id(), &channel, &Default::default())
            .await
            .unwrap();
        db.set_channels_domain_separator(&Hash::default(), &Snapshot::default())
            .await
            .unwrap();

        (channel, input_tickets)
    }

    #[async_std::test]
    async fn test_ticket_redeem_flow() {
        let _ = env_logger::builder().is_test(true).try_init();

        let ticket_count = 5;
        let rdb = RustyLevelDbShim::new_in_memory();

        let (channel_from_bob, bob_tickets) = create_channel_with_ack_tickets(rdb.clone(), ticket_count, &BOB).await;
        let (channel_from_charlie, charlie_tickets) =
            create_channel_with_ack_tickets(rdb.clone(), ticket_count, &CHARLIE).await;

        let db = Arc::new(RwLock::new(CoreEthereumDb::new(
            DB::new(rdb.clone()),
            ALICE.public().to_address(),
        )));

        let dummy_tx_hash = Hash::new(&random_bytes::<{ Hash::SIZE }>());
        let redeemed_tickets = Arc::new(Mutex::new(Vec::new()));
        let rt_clone = redeemed_tickets.clone();

        let actually_redeemed = redeem_all_tickets(
            db.clone(),
            &ALICE.public().to_address(),
            &|ack: AcknowledgedTicket| async {
                rt_clone.lock().await.push(ack);
                Ok(dummy_tx_hash.to_hex())
            },
        )
        .await
        .unwrap();

        let db_acks_bob = db
            .read()
            .await
            .get_acknowledged_tickets(Some(channel_from_bob))
            .await
            .unwrap();
        let db_acks_charlie = db
            .read()
            .await
            .get_acknowledged_tickets(Some(channel_from_charlie))
            .await
            .unwrap();

        assert_eq!(
            0,
            db_acks_bob.len(),
            "no unredeemed tickets should be remaining for Bob"
        );

        assert_eq!(
            0,
            db_acks_charlie.len(),
            "no unredeemed tickets should be remaining for Charlie"
        );

        assert_eq!(
            2 * ticket_count,
            actually_redeemed,
            "all tickets must be reported as redeemed"
        );

        assert_eq!(
            2 * ticket_count,
            db.read().await.get_redeemed_tickets_count().await.unwrap(),
            "all tickets have to be redeemed"
        );

        assert_eq!(
            2 * ticket_count,
            redeemed_tickets.lock().await.len(),
            "all tickets must make it to on chain redemption"
        );

        assert_eq!(
            bob_tickets.iter().map(|t| t.ticket.clone()).collect::<Vec<_>>(),
            redeemed_tickets
                .lock()
                .await
                .deref()
                .iter()
                .filter(|t| t.signer == BOB.public().to_address())
                .map(|t| t.ticket.clone())
                .collect::<Vec<_>>(),
            "all redeemed tickets from Bob must make it to on-chain redemption"
        );

        assert_eq!(
            charlie_tickets.iter().map(|t| t.ticket.clone()).collect::<Vec<_>>(),
            redeemed_tickets
                .lock()
                .await
                .deref()
                .iter()
                .filter(|t| t.signer == CHARLIE.public().to_address())
                .map(|t| t.ticket.clone())
                .collect::<Vec<_>>(),
            "all redeemed tickets from Charlie must make it to on-chain redemption"
        );
    }

    #[async_std::test]
    async fn test_ticket_redeem_in_channel() {
        let _ = env_logger::builder().is_test(true).try_init();

        let ticket_count = 5;
        let rdb = RustyLevelDbShim::new_in_memory();

        let (channel_from_bob, bob_tickets) = create_channel_with_ack_tickets(rdb.clone(), ticket_count, &BOB).await;
        let (channel_from_charlie, _) = create_channel_with_ack_tickets(rdb.clone(), ticket_count, &CHARLIE).await;

        let db = Arc::new(RwLock::new(CoreEthereumDb::new(
            DB::new(rdb.clone()),
            ALICE.public().to_address(),
        )));

        let dummy_tx_hash = Hash::new(&random_bytes::<{ Hash::SIZE }>());
        let redeemed_tickets = Arc::new(Mutex::new(Vec::new()));
        let rt_clone = redeemed_tickets.clone();

        let actually_redeemed = redeem_tickets_with_counterparty(
            db.clone(),
            &BOB.public().to_address(),
            &|ack: AcknowledgedTicket| async {
                rt_clone.lock().await.push(ack);
                Ok(dummy_tx_hash.to_hex())
            },
        )
        .await
        .unwrap();

        let db_acks_bob = db
            .read()
            .await
            .get_acknowledged_tickets(Some(channel_from_bob))
            .await
            .unwrap();
        let db_acks_charlie = db
            .read()
            .await
            .get_acknowledged_tickets(Some(channel_from_charlie))
            .await
            .unwrap();

        assert_eq!(
            ticket_count, actually_redeemed,
            "{ticket_count} must be reported as be redeemed"
        );

        assert_eq!(
            ticket_count,
            db_acks_charlie.len(),
            "charlie must still have {ticket_count} unredeemed"
        );

        assert_eq!(
            0,
            db_acks_bob.len(),
            "bob must have all {ticket_count} tickets redeemed"
        );

        assert!(
            db_acks_charlie.iter().all(|t| match t.status {
                Untouched => true,
                _ => false,
            }),
            "all tickets from Charlie must be Untouched"
        );

        assert_eq!(
            ticket_count,
            redeemed_tickets.lock().await.len(),
            "exactly {ticket_count} tickets must make it to on-chain redemption"
        );

        assert_eq!(
            bob_tickets.iter().map(|t| t.ticket.clone()).collect::<Vec<_>>(),
            redeemed_tickets
                .lock()
                .await
                .deref()
                .iter()
                .filter(|t| t.signer == BOB.public().to_address())
                .map(|t| t.ticket.clone())
                .collect::<Vec<_>>(),
            "all redeemed tickets from Bob must make it to on-chain redemption"
        );
    }

    #[async_std::test]
    async fn test_redeem_must_not_work_for_tickets_being_aggregated_and_being_redeemed() {
        let _ = env_logger::builder().is_test(true).try_init();

        let ticket_count = 3;
        let rdb = RustyLevelDbShim::new_in_memory();

        let (channel_from_bob, mut tickets) = create_channel_with_ack_tickets(rdb.clone(), ticket_count, &BOB).await;

        let db = Arc::new(RwLock::new(CoreEthereumDb::new(
            DB::new(rdb.clone()),
            ALICE.public().to_address(),
        )));

        // Make the first ticket unredeemable
        tickets[0].status = BeingAggregated { start: 0, end: 1 };
        db.write().await.update_acknowledged_ticket(&tickets[0]).await.unwrap();

        // Make the second ticket unredeemable
        tickets[1].status = BeingRedeemed {
            tx_hash: Hash::new(&random_bytes::<{ Hash::SIZE }>()),
        };
        db.write().await.update_acknowledged_ticket(&tickets[1]).await.unwrap();

        let dummy_tx_hash = Hash::new(&random_bytes::<{ Hash::SIZE }>());

        let redeemed_tickets = Arc::new(Mutex::new(Vec::new()));
        let rt_clone = redeemed_tickets.clone();

        let actually_redeemed =
            redeem_tickets_in_channel(db.clone(), &channel_from_bob, &|ack: AcknowledgedTicket| async {
                rt_clone.lock().await.push(ack);
                Ok(dummy_tx_hash.to_hex())
            })
            .await
            .unwrap();

        assert_eq!(
            ticket_count - 2,
            actually_redeemed,
            "remaining redeemable tickets must be reported as redeemed"
        );

        assert_eq!(
            ticket_count - 2,
            redeemed_tickets.lock().await.len(),
            "tickets being redeemed and aggregated must be skipped during redemption"
        );

        assert_eq!(
            tickets[2..].iter().map(|t| t.ticket.clone()).collect::<Vec<_>>(),
            redeemed_tickets
                .lock()
                .await
                .iter()
                .map(|t| t.ticket.clone())
                .collect::<Vec<_>>(),
            "only redeemable tickets must be redeemed"
        );

        assert!(
            redeem_ticket(db.clone(), tickets[0].clone(), &|_| async {
                Ok(dummy_tx_hash.to_hex())
            })
            .await
            .is_err(),
            "cannot redeem a ticket that's being aggregated"
        );

        assert!(
            redeem_ticket(db.clone(), tickets[1].clone(), &|_| async {
                Ok(dummy_tx_hash.to_hex())
            })
            .await
            .is_err(),
            "cannot redeem a ticket that's being redeemed"
        );
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
    use wasm_bindgen::prelude::wasm_bindgen;

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
    pub async fn redeem_tickets_with_counterparty(
        db: &Database,
        counterparty: &Address,
        on_chain_tx_sender: &js_sys::Function,
    ) -> JsResult<()> {
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
