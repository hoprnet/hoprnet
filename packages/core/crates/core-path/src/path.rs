use crate::channel_graph::ChannelGraph;
use crate::errors::PathError;
use crate::errors::PathError::{ChannelNotOpened, InvalidPeer, LoopsNotAllowed, MissingChannel, PathNotValid};
use crate::errors::Result;
use core_crypto::types::OffchainPublicKey;
use core_types::channels::ChannelStatus;
use core_types::protocol::PeerAddressResolver;
use futures::future::FutureExt;
use futures::stream::FuturesOrdered;
use futures::TryStreamExt;
use libp2p_identity::PeerId;
use std::collections::hash_map::RandomState;
use std::collections::HashSet;
use std::fmt::{Display, Formatter};
use std::hash::Hash;
use utils_log::warn;
use utils_types::primitives::Address;
use utils_types::traits::{PeerIdLike, ToHex};

/// Base implementation of an abstract path.
/// Must contain always at least a single hop
pub trait BasePath<N>: Display + Clone + Eq + PartialEq
where
    N: Copy + Clone + Eq + PartialEq + Hash,
{
    /// Individual hops in the path.
    /// There must be always at least one hop.
    fn hops(&self) -> &[N];

    /// Shorthand for number of hops.
    fn length(&self) -> usize {
        self.hops().len()
    }

    /// Gets the last hop
    fn last_hop(&self) -> &N {
        self.hops().last().expect("path is invalid")
    }

    /// Checks if the path contains simple hops between the same addresses.
    /// If `true`, implies `is_cyclic()` to be `true` as well.
    fn has_simple_loops(&self) -> bool {
        let mut last_addr = self.hops()[0];
        for addr in self.hops().iter().skip(1) {
            if last_addr.eq(addr) {
                return true;
            }
            last_addr = *addr;
        }
        false
    }

    /// Checks if all the hops in this path are to distinct addresses.
    /// Returns `true` if there are duplicate Addresses on this path.
    /// If `true` does not imply `has_simple_loops()` to be necessarily `true` as well.
    fn is_cyclic(&self) -> bool {
        let set = HashSet::<&N, RandomState>::from_iter(self.hops().iter());
        set.len() != self.hops().len()
    }
}

/// Represents an on-chain path in the `ChannelGraph`.
/// The path is never allowed to be empty and is always constructed from
/// hops that must be known to have open channels (at the time of construction).
/// The path may or may not contain simple loops (same adjacent nodes).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChannelPath {
    pub(crate) hops: Vec<Address>,
}

impl ChannelPath {
    /// Creates a new path by validating the list of peer ids using the channel graph.
    /// The given path does not contain the sender node, which assume to be this node.
    pub fn new(hops: Vec<Address>, allow_loops: bool, graph: &ChannelGraph) -> Result<Self> {
        if hops.is_empty() {
            return Err(PathNotValid);
        }

        let mut ticket_receiver;
        let mut ticket_issuer = graph.my_address();

        // Ignore the last hop in the check, because channels are not required for direct messages
        for hop in hops.iter().take(hops.len() - 1) {
            ticket_receiver = *hop;

            // Check for loops
            if ticket_issuer == ticket_receiver {
                if !allow_loops {
                    return Err(LoopsNotAllowed(ticket_receiver.to_hex()));
                }
                warn!("duplicated adjacent path entries")
            }

            // Check if the channel is opened
            let channel = graph
                .get_channel(ticket_issuer, ticket_receiver)
                .ok_or(MissingChannel(ticket_issuer.to_hex(), ticket_receiver.to_hex()))?;

            if channel.status != ChannelStatus::Open {
                return Err(ChannelNotOpened(ticket_issuer.to_hex(), ticket_receiver.to_hex()));
            }

            ticket_issuer = ticket_receiver;
        }

        Ok(Self { hops })
    }

    pub(crate) fn new_valid(hops: Vec<Address>) -> Self {
        assert!(!hops.is_empty(), "must not be empty");
        Self { hops }
    }

    /// Resolves this on-chain `ChannelPath` into the off-chain `Path`.
    pub async fn to_path<R: PeerAddressResolver>(&self, resolver: &R) -> Result<Path> {
        let hops = self
            .hops
            .iter()
            .map(|addr| {
                resolver.resolve_packet_key(addr).map(move |opt| {
                    opt.map(|k| k.to_peerid())
                        .ok_or(InvalidPeer(format!("could not resolve off-chain key for {addr}")))
                })
            })
            .collect::<FuturesOrdered<_>>()
            .try_collect::<Vec<_>>()
            .await?;

        Ok(Path::new_valid(hops))
    }
}

impl BasePath<Address> for ChannelPath {
    fn hops(&self) -> &[Address] {
        &self.hops
    }
}

impl Display for ChannelPath {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[ {} ] ({} hops)",
            self.hops.iter().map(|p| p.to_string()).collect::<Vec<_>>().join("->"),
            self.hops.len()
        )
    }
}

/// Represents an off-chain path of `PeerId`s.
/// The path is never allowed to be empty and is always constructed from
/// hops that must be known to have open channels (at the time of construction).
/// The path may or may not contain simple loops (same adjacent nodes).
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Path {
    hops: Vec<PeerId>,
}

impl Path {
    /// Resolves vector of `PeerIds` into the corresponding `Path` and `ChannelPath` pair.
    /// This works by first resolving `PeerId`s into `Address`es and then validating the `ChannelPath`.
    /// To do an inverse resolution, from `Address`es to `PeerId`s, construct the `ChannelPath` and use `ChannelPath::to_path()` to resolve the
    /// on-chain path.
    pub async fn resolve<R: PeerAddressResolver>(
        peers: Vec<PeerId>,
        allow_loops: bool,
        resolver: &R,
        graph: &ChannelGraph,
    ) -> Result<(Self, ChannelPath)> {
        let (addrs, hops) = peers
            .into_iter()
            .map(|peer| async move {
                let key = OffchainPublicKey::from_peerid(&peer)?;
                resolver
                    .resolve_chain_key(&key)
                    .await
                    .map(|addr| (addr, peer))
                    .ok_or(InvalidPeer(peer.to_string()))
            })
            .collect::<FuturesOrdered<_>>()
            .try_collect::<Vec<_>>()
            .await?
            .into_iter()
            .unzip();

        Ok((Self { hops }, ChannelPath::new(addrs, allow_loops, graph)?))
    }
}

impl Path {
    /// Creates an already pre-validated path.
    /// Used for testing only.
    pub fn new_valid(hops: Vec<PeerId>) -> Self {
        assert!(!hops.is_empty(), "must not be empty");
        Self { hops }
    }
}

impl BasePath<PeerId> for Path {
    fn hops(&self) -> &[PeerId] {
        &self.hops
    }
}

impl<T> TryFrom<&Path> for Vec<T>
where
    T: PeerIdLike,
{
    type Error = PathError;

    fn try_from(value: &Path) -> std::result::Result<Self, Self::Error> {
        value
            .hops()
            .iter()
            .map(|p| T::from_peerid(p).map_err(|_| InvalidPeer(p.to_string())))
            .collect()
    }
}

impl Display for Path {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[ {} ] ({} hops)",
            self.hops.iter().map(|p| p.to_string()).collect::<Vec<_>>().join("->"),
            self.length()
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::channel_graph::ChannelGraph;
    use crate::errors::PathError;
    use crate::path::{BasePath, Path};
    use async_trait::async_trait;
    use core_crypto::types::{OffchainPublicKey, PublicKey};
    use core_ethereum_db::{db::CoreEthereumDb, traits::HoprCoreEthereumDbActions};
    use core_types::channels::{ChannelEntry, ChannelStatus};
    use core_types::protocol::PeerAddressResolver;
    use hex_literal::hex;
    use libp2p_identity::PeerId;
    use utils_db::db::DB;
    use utils_db::rusty::RustyLevelDbShim;
    use utils_types::primitives::{Address, Balance, BalanceType, Snapshot, U256};
    use utils_types::traits::PeerIdLike;

    const PEERS_PRIVS: [[u8; 32]; 5] = [
        hex!("492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775"),
        hex!("5bf21ea8cccd69aa784346b07bf79c84dac606e00eecaa68bf8c31aff397b1ca"),
        hex!("3477d7de923ba3a7d5d72a7d6c43fd78395453532d03b2a1e2b9a7cc9b61bafa"),
        hex!("db7e3e8fcac4c817aa4cecee1d6e2b4d53da51f9881592c0e1cc303d8a012b92"),
        hex!("0726a9704d56a013980a9077d195520a61b5aed28f92d89c50bca6e0e0c48cfc"),
    ];

    #[test]
    fn test_path_validated() {
        const HOPS: usize = 5;
        let peer_ids = (0..HOPS).map(|_| PeerId::random()).collect::<Vec<_>>();

        let path = Path::new_valid(peer_ids.clone());
        assert_eq!(HOPS, path.length());
        assert_eq!(&peer_ids, path.hops());
    }

    fn create_dummy_channel(source: Address, destination: Address, status: ChannelStatus) -> ChannelEntry {
        ChannelEntry::new(
            source,
            destination,
            Balance::new(U256::from(1234 * 10000000000000000u128), BalanceType::HOPR),
            U256::zero(),
            status,
            U256::zero(),
            U256::zero(),
        )
    }

    // Channels: 0 -> 1 -> 2 -> 3 -> 4, 4 /> 0
    async fn create_db_with_channel_topology(peers: &mut Vec<PeerId>) -> CoreEthereumDb<RustyLevelDbShim> {
        let chain_key = PublicKey::from_privkey(&PEERS_PRIVS[0]).unwrap();
        let testing_snapshot = Snapshot::new(U256::zero(), U256::zero(), U256::zero());

        let mut last_addr = chain_key.to_address();
        let mut db = CoreEthereumDb::new(DB::new(RustyLevelDbShim::new_in_memory()), last_addr);

        let packet_key = OffchainPublicKey::from_privkey(&PEERS_PRIVS[0]).unwrap();
        peers.push(packet_key.to_peerid());

        db.link_chain_and_packet_keys(&chain_key.to_address(), &packet_key, &testing_snapshot)
            .await
            .unwrap();

        for peer in PEERS_PRIVS.iter().skip(1) {
            // For testing purposes only: derive both keys from the same secret key, which does not work in general
            let chain_key = PublicKey::from_privkey(peer).unwrap();
            let packet_key = OffchainPublicKey::from_privkey(peer).unwrap();

            // Link both keys
            db.link_chain_and_packet_keys(&chain_key.to_address(), &packet_key, &testing_snapshot)
                .await
                .unwrap();

            // Open channel to self
            let channel = create_dummy_channel(chain_key.to_address(), chain_key.to_address(), ChannelStatus::Open);
            db.update_channel_and_snapshot(&channel.get_id(), &channel, &testing_snapshot)
                .await
                .unwrap();

            // Open channel from last node to us
            let channel = create_dummy_channel(last_addr, chain_key.to_address(), ChannelStatus::Open);
            db.update_channel_and_snapshot(&channel.get_id(), &channel, &testing_snapshot)
                .await
                .unwrap();

            last_addr = chain_key.to_address();
            peers.push(packet_key.to_peerid());
        }

        // Add a pending to close channel between 4 -> 0
        let chain_key_0 = PublicKey::from_privkey(&PEERS_PRIVS[0]).unwrap().to_address();
        let chain_key_4 = PublicKey::from_privkey(&PEERS_PRIVS[4]).unwrap().to_address();
        let channel = create_dummy_channel(chain_key_4, chain_key_0, ChannelStatus::PendingToClose);
        db.update_channel_and_snapshot(&channel.get_id(), &channel, &testing_snapshot)
            .await
            .unwrap();

        db
    }

    struct TestResolver(CoreEthereumDb<RustyLevelDbShim>);

    #[async_trait(? Send)]
    impl PeerAddressResolver for TestResolver {
        async fn resolve_packet_key(&self, onchain_key: &Address) -> Option<OffchainPublicKey> {
            self.0.get_packet_key(onchain_key).await.ok().flatten()
        }

        async fn resolve_chain_key(&self, offchain_key: &OffchainPublicKey) -> Option<Address> {
            self.0.get_chain_key(offchain_key).await.ok().flatten()
        }

        async fn link_keys(&mut self, _onchain_key: &Address, _offchain_key: &OffchainPublicKey) -> bool {
            panic!("should not be called in tests")
        }
    }

    #[async_std::test]
    async fn test_path_validation() {
        let mut peers = Vec::new();

        let me = PublicKey::from_privkey(&PEERS_PRIVS[0]).unwrap().to_address();
        let mut cg = ChannelGraph::new(me);

        let db = create_db_with_channel_topology(&mut peers).await;
        cg.sync_channels(&db).await.expect("failed to sync graph with the DB");
        let resolver = TestResolver(db);

        let (path, cpath) = Path::resolve(vec![peers[1], peers[2]], false, &resolver, &cg)
            .await
            .expect("path 0 -> 1 -> 2 must be valid");

        assert_eq!(2, path.length());
        assert!(!path.has_simple_loops(), "path must not have loops");
        assert!(!cpath.has_simple_loops(), "channel path must not have loops");
        assert_eq!(cpath.length(), path.length(), "length must be equal");

        let path_2 = cpath.to_path(&resolver).await.expect("must be reverse-resolvable");
        assert_eq!(path, path_2, "must be equal");

        let (path, _) = Path::resolve(vec![peers[2]], false, &resolver, &cg)
            .await
            .expect("path 0 -> 2 must be valid, because channel not needed for direct message");

        assert_eq!(1, path.length());

        let (path, _) = Path::resolve(vec![peers[1], peers[2], peers[3]], false, &resolver, &cg)
            .await
            .expect("path 0 -> 1 -> 2 -> 3 must be valid");

        assert_eq!(3, path.length());
        assert!(!path.has_simple_loops(), "must not have loops");

        let (path, _) = Path::resolve(vec![peers[1], peers[2], peers[3], peers[4]], false, &resolver, &cg)
            .await
            .expect("path 0 -> 1 -> 2 -> 3 -> 4 must be valid");

        assert_eq!(4, path.length());
        assert!(!path.has_simple_loops(), "must not have loops");

        let (path, cpath) = Path::resolve(vec![peers[1], peers[2], peers[2], peers[3]], true, &resolver, &cg)
            .await
            .expect("path 0 -> 1 -> 2 -> 2 -> 3 must be valid if loops are allowed");

        assert_eq!(4, path.length()); // loop still counts as a hop
        assert!(path.has_simple_loops(), "must have loops");
        assert!(path.is_cyclic(), "must be cyclic");
        assert!(cpath.has_simple_loops(), "must have loops");
        assert!(cpath.is_cyclic(), "must be cyclic");

        match Path::resolve(vec![peers[1], peers[2], peers[2], peers[3]], false, &resolver, &cg)
            .await
            .expect_err("path 0 -> 1 -> 2 -> 2 must be invalid if loops are not allowed")
        {
            PathError::LoopsNotAllowed(_) => {}
            _ => panic!("error must be LoopsNotAllowed"),
        };

        match Path::resolve(vec![peers[3], peers[4]], false, &resolver, &cg)
            .await
            .expect_err("path 0 -> 3 must be invalid, because channel 0 -> 3 is not opened")
        {
            PathError::MissingChannel(_, _) => {}
            _ => panic!("error must be MissingChannel"),
        };

        match Path::resolve(vec![peers[1], peers[3], peers[4]], false, &resolver, &cg)
            .await
            .expect_err("path 0 -> 1 -> 3 -> 4 must be invalid, because channel 1 -> 3 is not opened")
        {
            PathError::MissingChannel(_, _) => {}
            _ => panic!("error must be MissingChannel"),
        };

        match Path::resolve(
            vec![peers[1], peers[2], peers[3], peers[4], peers[0], peers[1]],
            false,
            &resolver,
            &cg,
        )
        .await
        .expect_err("path 0 -> 1 -> 2 -> 3 -> 4 -> 0 -> 1 must be invalid, because channel 4 -> 0 is already closed")
        {
            PathError::ChannelNotOpened(_, _) => {}
            _ => panic!("error must be ChannelNotOpened"),
        };

        let me = PublicKey::from_privkey(&PEERS_PRIVS[4]).unwrap().to_address();
        let mut cg = ChannelGraph::new(me);

        let db = create_db_with_channel_topology(&mut peers).await;
        cg.sync_channels(&db).await.expect("failed to sync graph with the DB");
        let resolver = TestResolver(db);

        match Path::resolve(vec![peers[0], peers[1]], false, &resolver, &cg)
            .await
            .expect_err("path 4 -> 0 -> 1 must be invalid, because channel 4 -> 0 is already closed")
        {
            PathError::ChannelNotOpened(_, _) => {}
            _ => panic!("error must be ChannelNotOpened"),
        };
    }
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use crate::channel_graph::wasm::ChannelGraph;
    use crate::errors::{PathError::InvalidPeer, Result};
    use crate::path::{BasePath, Path};
    use async_trait::async_trait;
    use core_crypto::types::OffchainPublicKey;
    use core_ethereum_db::db::wasm::Database;
    use core_ethereum_db::traits::HoprCoreEthereumDbActions;
    use core_types::protocol::PeerAddressResolver;
    use js_sys::JsString;
    use libp2p_identity::PeerId;
    use std::str::FromStr;
    use utils_misc::utils::wasm::JsResult;
    use utils_types::primitives::Address;
    use wasm_bindgen::prelude::wasm_bindgen;

    pub struct PathResolver<'a, Db: HoprCoreEthereumDbActions>(pub &'a Db);

    #[async_trait(? Send)]
    impl<Db: HoprCoreEthereumDbActions> PeerAddressResolver for PathResolver<'_, Db> {
        async fn resolve_packet_key(&self, onchain_key: &Address) -> Option<OffchainPublicKey> {
            self.0.get_packet_key(onchain_key).await.ok().flatten()
        }

        async fn resolve_chain_key(&self, offchain_key: &OffchainPublicKey) -> Option<Address> {
            self.0.get_chain_key(offchain_key).await.ok().flatten()
        }

        async fn link_keys(&mut self, _onchain_key: &Address, _offchain_key: &OffchainPublicKey) -> bool {
            unimplemented!()
        }
    }

    #[wasm_bindgen]
    impl Path {
        #[wasm_bindgen(js_name = "validated")]
        pub async fn _validated(
            path: Vec<JsString>,
            allow_loops: bool,
            db: &Database,
            channel_graph: &ChannelGraph,
        ) -> JsResult<Path> {
            let database = db.as_ref_counted();
            let graph = channel_graph.as_ref_counted();
            let g = database.read().await;
            let cg = graph.read().await;
            Ok(Path::resolve(
                path.into_iter()
                    .map(|p| PeerId::from_str(&p.as_string().unwrap()).map_err(|_| InvalidPeer(p.as_string().unwrap())))
                    .collect::<Result<Vec<PeerId>>>()?,
                allow_loops,
                &PathResolver(&*g),
                &*cg,
            )
            .await
            .map(|(p, _)| p)?)
        }

        #[wasm_bindgen(js_name = "length")]
        pub fn _length(&self) -> u32 {
            self.length() as u32
        }

        #[wasm_bindgen(js_name = "to_string")]
        pub fn _to_string(&self) -> String {
            self.to_string()
        }
    }
}
