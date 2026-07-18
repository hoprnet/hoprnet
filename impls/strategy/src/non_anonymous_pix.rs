//! ## Non-Anonymous PIX Strategy
//!
//! This strategy is responsible for handling non-anonymous PIX transactions.
//!
//! It is responsible for:
//! - Handling new deposit addresses
//! - Handling deposit address recovery
//! - Handling PIX transfers
//!
//! All of these are done in a **non-anonymous** way, using plain on-chain transactions.
//!
//! **DO NOT USE THIS STRATEGY IN PRODUCTION**

use std::{
    collections::HashSet,
    convert::identity,
    fmt::{Debug, Display, Formatter},
    sync::Arc,
    time::Duration,
};

use futures::{FutureExt, SinkExt, StreamExt, TryFutureExt, TryStreamExt};
use futures_time::future::FutureExt as TimeExt;
use hopr_api::{
    ChainKeypair,
    chain::{ChainValues, ChainWriteAccountOperations},
    node::{ActionableEventDiscriminant, ActionableEventSource, HasChainApi, PixEvent},
    types::{crypto::prelude::Keypair, primitive::prelude::*},
};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::{errors::StrategyError, strategy::Strategy as StrategyTrait};

#[derive(Clone, Debug, Serialize, Deserialize, Validate)]
pub struct NonAnonymousPixStrategyConfig {
    pub price_per_byte: HoprBalance,
    pub max_ssa_allocation: HoprBalance,
    pub max_deposit_tracking_time: Duration,
    /// If set, the strategy persists recovered private keys to `redb` at this
    /// path before withdrawing (Exit role).  `None` means in-memory only (Entry role).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pix_recovery_db_path: Option<std::path::PathBuf>,
}

/// Builder for [`NonAnonymousPixStrategy`].
///
/// Call [`new`](NonAnonymousPixStrategy::new) with the strategy configuration,
/// then [`build`](NonAnonymousPixStrategy::build) to wire in a node and obtain a
/// runnable `Box<dyn StrategyTrait + Send>`.
pub struct NonAnonymousPixStrategy {
    cfg: NonAnonymousPixStrategyConfig,
    interval: Duration,
}

impl NonAnonymousPixStrategy {
    /// Create a new builder with the given configuration.
    pub fn new(cfg: NonAnonymousPixStrategyConfig, interval: Duration) -> Self {
        Self { cfg, interval }
    }

    /// Wire in a node and return a running-ready strategy.
    pub fn build<N>(self, node: Arc<N>) -> Box<dyn StrategyTrait + Send>
    where
        N: HasChainApi + ActionableEventSource + Send + Sync + 'static,
    {
        let recovery_store = self
            .cfg
            .pix_recovery_db_path
            .as_ref()
            .map(|path| crate::pix_recovery_store::PixRecoveryStore::open(path).expect("PixRecoveryStore::open"));

        Box::new(NonAnonymousPixStrategyInner {
            cfg: self.cfg,
            interval: self.interval,
            node,
            recovery_store,
            processed_deposits: HashSet::new(),
        })
    }
}

/// Private generic runner — constructed by [`NonAnonymousPixStrategy::build`].
struct NonAnonymousPixStrategyInner<N: HasChainApi> {
    node: Arc<N>,
    cfg: NonAnonymousPixStrategyConfig,
    interval: Duration,
    /// Exit role only: persisted recovery store for [`PixEvent::PrivateKeyRecovered`].
    /// Entry role leaves this as `None`.
    recovery_store: Option<crate::pix_recovery_store::PixRecoveryStore>,
    /// Entry role only: IDs of deposit addresses already funded.
    processed_deposits: HashSet<hopr_api::node::PixAddressId>,
}

impl<N> NonAnonymousPixStrategyInner<N>
where
    N: HasChainApi + ActionableEventSource + Send + Sync + 'static,
{
    /// Periodic task logic.
    async fn on_tick(&self) -> crate::errors::Result<()> {
        tracing::debug!("PixStrategy tick");
        Ok(())
    }

    /// Handle PIX event.
    async fn on_pix_event(&mut self, event: PixEvent) -> crate::errors::Result<()> {
        tracing::debug!(?event, "PixStrategy event");
        match event {
            PixEvent::NewDepositAddress(new_deposit_address) => {
                tracing::info!(?new_deposit_address, "new deposit address");

                // Entry-side dedup: skip duplicates within the same strategy lifetime.
                if !self.processed_deposits.insert(new_deposit_address.id) {
                    tracing::warn!(id = ?new_deposit_address.id, "duplicate NewDepositAddress event, skipping");
                    return Ok(());
                }

                let target_deposit = self.cfg.price_per_byte * new_deposit_address.quota;
                if target_deposit > self.cfg.max_ssa_allocation {
                    tracing::warn!(%target_deposit, max_deposit = %self.cfg.max_ssa_allocation, "target deposit too high");
                    return Err(StrategyError::CriteriaNotSatisfied);
                }

                // TODO: do not allow parallel withdrawals to any address
                if let Err(error) = self
                    .node
                    .chain_api()
                    .withdraw(target_deposit, &new_deposit_address.address.try_into()?)
                    .and_then(identity)
                    .await
                {
                    tracing::error!(%error, %target_deposit, ?new_deposit_address, "withdraw failed");
                    return Err(StrategyError::other(error));
                }
                tracing::info!(%target_deposit, ?new_deposit_address, "deposit successful");
            }
            PixEvent::DepositAddressReceived(deposit_address_recv) => {
                tracing::info!(?deposit_address_recv, "deposit address received");

                let target_deposit = self.cfg.price_per_byte * deposit_address_recv.quota;
                let pix_id = deposit_address_recv.id;
                let deposit_updated = deposit_address_recv.deposit_updated;
                let node_clone = self.node.clone();
                let node_clone_for_initial = self.node.clone();
                let deposit_addr: Address = deposit_address_recv.address.try_into()?;

                let max_tracking_time = self.cfg.max_deposit_tracking_time;

                let mut stream = futures_time::stream::interval(
                    futures_time::time::Duration::from(max_tracking_time / 10).max(Duration::from_secs(1).into()),
                )
                .then(move |_| {
                    let node_clone = node_clone.clone();
                    async move {
                        node_clone
                            .chain_api()
                            .balance(deposit_addr)
                            .map_err(StrategyError::other)
                            .await
                    }
                })
                .try_skip_while(move |balance| futures::future::ok(balance < &target_deposit))
                .boxed();

                tracing::info!(%target_deposit, ?max_tracking_time, "tracking until deposit");
                hopr_utils::runtime::prelude::spawn(
                    async move {
                        // Check balance immediately (first poll) to avoid the sub-second
                        // first-poll delay inherent to stream::interval. Both the immediate
                        // check and the interval polling are inside the timeout guard so a
                        // stalled RPC does not block event handling indefinitely.
                        let immediate = node_clone_for_initial
                            .chain_api()
                            .balance(deposit_addr)
                            .await
                            .ok()
                            .filter(|b| *b >= target_deposit);

                        let deposit = match (immediate, stream.try_next().await) {
                            (Some(balance), _) => balance,
                            (None, Ok(Some(balance))) => balance,
                            _ => return Err(StrategyError::other(anyhow::anyhow!("deposit tracking not available"))),
                        };

                        if let Some(mut notifier) = deposit_updated {
                            notifier.send((pix_id, deposit)).await.map_err(StrategyError::other)
                        } else {
                            Ok(())
                        }
                    }
                    .timeout(futures_time::time::Duration::from(max_tracking_time))
                    .inspect(|res| match res {
                        Ok(Ok(_)) => tracing::info!("deposit tracking completed"),
                        Ok(Err(error)) => tracing::error!(%error, "deposit tracking failed:"),
                        Err(_) => tracing::error!("deposit tracking timed out"),
                    }),
                );
            }
            PixEvent::PrivateKeyRecovered(private_key_recovered) => {
                tracing::info!(?private_key_recovered, "private key recovered");

                // Exit-side persistence: write to redb before withdrawing so the key
                // survives crashes and can be replayed on restart.
                if let Some(ref store) = self.recovery_store {
                    if let Err(error) = store.insert(&private_key_recovered.id, &private_key_recovered.secret) {
                        tracing::error!(%error, ?private_key_recovered.id, "failed to persist recovered key");
                        return Err(StrategyError::other(error));
                    }
                }

                let chain_key =
                    ChainKeypair::from_secret(private_key_recovered.secret.0.as_ref()).map_err(StrategyError::other)?;

                let safe_address = self.node.identity().safe_address;

                let recovered_balance: HoprBalance = self
                    .node
                    .chain_api()
                    .balance(chain_key.public().to_address())
                    .await
                    .map_err(StrategyError::other)?;
                tracing::info!(%recovered_balance, address = %chain_key.public().to_address(), "recovered deposit balance");

                self.node
                    .chain_api()
                    .withdraw_from_signer(&chain_key, recovered_balance, &safe_address)
                    .await
                    .map_err(StrategyError::other)?
                    .await
                    .map_err(StrategyError::other)?;

                tracing::info!(%recovered_balance, address = %chain_key.public().to_address(),  "deposit withdrawn");
            }
        }

        Ok(())
    }

    /// Replay recovery entries whose on-chain balance is still non-zero (crash recovery).
    async fn replay_pending_recoveries(&self, store: &crate::pix_recovery_store::PixRecoveryStore) {
        let entries = match store.iter() {
            Ok(entries) => entries,
            Err(error) => {
                tracing::error!(%error, "failed to iterate recovery store on startup");
                return;
            }
        };

        if entries.is_empty() {
            return;
        }

        tracing::info!(count = entries.len(), "replaying pending private key recoveries");

        for (id, secret) in entries {
            match ChainKeypair::from_secret(secret.0.as_ref()) {
                Ok(chain_key) => {
                    let balance: HoprBalance =
                        match self.node.chain_api().balance(chain_key.public().to_address()).await {
                            Ok(b) => b,
                            Err(e) => {
                                tracing::error!(%e, ?id, "failed to query balance during recovery replay");
                                continue;
                            }
                        };

                    if balance.is_zero() {
                        continue;
                    }

                    tracing::info!(%balance, ?id, "replaying withdrawal for pending recovery");

                    let withdraw_fut = match self
                        .node
                        .chain_api()
                        .withdraw_from_signer(&chain_key, balance, &self.node.identity().safe_address)
                        .await
                    {
                        Ok(fut) => fut,
                        Err(error) => {
                            tracing::error!(%error, ?id, "recovery replay: withdraw_from_signer call failed");
                            continue;
                        }
                    };
                    if let Err(error) = withdraw_fut.await {
                        tracing::error!(%error, ?id, "recovery replay withdrawal failed");
                    }
                }
                Err(error) => {
                    tracing::error!(%error, ?id, "failed to reconstruct chain key during recovery replay");
                }
            }
        }
    }
}

impl<N: HasChainApi> Debug for NonAnonymousPixStrategyInner<N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "NonAnonymousPixStrategy({:?})", self.cfg)
    }
}

impl<N: HasChainApi> Display for NonAnonymousPixStrategyInner<N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "non_anonymous_pix")
    }
}

#[async_trait::async_trait]
impl<N: HasChainApi> StrategyTrait for NonAnonymousPixStrategyInner<N>
where
    N: HasChainApi + ActionableEventSource + Send + Sync + 'static,
{
    async fn run(&mut self) -> crate::errors::Result<()> {
        enum Event {
            Tick,
            Pix(PixEvent),
        }

        // Run the first scan immediately at startup without waiting for the initial interval.
        if let Err(error) = self.on_tick().await
            && !matches!(error, StrategyError::CriteriaNotSatisfied)
        {
            tracing::error!(%error, "pix tick failed");
        }

        // Startup recovery: replay persisted keys whose on-chain balance is still non-zero.
        if let Some(ref store) = self.recovery_store {
            self.replay_pending_recoveries(store).await;
        }

        let tick_stream = futures_time::stream::interval(self.interval.into()).map(|_| Event::Tick);
        let event_stream = self
            .node
            .subscribe_to_actionable_events(Some(&[ActionableEventDiscriminant::Pix]))
            .map_err(|e| StrategyError::Other(anyhow::anyhow!(e)))?
            .filter_map(|event| futures::future::ready(event.try_as_pix().map(Event::Pix)));

        let mut combined = futures_concurrency::stream::Merge::merge((tick_stream, event_stream));

        while let Some(event) = combined.next().await {
            match event {
                Event::Tick => {
                    if let Err(error) = self.on_tick().await
                        && !matches!(error, StrategyError::CriteriaNotSatisfied)
                    {
                        tracing::error!(%error, "pix tick failed");
                    }
                }
                Event::Pix(event) => {
                    if let Err(error) = self.on_pix_event(event).await {
                        tracing::error!(%error, "pix event failed");
                    }
                }
            }
        }

        Ok(())
    }
}

/// Test-only helpers for driving `NonAnonymousPixStrategyInner` from unit tests.
#[cfg(test)]
impl<N> NonAnonymousPixStrategyInner<N>
where
    N: HasChainApi + ActionableEventSource + Send + Sync + 'static,
{
    /// Read the HOPR balance of the given address via the node's chain API.
    async fn get_balance(&self, address: Address) -> crate::errors::Result<HoprBalance> {
        self.node
            .chain_api()
            .balance(address)
            .await
            .map_err(StrategyError::other)
    }
}

#[cfg(test)]
mod tests {
    use std::{num::NonZeroU32, time::Duration as StdDuration};

    use anyhow::Context;
    use futures::{StreamExt, channel::mpsc};
    use hex_literal::hex;
    use hopr_api::{
        chain::{
            AccountSelector, ChainEvents, ChainReadAccountOperations, ChainReadChannelOperations,
            ChainWriteAccountOperations, HoprChainApi,
        },
        node::{
            ActionableEvent, ActionableEventDiscriminant, ComponentStatus, ComponentStatusReporter, EventWaitResult,
            HasChainApi, NodeOnchainIdentity, PixDepositAddressReceived, PixEvent,
        },
        types::{
            crypto::{keypairs::Keypair, prelude::ChainKeypair},
            crypto_random::Randomizable,
            internal::prelude::HoprPseudonym,
            primitive::prelude::{Address, HoprBalance, XDaiBalance},
        },
    };
    use hopr_chain_connector::{create_trustful_hopr_blokli_connector, testing::BlokliTestStateBuilder};
    use tokio::time::timeout;

    use super::*;

    lazy_static::lazy_static! {
        static ref BOB_KP: ChainKeypair = ChainKeypair::from_secret(&hex!(
            "492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775"
        ))
        .expect("lazy static keypair should be valid");

        static ref ALICE: Address = hex!("18f8ae833c85c51fbeba29cef9fbfb53b3bad950").into();
        static ref BOB: Address = BOB_KP.public().to_address();
        static ref CHRIS: Address = hex!("b6021e0860dd9d96c9ff0a73e2e5ba3a466ba234").into();
    }

    /// Minimal node wrapper used in strategy tests.
    struct ChainNode<C>(C);

    impl<C> HasChainApi for ChainNode<C>
    where
        C: HoprChainApi + ChainReadChannelOperations + ComponentStatusReporter + Clone + Send + Sync + 'static,
    {
        type ChainApi = C;
        type ChainError = <C as HoprChainApi>::ChainError;

        fn identity(&self) -> &NodeOnchainIdentity {
            static IDENTITY: std::sync::OnceLock<NodeOnchainIdentity> = std::sync::OnceLock::new();
            IDENTITY.get_or_init(|| {
                let me = *self.0.me();
                NodeOnchainIdentity {
                    node_address: me,
                    safe_address: me,
                    module_address: [1u8; Address::SIZE].into(),
                }
            })
        }

        fn chain_api(&self) -> &C {
            &self.0
        }

        fn status(&self) -> ComponentStatus {
            self.0.component_status()
        }

        fn wait_for_on_chain_event<F>(
            &self,
            _predicate: F,
            _context: String,
            _timeout: std::time::Duration,
        ) -> EventWaitResult<<C as HoprChainApi>::ChainError, <C as HoprChainApi>::ChainError>
        where
            F: Fn(&hopr_api::chain::ChainEvent) -> bool + Send + Sync + 'static,
        {
            unimplemented!("tests do not call wait_for_on_chain_event")
        }
    }

    impl<C> ActionableEventSource for ChainNode<C>
    where
        C: ChainEvents + Send + Sync + 'static,
    {
        fn subscribe_to_actionable_events(
            &self,
            _filter: Option<&[ActionableEventDiscriminant]>,
        ) -> Result<futures::stream::BoxStream<'static, ActionableEvent>, String> {
            Ok(self
                .0
                .subscribe()
                .map_err(|e| e.to_string())?
                .map(ActionableEvent::Chain)
                .boxed())
        }
    }

    async fn register_test_safe<C>(chain_connector: &C, node_address: Address) -> anyhow::Result<()>
    where
        C: HoprChainApi + ChainReadAccountOperations + ChainWriteAccountOperations,
    {
        let account = chain_connector
            .stream_accounts(AccountSelector::default().with_chain_key(node_address))?
            .next()
            .await
            .context("missing test account for node")?;
        let safe_address = account.safe_address.context("missing test safe address for node")?;

        chain_connector.register_safe(&safe_address).await?.await?;

        Ok(())
    }

    /// PixEvent::DepositAddressReceived spawns a background task that polls the deposit
    /// address balance until it reaches the target (`price_per_byte * quota`), then sends the
    /// received amount through the `deposit_updated` notifier channel.
    ///
    /// The handler returns immediately after spawning — the polling runs asynchronously.
    /// This test pre-sets the deposit address balance to the target so the first poll
    /// (every max_tracking_time/10, capped at 1 s) immediately detects it.
    ///
    /// Verifies the notifier receives `(PixAddressId, HoprBalance)` with the correct amount.
    #[test_log::test(tokio::test)]
    async fn test_deposit_address_received_notifies_on_balance_arrival() -> anyhow::Result<()> {
        let price_per_byte = HoprBalance::new_base(1);
        let max_ssa_allocation = HoprBalance::new_base(100);
        let quota = 100_u64;
        let target_deposit = price_per_byte * quota; // 100 wxHOPR

        let deposit_addr: Address = [0x99u8; 20].into();

        let (tx, mut rx) = mpsc::channel::<(hopr_api::node::PixAddressId, HoprBalance)>(1);

        let blokli_sim = BlokliTestStateBuilder::default()
            .with_generated_accounts(
                &[&*ALICE, &*BOB, &*CHRIS],
                false,
                XDaiBalance::new_base(1),
                HoprBalance::new_base(1000),
            )
            // Pre-set the deposit address balance to the target so the first poll succeeds.
            .with_balances([(deposit_addr, target_deposit)])
            .build_dynamic_client([1; Address::SIZE].into());

        let mut chain_connector =
            create_trustful_hopr_blokli_connector(&BOB_KP, Default::default(), blokli_sim, [1; Address::SIZE].into())
                .await?;
        chain_connector.connect().await?;
        let chain_connector = Arc::new(chain_connector);

        let cfg = NonAnonymousPixStrategyConfig {
            price_per_byte,
            max_ssa_allocation,
            max_deposit_tracking_time: StdDuration::from_secs(5),
            pix_recovery_db_path: None,
        };

        let mut strategy = NonAnonymousPixStrategyInner {
            cfg,
            interval: StdDuration::from_secs(60),
            node: Arc::new(ChainNode(Arc::clone(&chain_connector))),
            recovery_store: None,
            processed_deposits: HashSet::new(),
        };

        let event = PixEvent::DepositAddressReceived(PixDepositAddressReceived {
            id: (HoprPseudonym::random(), NonZeroU32::new(1).unwrap()),
            address: deposit_addr.into(),
            quota,
            deposit_updated: Some(tx),
        });

        // Spawn the handler (returns immediately, polling runs in background).
        strategy.on_pix_event(event).await?;

        // Wait for the notifier to receive the deposit. The first poll fires after
        // max_tracking_time / 10 (capped at 1 s). Allow up to 10 s for the notification.
        let notified = timeout(StdDuration::from_secs(10), rx.next())
            .await
            .context("deposit notification timed out")?
            .context("notifier dropped before sending deposit")?;

        let (_pix_id, notified_balance) = notified;
        assert_eq!(
            notified_balance, target_deposit,
            "notifier should receive the target deposit amount"
        );

        Ok(())
    }

    /// Step 1/2 — PixEvent::NewDepositAddress handler calls `withdraw` to move funds
    /// from the node's own address into the newly-assigned deposit address.
    ///
    /// Verifies that the withdrawal amount equals `price_per_byte * quota` (20 wxHOPR),
    /// the sender's balance decreases by that amount, and the deposit address receives it.
    ///
    /// Step 2/2 — the blokli snapshot records the final state so regressions are caught.
    #[test_log::test(tokio::test)]
    async fn test_new_deposit_address_withdraws_to_deposit_address() -> anyhow::Result<()> {
        let price_per_byte = HoprBalance::new_base(1);
        let max_ssa_allocation = HoprBalance::new_base(100);
        let quota = 20_u64;

        let deposit_address: Address = [0x42u8; 20].into();

        let blokli_sim = BlokliTestStateBuilder::default()
            .with_generated_accounts(
                &[&*ALICE, &*BOB, &*CHRIS],
                false,
                XDaiBalance::new_base(1),
                HoprBalance::new_base(1000),
            )
            // with_generated_accounts sets balances for each account's derived safe address,
            // but the test queries balance of BOB's raw chain address directly.
            .with_balances([(*BOB, HoprBalance::new_base(1000))])
            .build_dynamic_client([1; Address::SIZE].into());

        let snapshot = blokli_sim.snapshot();

        let mut chain_connector =
            create_trustful_hopr_blokli_connector(&BOB_KP, Default::default(), blokli_sim, [1; Address::SIZE].into())
                .await?;
        chain_connector.connect().await?;
        let chain_connector = Arc::new(chain_connector);

        let cfg = NonAnonymousPixStrategyConfig {
            price_per_byte,
            max_ssa_allocation,
            max_deposit_tracking_time: Duration::from_secs(5),
            pix_recovery_db_path: None,
        };

        let mut strategy = NonAnonymousPixStrategyInner {
            cfg,
            interval: Duration::from_secs(60),
            node: Arc::new(ChainNode(Arc::clone(&chain_connector))),
            recovery_store: None,
            processed_deposits: HashSet::new(),
        };

        let bob_balance_before = strategy
            .get_balance(*BOB)
            .await
            .context("get bob balance before withdraw")?;

        let event = PixEvent::NewDepositAddress(hopr_api::node::PixNewDepositAddress {
            id: (HoprPseudonym::random(), NonZeroU32::new(1).unwrap()),
            address: deposit_address.into(),
            quota,
        });

        strategy.on_pix_event(event).await?;

        // The withdrawal amount is price_per_byte * quota = 1 * 20 = 20.
        let bob_balance_after = strategy
            .get_balance(*BOB)
            .await
            .context("get bob balance after withdraw")?;

        assert_eq!(
            bob_balance_after,
            bob_balance_before - HoprBalance::new_base(20),
            "bob's balance should decrease by the withdrawal amount"
        );

        let deposit_balance = strategy
            .get_balance(deposit_address)
            .await
            .context("get deposit address balance")?;
        assert_eq!(
            deposit_balance,
            HoprBalance::new_base(20),
            "deposit address should have received the withdrawal"
        );

        insta::assert_yaml_snapshot!(*snapshot.refresh());

        Ok(())
    }

    /// PixEvent::NewDepositAddress handler rejects the withdrawal when the computed
    /// target deposit (`price_per_byte * quota`) exceeds `max_ssa_allocation`.
    ///
    /// price_per_byte=10, quota=10 → target=100, but max_ssa_allocation=50,
    /// so the handler must return `CriteriaNotSatisfied` and not send any transaction.
    #[test_log::test(tokio::test)]
    async fn test_new_deposit_address_rejects_when_exceeds_max_ssa_allocation() -> anyhow::Result<()> {
        // price_per_byte=10, quota=10 -> target=100, but max_ssa_allocation=50
        let price_per_byte = HoprBalance::new_base(10);
        let max_ssa_allocation = HoprBalance::new_base(50);
        let quota = 10_u64;

        let blokli_sim = BlokliTestStateBuilder::default()
            .with_generated_accounts(
                &[&*ALICE, &*BOB, &*CHRIS],
                false,
                XDaiBalance::new_base(1),
                HoprBalance::new_base(1000),
            )
            .build_dynamic_client([1; Address::SIZE].into());

        let mut chain_connector =
            create_trustful_hopr_blokli_connector(&BOB_KP, Default::default(), blokli_sim, [1; Address::SIZE].into())
                .await?;
        chain_connector.connect().await?;

        let cfg = NonAnonymousPixStrategyConfig {
            price_per_byte,
            max_ssa_allocation,
            max_deposit_tracking_time: Duration::from_secs(5),
            pix_recovery_db_path: None,
        };

        let mut strategy = NonAnonymousPixStrategyInner {
            cfg,
            interval: Duration::from_secs(60),
            node: Arc::new(ChainNode(Arc::new(chain_connector))),
            recovery_store: None,
            processed_deposits: HashSet::new(),
        };

        let event = PixEvent::NewDepositAddress(hopr_api::node::PixNewDepositAddress {
            id: (HoprPseudonym::random(), NonZeroU32::new(1).unwrap()),
            address: Address::from([0x42u8; 20]).into(),
            quota,
        });

        let result = strategy.on_pix_event(event).await;
        assert!(
            matches!(result, Err(crate::errors::StrategyError::CriteriaNotSatisfied)),
            "withdrawal should be rejected when target deposit exceeds max_ssa_allocation"
        );

        Ok(())
    }

    /// PixEvent::PrivateKeyRecovered reads the balance of the recovered keypair's
    /// raw chain address, then calls `withdraw_from_signer` to sweep the full balance
    /// to the node's own safe address.
    ///
    /// Verifies the recovered address ends at 0 wxHOPR and the safe receives the
    /// full recovered balance (50 wxHOPR). The blokli snapshot records the final state.
    #[test_log::test(tokio::test)]
    async fn test_private_key_recovered_withdraws_to_safe() -> anyhow::Result<()> {
        // Construct a deterministic keypair to simulate a recovered private key.
        let recovered_kp = ChainKeypair::from_secret(&hex!(
            "d4945a08c3c2d47f89df2c6d3e5e7d8e9f0a1b2c3d4e5f6a7b8c9d0e1f2a3b4c"
        ))
        .expect("recovered keypair should be valid");
        let recovered_address = recovered_kp.public().to_address();

        let price_per_byte = HoprBalance::new_base(1);
        let max_ssa_allocation = HoprBalance::new_base(100);
        let recovered_initial_balance = HoprBalance::new_base(50);

        let blokli_sim = BlokliTestStateBuilder::default()
            .with_generated_accounts(
                &[&*ALICE, &*BOB, &*CHRIS, &recovered_address],
                false,
                XDaiBalance::new_base(1),
                recovered_initial_balance,
            )
            // with_generated_accounts sets balances for each account's derived safe address,
            // but the test queries balance of the recovered address directly.
            .with_balances([(recovered_address, recovered_initial_balance)])
            .build_dynamic_client([1; Address::SIZE].into());

        let snapshot = blokli_sim.snapshot();

        let mut chain_connector =
            create_trustful_hopr_blokli_connector(&BOB_KP, Default::default(), blokli_sim, [1; Address::SIZE].into())
                .await?;
        chain_connector.connect().await?;
        register_test_safe(&chain_connector, *BOB).await?;
        let chain_connector = Arc::new(chain_connector);

        let cfg = NonAnonymousPixStrategyConfig {
            price_per_byte,
            max_ssa_allocation,
            max_deposit_tracking_time: std::time::Duration::from_secs(5),
            pix_recovery_db_path: None,
        };

        let mut strategy = NonAnonymousPixStrategyInner {
            cfg,
            interval: Duration::from_secs(60),
            node: Arc::new(ChainNode(Arc::clone(&chain_connector))),
            recovery_store: None,
            processed_deposits: HashSet::new(),
        };

        let safe_address = strategy.node.identity().safe_address;

        let event = PixEvent::PrivateKeyRecovered(hopr_api::node::PixPrivateKeyRecovered {
            id: (HoprPseudonym::random(), NonZeroU32::new(1).unwrap()),
            secret: hopr_api::node::PixDepositSecret(recovered_kp.secret().clone()),
        });

        strategy.on_pix_event(event).await?;

        // Recovered keypair's balance should be zero after withdrawal.
        let recovered_balance = strategy
            .get_balance(recovered_address)
            .await
            .context("get recovered address balance after withdraw")?;
        assert_eq!(
            recovered_balance,
            HoprBalance::zero(),
            "recovered keypair's balance should be zero after withdrawal"
        );

        // Safe should have received the full recovered balance.
        let safe_balance = strategy
            .get_balance(safe_address)
            .await
            .context("get safe balance after withdraw")?;
        assert_eq!(
            safe_balance, recovered_initial_balance,
            "safe should have received the full recovered balance"
        );

        insta::assert_yaml_snapshot!(*snapshot.refresh());

        Ok(())
    }

    #[test]
    fn test_config_default_passes_validation() {
        let cfg = NonAnonymousPixStrategyConfig {
            price_per_byte: HoprBalance::new_base(1),
            max_ssa_allocation: HoprBalance::new_base(100),
            max_deposit_tracking_time: std::time::Duration::from_secs(60),
            pix_recovery_db_path: None,
        };
        assert!(cfg.validate().is_ok(), "default config should pass validation");
    }

    /// Tests the public builder API: `NonAnonymousPixStrategy::new(...).build(node)` must
    /// return a `Box<dyn Strategy + Send>` with the expected Display string.
    #[tokio::test]
    async fn test_build_returns_strategy_trait_object() -> anyhow::Result<()> {
        let blokli_sim = BlokliTestStateBuilder::default()
            .with_generated_accounts(
                &[&*ALICE, &*BOB],
                false,
                XDaiBalance::new_base(1),
                HoprBalance::new_base(1000),
            )
            .build_dynamic_client([1; Address::SIZE].into());

        let mut chain_connector =
            create_trustful_hopr_blokli_connector(&BOB_KP, Default::default(), blokli_sim, [1; Address::SIZE].into())
                .await?;
        chain_connector.connect().await?;
        let node = Arc::new(ChainNode(Arc::new(chain_connector)));

        let strategy: Box<dyn crate::strategy::Strategy + Send> = NonAnonymousPixStrategy::new(
            NonAnonymousPixStrategyConfig {
                price_per_byte: HoprBalance::new_base(1),
                max_ssa_allocation: HoprBalance::new_base(100),
                max_deposit_tracking_time: Duration::from_secs(60),
                pix_recovery_db_path: None,
            },
            Duration::from_secs(60),
        )
        .build(node);

        assert_eq!(strategy.to_string(), "non_anonymous_pix");
        fn assert_send<T: Send>(_: T) {}
        assert_send(strategy);

        Ok(())
    }

    /// Duplicate NewDepositAddress events with the same ID must be silently skipped.
    #[test_log::test(tokio::test)]
    async fn test_new_deposit_address_dedup_skips_duplicate() -> anyhow::Result<()> {
        let price_per_byte = HoprBalance::new_base(1);
        let max_ssa_allocation = HoprBalance::new_base(100);
        let quota = 20_u64;

        let deposit_address: Address = [0x42u8; 20].into();
        let duplicate_id = (HoprPseudonym::random(), NonZeroU32::new(1).unwrap());

        let blokli_sim = BlokliTestStateBuilder::default()
            .with_generated_accounts(
                &[&*ALICE, &*BOB, &*CHRIS],
                false,
                XDaiBalance::new_base(1),
                HoprBalance::new_base(1000),
            )
            .with_balances([(*BOB, HoprBalance::new_base(1000))])
            .build_dynamic_client([1; Address::SIZE].into());

        let mut chain_connector =
            create_trustful_hopr_blokli_connector(&BOB_KP, Default::default(), blokli_sim, [1; Address::SIZE].into())
                .await?;
        chain_connector.connect().await?;
        let chain_connector = Arc::new(chain_connector);

        let cfg = NonAnonymousPixStrategyConfig {
            price_per_byte,
            max_ssa_allocation,
            max_deposit_tracking_time: Duration::from_secs(5),
            pix_recovery_db_path: None,
        };

        let mut strategy = NonAnonymousPixStrategyInner {
            cfg,
            interval: Duration::from_secs(60),
            node: Arc::new(ChainNode(Arc::clone(&chain_connector))),
            recovery_store: None,
            processed_deposits: HashSet::new(),
        };

        let bob_before = strategy.get_balance(*BOB).await?;

        // First event: should withdraw.
        let event1 = PixEvent::NewDepositAddress(hopr_api::node::PixNewDepositAddress {
            id: duplicate_id,
            address: deposit_address.into(),
            quota,
        });
        strategy.on_pix_event(event1).await?;

        let bob_mid = strategy.get_balance(*BOB).await?;
        assert_eq!(
            bob_mid,
            bob_before - HoprBalance::new_base(20),
            "first event should withdraw"
        );

        // Second event with the same ID: should be skipped.
        let event2 = PixEvent::NewDepositAddress(hopr_api::node::PixNewDepositAddress {
            id: duplicate_id,
            address: deposit_address.into(),
            quota,
        });
        strategy.on_pix_event(event2).await?;

        let bob_after = strategy.get_balance(*BOB).await?;
        assert_eq!(
            bob_after, bob_mid,
            "duplicate event must not trigger another withdrawal"
        );

        Ok(())
    }

    /// Builder with a `pix_recovery_db_path` must open the recovery store.
    #[tokio::test]
    async fn test_build_with_recovery_path_opens_store() -> anyhow::Result<()> {
        let dir = tempfile::tempdir().context("tempdir")?;
        let db_path = dir.path().join("pix_recovery.db");

        let blokli_sim = BlokliTestStateBuilder::default()
            .with_generated_accounts(
                &[&*ALICE, &*BOB],
                false,
                XDaiBalance::new_base(1),
                HoprBalance::new_base(1000),
            )
            .build_dynamic_client([1; Address::SIZE].into());

        let mut chain_connector =
            create_trustful_hopr_blokli_connector(&BOB_KP, Default::default(), blokli_sim, [1; Address::SIZE].into())
                .await?;
        chain_connector.connect().await?;
        let node = Arc::new(ChainNode(Arc::new(chain_connector)));

        let strategy: Box<dyn crate::strategy::Strategy + Send> = NonAnonymousPixStrategy::new(
            NonAnonymousPixStrategyConfig {
                price_per_byte: HoprBalance::new_base(1),
                max_ssa_allocation: HoprBalance::new_base(100),
                max_deposit_tracking_time: Duration::from_secs(60),
                pix_recovery_db_path: Some(db_path.clone()),
            },
            Duration::from_secs(60),
        )
        .build(node);

        assert!(db_path.exists(), "recovery db should be created on build");
        assert_eq!(strategy.to_string(), "non_anonymous_pix");
        Ok(())
    }

    /// PrivateKeyRecovered handler persists the key to the recovery store before withdrawing.
    #[test_log::test(tokio::test)]
    async fn test_private_key_recovered_with_recovery_store() -> anyhow::Result<()> {
        let dir = tempfile::tempdir().context("tempdir")?;
        let db_path = dir.path().join("pix_recovery.db");

        let recovered_kp = ChainKeypair::from_secret(&hex!(
            "d4945a08c3c2d47f89df2c6d3e5e7d8e9f0a1b2c3d4e5f6a7b8c9d0e1f2a3b4c"
        ))
        .expect("recovered keypair should be valid");
        let recovered_address = recovered_kp.public().to_address();

        let price_per_byte = HoprBalance::new_base(1);
        let max_ssa_allocation = HoprBalance::new_base(100);
        let recovered_initial_balance = HoprBalance::new_base(50);

        let blokli_sim = BlokliTestStateBuilder::default()
            .with_generated_accounts(
                &[&*ALICE, &*BOB, &*CHRIS, &recovered_address],
                false,
                XDaiBalance::new_base(1),
                recovered_initial_balance,
            )
            .with_balances([(recovered_address, recovered_initial_balance)])
            .build_dynamic_client([1; Address::SIZE].into());

        let mut chain_connector =
            create_trustful_hopr_blokli_connector(&BOB_KP, Default::default(), blokli_sim, [1; Address::SIZE].into())
                .await?;
        chain_connector.connect().await?;
        register_test_safe(&chain_connector, *BOB).await?;
        let chain_connector = Arc::new(chain_connector);

        let cfg = NonAnonymousPixStrategyConfig {
            price_per_byte,
            max_ssa_allocation,
            max_deposit_tracking_time: std::time::Duration::from_secs(5),
            pix_recovery_db_path: Some(db_path.clone()),
        };

        let recovery_store = Some(crate::pix_recovery_store::PixRecoveryStore::open(&db_path)?);

        let mut strategy = NonAnonymousPixStrategyInner {
            cfg: cfg.clone(),
            interval: Duration::from_secs(60),
            node: Arc::new(ChainNode(Arc::clone(&chain_connector))),
            recovery_store,
            processed_deposits: HashSet::new(),
        };

        let event_id = (HoprPseudonym::random(), NonZeroU32::new(1).unwrap());

        let event = PixEvent::PrivateKeyRecovered(hopr_api::node::PixPrivateKeyRecovered {
            id: event_id,
            secret: hopr_api::node::PixDepositSecret(recovered_kp.secret().clone()),
        });

        strategy.on_pix_event(event).await?;

        // Verify the on-chain balance was swept *before* dropping the strategy.
        let recovered_balance = strategy.get_balance(recovered_address).await?;
        assert_eq!(
            recovered_balance,
            HoprBalance::zero(),
            "recovered balance should be zero after withdrawal"
        );

        // Drop the strategy so the redb file lock is released, then re-open to verify persistence.
        drop(strategy);

        let store = crate::pix_recovery_store::PixRecoveryStore::open(&db_path)?;
        assert!(
            store.contains(&event_id).unwrap(),
            "key should be persisted to recovery store"
        );

        Ok(())
    }
}
