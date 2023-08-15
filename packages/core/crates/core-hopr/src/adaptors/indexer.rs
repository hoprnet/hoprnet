use std::{sync::Arc, pin::Pin};

use async_lock::RwLock;
use core_crypto::types::PublicKey;
use core_ethereum_db::traits::HoprCoreEthereumDbActions;
use core_network::network::Network;
use core_p2p::libp2p_swarm::derive_prelude::Multiaddr;
use futures::{
    StreamExt,
    channel::mpsc::Sender,
    future::poll_fn
};

#[cfg(any(not(feature = "wasm"), test))]
use async_std::task::spawn_local;

use utils_types::traits::PeerIdLike;
#[cfg(all(feature = "wasm", not(test)))]
use wasm_bindgen_futures::spawn_local;

use core_network::PeerId;
use utils_log::{warn,error};
use utils_types::primitives::Address;

use crate::errors::Result;
use crate::CoreEthereumDb;
use crate::LevelDbShim;

use super::network::ExternalNetworkInteractions;

pub const INDEXER_UPDATE_QUEUE_SIZE: usize = 4096;


async fn is_allowed_to_access_network<T>(db: &T, chain_address: &Address) -> Result<bool>
where
    T: HoprCoreEthereumDbActions,
{
    let nr_enabled = db.is_network_registry_enabled().await?;

    if !nr_enabled {
        return Ok(true);
    }

    let maybe_stake_account = db.get_account_from_network_registry(&chain_address).await?;

    match maybe_stake_account {
        None => Ok(false),
        Some(account) => Ok(db.is_eligible(&account).await?),
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PeerEligibility {
    Eligible,
    Ineligible
}

impl From<bool> for PeerEligibility {
    fn from(value: bool) -> Self {
        if value {
            PeerEligibility::Eligible
        } else {
            PeerEligibility::Ineligible
        }
    }
}

pub enum IndexerToProcess {
    EligibilityUpdate(PeerId, PeerEligibility),
    RegisterStatusUpdate,
    Announce(PeerId, Vec<Multiaddr>)
}

#[derive(Debug)]
pub enum IndexerProcessed {
    Allow(PeerId),
    Ban(PeerId),
    Announce(PeerId, Vec<Multiaddr>)
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct WasmIndexerInteractions {
    internal_emitter: Sender<IndexerToProcess>,
}

impl WasmIndexerInteractions {
    pub fn new(db: Arc<RwLock<CoreEthereumDb<LevelDbShim>>>,
        network: Arc<RwLock<Network<ExternalNetworkInteractions>>>,
        emitter: Sender<IndexerProcessed>) -> Self 
    {
        let (to_process_tx, mut to_process_rx) = futures::channel::mpsc::channel::<IndexerToProcess>(INDEXER_UPDATE_QUEUE_SIZE);

        spawn_local(async move {
            let mut emitter = emitter; 
            let db = db;

            while let Some(value) = to_process_rx.next().await {
                let event = match value {
                    IndexerToProcess::EligibilityUpdate(peer, eligibility) => {
                        match eligibility {
                            PeerEligibility::Eligible => IndexerProcessed::Allow(peer),
                            PeerEligibility::Ineligible => IndexerProcessed::Ban(peer),
                        }
                    },
                    IndexerToProcess::Announce(peer, multiaddress) => IndexerProcessed::Announce(peer, multiaddress),
                    IndexerToProcess::RegisterStatusUpdate => {
                        let peers = (*network.read().await).get_all_peers();

                        for peer in peers.into_iter() {
                            let is_allowed = {
                                let address = {
                                    let a = PublicKey::from_peerid(&peer)
                                        .map(|v| v.to_address());
                                    if a.is_ok() {
                                        a.unwrap()
                                    } else {
                                        continue
                                    }
                                };
                                
                                match is_allowed_to_access_network(&(*db.read().await), &address).await {
                                    Ok(v) => v,
                                    Err(_) => continue,
                                }
                            };

                            let event = if is_allowed {
                                IndexerProcessed::Allow(peer)
                            } else {
                                (*network.write().await).remove(&peer);
                                IndexerProcessed::Ban(peer)
                            };
                            
                            match poll_fn(|cx| Pin::new(&mut emitter).poll_ready(cx)).await {
                                Ok(_) => {
                                    match emitter.start_send(event) {
                                        Ok(_) => {},
                                        Err(e) => error!("Failed to emit an indexer event: {}", e),
                                    }
                                },
                                Err(e) => {
                                    warn!("The receiver for processed indexer events no longer exists: {}", e);
                                }
                            };
                        }
                        continue
                    },
                };

                match poll_fn(|cx| Pin::new(&mut emitter).poll_ready(cx)).await {
                    Ok(_) => {
                        match emitter.start_send(event) {
                            Ok(_) => {},
                            Err(e) => error!("Failed to emit an indexer event: {}", e),
                        }
                    },
                    Err(e) => {
                        warn!("The receiver for processed indexer events no longer exists: {}", e);
                    }
                };
            }
        });

        Self { internal_emitter: to_process_tx }
    }
}


#[cfg(feature = "wasm")]
pub mod wasm {
    use std::{str::FromStr, pin::Pin};

    use super::*;
    use futures::future::poll_fn;
    use js_sys::JsString;
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    impl WasmIndexerInteractions {
        pub async fn update_peer_eligibility(&mut self, peer: JsString, eligible: bool) {
            let peer: String = peer.into();
            match PeerId::from_str(&peer) {
                Ok(p) => {
                    match poll_fn(|cx| Pin::new(&mut self.internal_emitter).poll_ready(cx)).await {
                        Ok(_) => {
                            match self.internal_emitter.start_send(IndexerToProcess::EligibilityUpdate(p, eligible.into())) {
                                Ok(_) => {},
                                Err(e) => error!("Failed to send register update 'eligibility' to the receiver: {}", e),
                            }
                        }
                        Err(e) => error!("The receiver for indexer updates was dropped: {}", e)
                    }
                },
                Err(err) => {
                    warn!(
                        "Failed to parse peer id {}, cannot update eligibility to {}: {}",
                        peer,
                        eligible,
                        err.to_string()
                    );
                }
            }
        }

        pub async fn register_status_update(&mut self) {
            match poll_fn(|cx| Pin::new(&mut self.internal_emitter).poll_ready(cx)).await {
                Ok(_) => {
                    match self.internal_emitter.start_send(IndexerToProcess::RegisterStatusUpdate) {
                        Ok(_) => {},
                        Err(e) => error!("Failed to send register update 'register status' to the receiver: {}", e),
                    }
                }
                Err(e) => error!("The receiver for indexer updates was dropped: {}", e)
            }
        }

        pub async fn announce(&mut self, peer: JsString, multiaddresses: js_sys::Array) {
            let peer: String = peer.into();
            match PeerId::from_str(&peer) {
                Ok(p) => {
                    let mas = multiaddresses.to_vec()
                        .into_iter()
                        .filter_map(|v| {
                            let v: String = JsString::from(v).into();
                            Multiaddr::from_str(&v).ok()
                        })
                        .collect::<Vec<Multiaddr>>();

                    match poll_fn(|cx| Pin::new(&mut self.internal_emitter).poll_ready(cx)).await {
                        Ok(_) => {
                            match self.internal_emitter.start_send(IndexerToProcess::Announce(p, mas)) {
                                Ok(_) => {},
                                Err(e) => error!("Failed to send indexer update 'announce' to the receiver: {}", e),
                            }
                        }
                        Err(e) => error!("The receiver for indexer updates was dropped: {}", e)
                    }
                },
                Err(err) => {
                    warn!(
                        "Failed to parse peer id {}, cannot announce multiaddresses: {}",
                        peer,
                        err.to_string()
                    );
                }
            }
        }
    }
}