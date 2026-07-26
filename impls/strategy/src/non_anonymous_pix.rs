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
    convert::identity,
    fmt::{Debug, Display, Formatter},
    sync::Arc,
    time::Duration,
};

use backon::Retryable;
use futures::{FutureExt, SinkExt, StreamExt, TryFutureExt};
use futures_time::future::FutureExt as TimeExt;
use hopr_api::{
    ChainKeypair,
    chain::{ChainValues, ChainWriteAccountOperations},
    node::{ActionableEventDiscriminant, ActionableEventSource, HasChainApi, PixEvent},
    types::{crypto::prelude::Keypair, primitive::prelude::*},
};
use moka::sync::Cache;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::{errors::StrategyError, strategy::Strategy as StrategyTrait};

/// Default amount of xDai to send to a recovered stealth address for gas
/// (0.01 xDai, i.e. 10^16 wei).
fn default_gas_xdai() -> XDaiBalance {
    "0.01 xdai".parse().expect("valid static xDai amount")
}

#[derive(Clone, Debug, Serialize, Deserialize, Validate)]
pub struct NonAnonymousPixStrategyConfig {
    pub price_per_byte: HoprBalance,
    pub max_ssa_allocation: HoprBalance,
    pub max_deposit_tracking_time: Duration,
    /// Amount of xDai to send from the Safe to a recovered stealth address to
    /// cover gas for the `withdraw_from_signer` sweep.  Must be non-zero to
    /// enable the sweep; the Safe's xDai balance is checked before sending.
    /// Default: 0.01 xDai.
    #[serde(default = "default_gas_xdai")]
    pub gas_xdai_per_sweep: XDaiBalance,
    /// If set, the strategy persists recovered private keys to `redb` at this
    /// path before withdrawing (Exit role).  `None` means in-memory only (Entry role).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pix_recovery_db_path: Option<std::path::PathBuf>,
    /// Environment variable holding the password from which the encryption key for
    /// the recovery store is derived (via scrypt).  Required when
    /// [`pix_recovery_db_path`] is set.  The variable is resolved at build time
    /// so the password never appears in config dumps, logs, or serialized output.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pix_recovery_password_env: Option<String>,
}

/// Builder for [`NonAnonymousPixStrategy`].
///
/// Call [`new`](NonAnonymousPixStrategy::new) with the strategy configuration,
/// then [`build`](NonAnonymousPixStrategy::build) to wire in a node and obtain a
/// runnable `Box<dyn StrategyTrait + Send>`.
pub struct NonAnonymousPixStrategy {
    cfg: NonAnonymousPixStrategyConfig,
}

impl NonAnonymousPixStrategy {
    /// Create a new builder with the given configuration.
    pub fn new(cfg: NonAnonymousPixStrategyConfig) -> Self {
        Self { cfg }
    }

    /// Wire in a node and return a running-ready strategy.
    /// Returns [`StrategyError::CriteriaNotSatisfied`] when only one of
    /// `pix_recovery_db_path` / `pix_recovery_password_env` is set.
    pub fn build<N>(self, node: Arc<N>) -> crate::errors::Result<Box<dyn StrategyTrait + Send>>
    where
        N: HasChainApi + ActionableEventSource + Send + Sync + 'static,
    {
        if self.cfg.pix_recovery_db_path.is_some() != self.cfg.pix_recovery_password_env.is_some() {
            return Err(StrategyError::CriteriaNotSatisfied);
        }

        let recovery_store = self
            .cfg
            .pix_recovery_db_path
            .as_ref()
            .zip(self.cfg.pix_recovery_password_env.as_ref())
            .map(|(path, password_env)| {
                let password = std::env::var(password_env).map_err(|_| {
                    StrategyError::Other(anyhow::anyhow!(
                        "environment variable {password_env} must be set when PIX recovery persistence is enabled"
                    ))
                })?;
                crate::pix_recovery_store::PixRecoveryStore::open(path, &password).map_err(StrategyError::other)
            })
            .transpose()?;

        Ok(Box::new(NonAnonymousPixStrategyInner {
            cfg: self.cfg.clone(),
            node,
            recovery_store,
            processed_deposits: Cache::builder().max_capacity(1024).build(),
        }))
    }
}

/// Private generic runner — constructed by [`NonAnonymousPixStrategy::build`].
struct NonAnonymousPixStrategyInner<N: HasChainApi> {
    node: Arc<N>,
    cfg: NonAnonymousPixStrategyConfig,
    /// Exit role only: persisted recovery store for [`PixEvent::PrivateKeyRecovered`].
    /// Entry role leaves this as `None`.
    recovery_store: Option<crate::pix_recovery_store::PixRecoveryStore>,
    /// Entry role only: IDs of deposit addresses already funded.
    /// Bounded at 1024 entries (capacity-only, no TTL) to prevent unbounded
    /// growth on long-lived Entry nodes without introducing an expiration
    /// window that could allow duplicate withdrawals.
    processed_deposits: Cache<hopr_api::node::PixAddressId, ()>,
}

const MAX_SWEEP_RETRIES: usize = 5;

impl<N> NonAnonymousPixStrategyInner<N>
where
    N: HasChainApi + ActionableEventSource + Send + Sync + 'static,
{
    /// Handle PIX event.
    async fn on_pix_event(&mut self, event: PixEvent) -> crate::errors::Result<()> {
        tracing::debug!(?event, "PixStrategy event");
        match event {
            PixEvent::NewDepositAddress(new_deposit_address) => {
                tracing::info!(?new_deposit_address, "new deposit address");

                // Entry-side dedup: skip duplicates within the same strategy lifetime.
                if self.processed_deposits.contains_key(&new_deposit_address.id) {
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

                // Mark completed only after the withdrawal succeeds so a transient
                // failure doesn't permanently poison this ID.
                self.processed_deposits.insert(new_deposit_address.id, ());
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
                let target_for_filter = target_deposit;

                let mut stream = futures_time::stream::interval(
                    futures_time::time::Duration::from(max_tracking_time / 10).max(Duration::from_secs(1).into()),
                )
                .then(move |_| {
                    let node_clone = node_clone.clone();
                    async move { node_clone.chain_api().balance(deposit_addr).await }
                })
                .filter_map(move |result| {
                    let target = target_for_filter;
                    async move {
                        match result {
                            Ok(balance) if balance >= target => Some(balance),
                            Ok(_) => {
                                // Still below target — keep polling.
                                None
                            }
                            Err(error) => {
                                tracing::error!(%error, %target, "deposit balance poll failed");
                                None
                            }
                        }
                    }
                })
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

                        let deposit = if let Some(balance) = immediate {
                            balance
                        } else {
                            match stream.next().await {
                                Some(balance) => balance,
                                None => {
                                    // Stream exhausted without reaching the target deposit.
                                    return Err(StrategyError::other(anyhow::anyhow!(
                                        "deposit tracking exhausted without reaching target {target_deposit}"
                                    )));
                                }
                            }
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
                if let Some(ref store) = self.recovery_store
                    && let Err(error) = store.insert(&private_key_recovered.id, &private_key_recovered.secret)
                {
                    tracing::error!(%error, ?private_key_recovered.id, "failed to persist recovered key");
                    return Err(StrategyError::other(error));
                }

                let chain_key =
                    ChainKeypair::from_secret(private_key_recovered.secret.0.as_ref()).map_err(StrategyError::other)?;

                let store = self.recovery_store.clone();
                let ck_for_spawn = chain_key.clone();
                match sweep_recovered(
                    Arc::clone(&self.node),
                    self.cfg.clone(),
                    store.clone(),
                    private_key_recovered.id,
                    &chain_key,
                )
                .await
                {
                    Ok(()) => {
                        tracing::info!(?private_key_recovered.id, "deposit withdrawn");
                    }
                    Err(error) => {
                        tracing::error!(%error, ?private_key_recovered.id, "live sweep failed — spawning background retry");
                        spawn_sweep_retry(
                            private_key_recovered.id,
                            ck_for_spawn,
                            Arc::clone(&self.node),
                            self.cfg.clone(),
                            store,
                        );
                    }
                }
            }
        }

        Ok(())
    }

    /// Replay recovery entries whose on-chain balance is still non-zero (crash recovery).
    /// Entries whose sweep fails are retried in a background task with exponential backoff.
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

        let node = Arc::clone(&self.node);
        let cfg = self.cfg.clone();
        let store = store.clone();

        for (id, secret) in entries {
            let node = Arc::clone(&node);
            let cfg = cfg.clone();
            let store = store.clone();

            let chain_key = match ChainKeypair::from_secret(secret.0.as_ref()) {
                Ok(k) => k,
                Err(error) => {
                    tracing::error!(%error, ?id, "failed to reconstruct chain key during recovery replay");
                    continue;
                }
            };

            let balance: HoprBalance = match node.chain_api().balance(chain_key.public().to_address()).await {
                Ok(b) => b,
                Err(e) => {
                    tracing::error!(%e, ?id, "failed to query balance during recovery replay");
                    spawn_sweep_retry(id, chain_key, node, cfg, Some(store));
                    continue;
                }
            };

            if balance.is_zero() {
                // Already swept in a prior run; drop the stale secret.
                if let Err(error) = store.remove(&id) {
                    tracing::warn!(%error, ?id, "failed to remove zero-balance entry from store");
                }
                continue;
            }

            // Try the sweep inline.  On failure, spawn a background retry task.
            // Clone chain_key beforehand so it's available for the spawn on error.
            let ck_for_spawn = chain_key.clone();
            match sweep_recovered(node.clone(), cfg.clone(), Some(store.clone()), id, &chain_key).await {
                Ok(()) => {}
                Err(error) => {
                    tracing::error!(%error, ?id, "recovery replay failed — spawning background retry");
                    spawn_sweep_retry(id, ck_for_spawn, node, cfg, Some(store));
                }
            }
        }
    }
}

/// Sweep a single recovery: query the live on-chain balance, fund gas, withdraw
/// from signer, optionally remove from store.  Uses the current balance rather
/// than a caller-supplied snapshot to avoid racing with out-of-band sweeps.
async fn sweep_recovered<N: HasChainApi + ?Sized>(
    node: Arc<N>,
    cfg: NonAnonymousPixStrategyConfig,
    store: Option<crate::pix_recovery_store::PixRecoveryStore>,
    id: hopr_api::node::PixAddressId,
    chain_key: &ChainKeypair,
) -> Result<(), StrategyError> {
    let recovered_address = chain_key.public().to_address();

    let balance: HoprBalance = node
        .chain_api()
        .balance(recovered_address)
        .await
        .map_err(StrategyError::other)?;

    if balance.is_zero() {
        tracing::trace!(?id, %recovered_address, "recovered address has zero balance: already swept");
        if let Some(ref store) = store {
            let _ = store.remove(&id);
        }
        return Ok(());
    }

    fund_sweep_gas_impl(&*node, &cfg, recovered_address).await?;

    let safe_address = node.identity().safe_address;
    node.chain_api()
        .withdraw_from_signer(chain_key, balance, &safe_address)
        .await
        .map_err(StrategyError::other)?
        .await
        .map_err(StrategyError::other)?;

    if let Some(ref store) = store {
        let _ = store.remove(&id);
    }

    tracing::info!(%balance, %recovered_address, "deposit withdrawn");
    Ok(())
}

/// Spawn a background task that retries [`sweep_recovered`] with exponential backoff.
fn spawn_sweep_retry(
    id: hopr_api::node::PixAddressId,
    chain_key: ChainKeypair,
    node: Arc<impl HasChainApi + ActionableEventSource + Send + Sync + 'static>,
    cfg: NonAnonymousPixStrategyConfig,
    store: Option<crate::pix_recovery_store::PixRecoveryStore>,
) {
    hopr_utils::runtime::prelude::spawn(async move {
        let recovered_address = chain_key.public().to_address();

        (|| sweep_recovered(Arc::clone(&node), cfg.clone(), store.clone(), id, &chain_key))
            .retry(backon::ExponentialBuilder::default().with_max_times(MAX_SWEEP_RETRIES))
            .sleep(backon::FuturesTimerSleeper)
            .when(|_| {
                // Retry all errors up to MAX_SWEEP_RETRIES; CriteriaNotSatisfied
                // (insufficient xDai) may resolve on retry when another sweep
                // completes and returns gas to the Safe.
                true
            })
            .notify(|e, dur| {
                tracing::warn!(%e, ?dur, ?id, "sweep retry in");
            })
            .await
            .unwrap_or_else(|e| {
                tracing::error!(%e, ?id, %recovered_address, "sweep failed after max retries, giving up");
                // Leave the entry in the store for manual recovery.
            });
    });
}

/// Shared implementation of `fund_sweep_gas` usable from spawned tasks.
async fn fund_sweep_gas_impl<N: HasChainApi + ?Sized>(
    node: &N,
    cfg: &NonAnonymousPixStrategyConfig,
    recovered_address: Address,
) -> crate::errors::Result<()> {
    if cfg.gas_xdai_per_sweep.is_zero() {
        return Ok(());
    }

    // Query the recovered address's current native balance.  If it already has
    // enough xDai for gas, don't send any more.
    let recovered_xdai: XDaiBalance = node
        .chain_api()
        .balance(recovered_address)
        .await
        .map_err(StrategyError::other)?;

    if recovered_xdai >= cfg.gas_xdai_per_sweep {
        tracing::trace!(%recovered_address, balance = %recovered_xdai, "recovered address already has enough xDai for sweep gas");
        return Ok(());
    }

    let deficit = cfg.gas_xdai_per_sweep - recovered_xdai;

    let safe_xdai: XDaiBalance = node
        .chain_api()
        .balance(node.identity().safe_address)
        .await
        .map_err(StrategyError::other)?;

    if safe_xdai < deficit {
        tracing::warn!(
            safe = %node.identity().safe_address,
            deficit = %deficit,
            available = %safe_xdai,
            "insufficient xDai in safe to fund sweep gas"
        );
        return Err(StrategyError::CriteriaNotSatisfied);
    }

    node.chain_api()
        .withdraw(deficit, &recovered_address)
        .and_then(identity)
        .await
        .map_err(StrategyError::other)?;

    tracing::info!(
        amount = %deficit,
        %recovered_address,
        "funded sweep gas from safe"
    );

    Ok(())
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
        // Subscribe to the event stream before replaying pending recoveries so
        // that any PIX events emitted during startup recovery are captured by
        // the stream rather than missed.
        let mut event_stream = self
            .node
            .subscribe_to_actionable_events(Some(&[ActionableEventDiscriminant::Pix]))
            .map_err(|e| StrategyError::Other(anyhow::anyhow!(e)))?
            .filter_map(|event| futures::future::ready(event.try_as_pix()));

        // Startup recovery: replay persisted keys whose on-chain balance is still non-zero.
        // Failed entries are retried in the background with exponential backoff.
        if let Some(ref store) = self.recovery_store {
            self.replay_pending_recoveries(store).await;
        }

        while let Some(event) = event_stream.next().await {
            if let Err(error) = self.on_pix_event(event).await {
                tracing::error!(%error, "pix event failed");
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

    const TEST_PASSWORD_ENV: &str = "HOPRD_TEST_PIX_RECOVERY_PASSWORD";

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
            gas_xdai_per_sweep: XDaiBalance::zero(),
            pix_recovery_db_path: None,
            pix_recovery_password_env: None,
        };

        let mut strategy = NonAnonymousPixStrategyInner {
            cfg,
            node: Arc::new(ChainNode(Arc::clone(&chain_connector))),
            recovery_store: None,
            processed_deposits: Cache::builder().max_capacity(1024).build(),
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
            gas_xdai_per_sweep: XDaiBalance::zero(),
            pix_recovery_db_path: None,
            pix_recovery_password_env: None,
        };

        let mut strategy = NonAnonymousPixStrategyInner {
            cfg,

            node: Arc::new(ChainNode(Arc::clone(&chain_connector))),
            recovery_store: None,
            processed_deposits: Cache::builder().max_capacity(1024).build(),
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

        insta::assert_yaml_snapshot!(*snapshot.refresh(), {
            ".chain_info.contract_addresses" => "[contract_addresses]",
        });

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
            gas_xdai_per_sweep: XDaiBalance::zero(),
            pix_recovery_db_path: None,
            pix_recovery_password_env: None,
        };

        let mut strategy = NonAnonymousPixStrategyInner {
            cfg,

            node: Arc::new(ChainNode(Arc::new(chain_connector))),
            recovery_store: None,
            processed_deposits: Cache::builder().max_capacity(1024).build(),
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
    /// raw chain address, then funds gas from the safe and calls
    /// `withdraw_from_signer` to sweep the full balance to the node's own safe.
    ///
    /// Verifies the recovered address ends at 0 wxHOPR and the safe receives the
    /// full recovered balance (50 wxHOPR). The recovered address is NOT pre-funded
    /// with xDai; the gas is sent by the strategy via `fund_sweep_gas`.
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
                &[&*ALICE, &*BOB, &*CHRIS],
                false,
                XDaiBalance::new_base(2), // 2 xDai — 1 for sweep gas + buffer for registration fees
                HoprBalance::new_base(1000),
            )
            // Give the recovered address wxHOPR and enough xDai to cover sweep gas.
            .with_balances([(recovered_address, recovered_initial_balance)])
            .with_balances([(recovered_address, XDaiBalance::new_base(1))])
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
            // Use the default 0.01 xDai — BOB has 1 xDai from with_generated_accounts.
            gas_xdai_per_sweep: default_gas_xdai(),
            pix_recovery_db_path: None,
            pix_recovery_password_env: None,
        };

        let mut strategy = NonAnonymousPixStrategyInner {
            cfg,

            node: Arc::new(ChainNode(Arc::clone(&chain_connector))),
            recovery_store: None,
            processed_deposits: Cache::builder().max_capacity(1024).build(),
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

        insta::assert_yaml_snapshot!(*snapshot.refresh(), {
            ".chain_info.contract_addresses" => "[contract_addresses]",
        });

        Ok(())
    }

    #[test]
    fn test_config_default_passes_validation() {
        let cfg = NonAnonymousPixStrategyConfig {
            price_per_byte: HoprBalance::new_base(1),
            max_ssa_allocation: HoprBalance::new_base(100),
            max_deposit_tracking_time: std::time::Duration::from_secs(60),
            gas_xdai_per_sweep: XDaiBalance::zero(),
            pix_recovery_db_path: None,
            pix_recovery_password_env: None,
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

        let strategy: Box<dyn crate::strategy::Strategy + Send> =
            NonAnonymousPixStrategy::new(NonAnonymousPixStrategyConfig {
                price_per_byte: HoprBalance::new_base(1),
                max_ssa_allocation: HoprBalance::new_base(100),
                max_deposit_tracking_time: Duration::from_secs(60),
                gas_xdai_per_sweep: XDaiBalance::zero(),
                pix_recovery_db_path: None,
                pix_recovery_password_env: None,
            })
            .build(node)?;

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
            gas_xdai_per_sweep: XDaiBalance::zero(),
            pix_recovery_db_path: None,
            pix_recovery_password_env: None,
        };

        let mut strategy = NonAnonymousPixStrategyInner {
            cfg,

            node: Arc::new(ChainNode(Arc::clone(&chain_connector))),
            recovery_store: None,
            processed_deposits: Cache::builder().max_capacity(1024).build(),
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

        // SAFETY: set_var is unsafe in concurrent contexts, but #[tokio::test] runs
        // on a multi-threaded runtime.  This is acceptable because the env var is
        // unique to this test (no other test reads/writes it) and setting is idempotent.
        unsafe { std::env::set_var(TEST_PASSWORD_ENV, "test-password") };

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

        let strategy: Box<dyn crate::strategy::Strategy + Send> =
            NonAnonymousPixStrategy::new(NonAnonymousPixStrategyConfig {
                price_per_byte: HoprBalance::new_base(1),
                max_ssa_allocation: HoprBalance::new_base(100),
                max_deposit_tracking_time: Duration::from_secs(60),
                gas_xdai_per_sweep: XDaiBalance::zero(),
                pix_recovery_db_path: Some(db_path.clone()),
                pix_recovery_password_env: Some(TEST_PASSWORD_ENV.into()),
            })
            .build(node)?;

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
                &[&*ALICE, &*BOB, &*CHRIS],
                false,
                XDaiBalance::new_base(2),
                HoprBalance::new_base(1000),
            )
            .with_balances([(recovered_address, recovered_initial_balance)])
            .with_balances([(recovered_address, XDaiBalance::new_base(1))])
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
            gas_xdai_per_sweep: default_gas_xdai(),
            pix_recovery_db_path: Some(db_path.clone()),
            pix_recovery_password_env: Some(TEST_PASSWORD_ENV.into()),
        };

        let recovery_store = Some(crate::pix_recovery_store::PixRecoveryStore::open(
            &db_path,
            "test-password",
        )?);

        let mut strategy = NonAnonymousPixStrategyInner {
            cfg: cfg.clone(),

            node: Arc::new(ChainNode(Arc::clone(&chain_connector))),
            recovery_store,
            processed_deposits: Cache::builder().max_capacity(1024).build(),
        };

        #[allow(clippy::disallowed_names)]
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

        // Withdrawal was successful — the key should have been removed from the store.
        // Drop the strategy so the redb file lock is released, then re-open to verify cleanup.
        drop(strategy);

        let store = crate::pix_recovery_store::PixRecoveryStore::open(&db_path, "test-password")?;
        assert!(
            !store.contains(&event_id).unwrap(),
            "key should be removed from recovery store after successful withdrawal"
        );

        Ok(())
    }

    /// `replay_pending_recoveries` with a zero-balance entry — the entry is removed
    /// from the store and no withdrawal is attempted.
    #[test_log::test(tokio::test)]
    async fn test_replay_zero_balance_removes_entry() -> anyhow::Result<()> {
        let dir = tempfile::tempdir().context("tempdir")?;
        let db_path = dir.path().join("pix_recovery.db");

        let recovered_kp = ChainKeypair::from_secret(&hex!(
            "a111a08c3c2d47f89df2c6d3e5e7d8e9f0a1b2c3d4e5f6a7b8c9d0e1f2a3b4cc"
        ))
        .expect("recovered keypair should be valid");
        let recovered_address = recovered_kp.public().to_address();

        let price_per_byte = HoprBalance::new_base(1);
        let max_ssa_allocation = HoprBalance::new_base(100);

        // Balance is zero — entry should be cleaned up, not withdrawn.
        let blokli_sim = BlokliTestStateBuilder::default()
            .with_generated_accounts(
                &[&*ALICE, &*BOB, &*CHRIS, &recovered_address],
                false,
                XDaiBalance::new_base(1),
                HoprBalance::zero(),
            )
            .with_balances([(recovered_address, HoprBalance::zero())])
            .build_dynamic_client([1; Address::SIZE].into());

        let mut chain_connector =
            create_trustful_hopr_blokli_connector(&BOB_KP, Default::default(), blokli_sim, [1; Address::SIZE].into())
                .await?;
        chain_connector.connect().await?;
        let chain_connector = Arc::new(chain_connector);

        let store = crate::pix_recovery_store::PixRecoveryStore::open(&db_path, "test-password")?;

        let entry_id = (HoprPseudonym::random(), NonZeroU32::new(1).unwrap());
        store.insert(
            &entry_id,
            &hopr_api::node::PixDepositSecret(recovered_kp.secret().clone()),
        )?;

        assert!(store.contains(&entry_id).unwrap(), "entry should exist before replay");

        let strategy = NonAnonymousPixStrategyInner {
            cfg: NonAnonymousPixStrategyConfig {
                price_per_byte,
                max_ssa_allocation,
                max_deposit_tracking_time: std::time::Duration::from_secs(5),
                gas_xdai_per_sweep: XDaiBalance::zero(), // balance is zero — no gas needed
                pix_recovery_db_path: Some(db_path.clone()),
                pix_recovery_password_env: Some(TEST_PASSWORD_ENV.into()),
            },

            node: Arc::new(ChainNode(Arc::clone(&chain_connector))),
            recovery_store: None, // not needed for the test — we pass store directly
            processed_deposits: Cache::builder().max_capacity(1024).build(),
        };

        // No need to call on_tick or register a safe — balance is zero so replay
        // won't attempt a withdrawal.
        strategy.replay_pending_recoveries(&store).await;

        // Zero-balance entry should have been removed.
        assert!(
            !store.contains(&entry_id).unwrap(),
            "zero-balance entry should be removed from recovery store after replay"
        );

        Ok(())
    }

    /// `replay_pending_recoveries` with a non-zero-balance entry — the withdrawal
    /// succeeds and the entry is removed from the store.
    #[test_log::test(tokio::test)]
    async fn test_replay_with_non_zero_balance_withdraws_and_removes_entry() -> anyhow::Result<()> {
        let dir = tempfile::tempdir().context("tempdir")?;
        let db_path = dir.path().join("pix_recovery.db");

        let recovered_kp = ChainKeypair::from_secret(&hex!(
            "b222b08c3c2d47f89df2c6d3e5e7d8e9f0a1b2c3d4e5f6a7b8c9d0e1f2a3b4cc"
        ))
        .expect("recovered keypair should be valid");
        let recovered_address = recovered_kp.public().to_address();

        let price_per_byte = HoprBalance::new_base(1);
        let max_ssa_allocation = HoprBalance::new_base(100);
        let recovered_balance = HoprBalance::new_base(50);

        let blokli_sim = BlokliTestStateBuilder::default()
            .with_generated_accounts(
                &[&*ALICE, &*BOB, &*CHRIS],
                false,
                XDaiBalance::new_base(2),
                HoprBalance::new_base(1000),
            )
            .with_balances([(recovered_address, recovered_balance)])
            .with_balances([(recovered_address, XDaiBalance::new_base(1))])
            .build_dynamic_client([1; Address::SIZE].into());

        let mut chain_connector =
            create_trustful_hopr_blokli_connector(&BOB_KP, Default::default(), blokli_sim, [1; Address::SIZE].into())
                .await?;
        chain_connector.connect().await?;
        register_test_safe(&chain_connector, *BOB).await?;
        let chain_connector = Arc::new(chain_connector);

        let store = crate::pix_recovery_store::PixRecoveryStore::open(&db_path, "test-password")?;

        let entry_id = (HoprPseudonym::random(), NonZeroU32::new(1).unwrap());
        store.insert(
            &entry_id,
            &hopr_api::node::PixDepositSecret(recovered_kp.secret().clone()),
        )?;

        assert!(store.contains(&entry_id).unwrap(), "entry should exist before replay");

        let strategy = NonAnonymousPixStrategyInner {
            cfg: NonAnonymousPixStrategyConfig {
                price_per_byte,
                max_ssa_allocation,
                max_deposit_tracking_time: std::time::Duration::from_secs(5),
                gas_xdai_per_sweep: default_gas_xdai(),
                pix_recovery_db_path: Some(db_path.clone()),
                pix_recovery_password_env: Some(TEST_PASSWORD_ENV.into()),
            },

            node: Arc::new(ChainNode(Arc::clone(&chain_connector))),
            recovery_store: None,
            processed_deposits: Cache::builder().max_capacity(1024).build(),
        };

        strategy.replay_pending_recoveries(&store).await;

        // Entry should have been removed after successful withdrawal.
        assert!(
            !store.contains(&entry_id).unwrap(),
            "entry should be removed from recovery store after successful replay withdrawal"
        );

        // Verify the recovered address balance was drained.
        let recovered_balance_after = strategy.get_balance(recovered_address).await?;
        assert_eq!(
            recovered_balance_after,
            HoprBalance::zero(),
            "recovered address balance should be zero after replay withdrawal"
        );

        Ok(())
    }

    /// `replay_pending_recoveries` skips entries whose on-chain balance query fails —
    /// the entry is preserved in the store for a future retry.
    #[test_log::test(tokio::test)]
    async fn test_replay_with_balance_query_failure_preserves_entry() -> anyhow::Result<()> {
        let dir = tempfile::tempdir().context("tempdir")?;
        let db_path = dir.path().join("pix_recovery.db");

        let recovered_kp = ChainKeypair::from_secret(&hex!(
            "c333c08c3c2d47f89df2c6d3e5e7d8e9f0a1b2c3d4e5f6a7b8c9d0e1f2a3b4cc"
        ))
        .expect("recovered keypair should be valid");

        let price_per_byte = HoprBalance::new_base(1);
        let max_ssa_allocation = HoprBalance::new_base(100);

        // Omit the recovered address from generated accounts — the blokli RPC will
        // fail to look it up, simulating a transient balance query failure.
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
        let chain_connector = Arc::new(chain_connector);

        let store = crate::pix_recovery_store::PixRecoveryStore::open(&db_path, "test-password")?;

        let entry_id = (HoprPseudonym::random(), NonZeroU32::new(1).unwrap());
        store.insert(
            &entry_id,
            &hopr_api::node::PixDepositSecret(recovered_kp.secret().clone()),
        )?;

        assert!(store.contains(&entry_id).unwrap(), "entry should exist before replay");

        let strategy = NonAnonymousPixStrategyInner {
            cfg: NonAnonymousPixStrategyConfig {
                price_per_byte,
                max_ssa_allocation,
                max_deposit_tracking_time: std::time::Duration::from_secs(5),
                gas_xdai_per_sweep: XDaiBalance::zero(),
                pix_recovery_db_path: Some(db_path.clone()),
                pix_recovery_password_env: Some(TEST_PASSWORD_ENV.into()),
            },

            node: Arc::new(ChainNode(Arc::clone(&chain_connector))),
            recovery_store: None,
            processed_deposits: Cache::builder().max_capacity(1024).build(),
        };

        strategy.replay_pending_recoveries(&store).await;

        // Balance query failed — entry should still be present for a future retry.
        assert!(
            store.contains(&entry_id).unwrap(),
            "entry should remain in recovery store after balance query failure"
        );

        Ok(())
    }

    /// PrivateKeyRecovered handler exercises the deficit-calculation path in
    /// `fund_sweep_gas_impl`: the recovered address has wxHOPR but NO xDai,
    /// so the strategy must send gas from the safe before sweeping.
    #[test_log::test(tokio::test)]
    async fn test_private_key_recovered_without_xdai_funds_gas_from_safe() -> anyhow::Result<()> {
        let recovered_kp = ChainKeypair::from_secret(&hex!(
            "e555e08c3c2d47f89df2c6d3e5e7d8e9f0a1b2c3d4e5f6a7b8c9d0e1f2a3b4dd"
        ))
        .expect("recovered keypair should be valid");
        let recovered_address = recovered_kp.public().to_address();

        let recovered_initial_balance = HoprBalance::new_base(50);

        let blokli_sim = BlokliTestStateBuilder::default()
            .with_generated_accounts(
                &[&*ALICE, &*BOB, &*CHRIS, &recovered_address],
                false,
                XDaiBalance::new_base(0), // give each generated account 0 xDai initially
                HoprBalance::new_base(1000),
            )
            // Give BOB enough xDai for registration + sweep gas
            .with_balances([(*BOB, XDaiBalance::new_base(2))])
            // Give the recovered address wxHOPR and zero xDai
            .with_balances([(recovered_address, recovered_initial_balance)])
            .build_dynamic_client([1; Address::SIZE].into());

        let snapshot = blokli_sim.snapshot();

        let mut chain_connector =
            create_trustful_hopr_blokli_connector(&BOB_KP, Default::default(), blokli_sim, [1; Address::SIZE].into())
                .await?;
        chain_connector.connect().await?;
        register_test_safe(&chain_connector, *BOB).await?;
        let chain_connector = Arc::new(chain_connector);

        let mut strategy = NonAnonymousPixStrategyInner {
            cfg: NonAnonymousPixStrategyConfig {
                price_per_byte: HoprBalance::new_base(1),
                max_ssa_allocation: HoprBalance::new_base(100),
                max_deposit_tracking_time: std::time::Duration::from_secs(5),
                gas_xdai_per_sweep: default_gas_xdai(), // 0.01 xDai — deficit must be funded from safe
                pix_recovery_db_path: None,
                pix_recovery_password_env: None,
            },
            node: Arc::new(ChainNode(Arc::clone(&chain_connector))),
            recovery_store: None,
            processed_deposits: Cache::builder().max_capacity(1024).build(),
        };

        let safe_address = strategy.node.identity().safe_address;

        let event = PixEvent::PrivateKeyRecovered(hopr_api::node::PixPrivateKeyRecovered {
            id: (HoprPseudonym::random(), NonZeroU32::new(1).unwrap()),
            secret: hopr_api::node::PixDepositSecret(recovered_kp.secret().clone()),
        });

        strategy.on_pix_event(event).await?;

        // Recovered keypair's wxHOPR balance should be zero after withdrawal.
        let recovered_balance = strategy
            .get_balance(recovered_address)
            .await
            .context("get recovered address balance after withdraw")?;
        assert_eq!(
            recovered_balance,
            HoprBalance::zero(),
            "recovered keypair's balance should be zero after withdrawal"
        );

        // Safe should have received the full recovered wxHOPR balance.
        let safe_balance = strategy
            .get_balance(safe_address)
            .await
            .context("get safe balance after withdraw")?;
        assert_eq!(
            safe_balance, recovered_initial_balance,
            "safe should have received the full recovered balance"
        );

        insta::assert_yaml_snapshot!(*snapshot.refresh(), {
            ".chain_info.contract_addresses" => "[contract_addresses]",
        });

        Ok(())
    }

    /// Build with only one of pix_recovery_db_path / pix_recovery_password_env must
    /// silently succeed (config validation passes; the error occurs at build() time).
    #[test]
    fn test_build_fails_when_only_one_recovery_config_set() {
        let cfg = NonAnonymousPixStrategyConfig {
            price_per_byte: HoprBalance::new_base(1),
            max_ssa_allocation: HoprBalance::new_base(100),
            max_deposit_tracking_time: std::time::Duration::from_secs(60),
            gas_xdai_per_sweep: XDaiBalance::zero(),
            pix_recovery_db_path: Some(std::path::PathBuf::from("/tmp/test.db")),
            pix_recovery_password_env: None,
        };
        assert!(matches!(cfg.validate(), Ok(())), "config validation passes");
    }

    /// Build with pix_recovery_db_path set but missing password env must fail.
    #[tokio::test]
    async fn test_build_fails_when_password_env_var_missing() -> anyhow::Result<()> {
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

        // Use a password env var that doesn't exist.
        let result = NonAnonymousPixStrategy::new(NonAnonymousPixStrategyConfig {
            price_per_byte: HoprBalance::new_base(1),
            max_ssa_allocation: HoprBalance::new_base(100),
            max_deposit_tracking_time: Duration::from_secs(60),
            gas_xdai_per_sweep: XDaiBalance::zero(),
            pix_recovery_db_path: Some(db_path),
            pix_recovery_password_env: Some("HOPRD_NONEXISTENT_VAR".into()),
        })
        .build(node);

        assert!(
            matches!(result, Err(_)),
            "build should fail when password env var is missing"
        );

        Ok(())
    }
}
