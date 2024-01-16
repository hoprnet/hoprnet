use crate::channel_graph::ChannelGraph;
use crate::errors::PathError;
use crate::errors::PathError::{ChannelNotOpened, InvalidPeer, LoopsNotAllowed, MissingChannel, PathNotValid};
use crate::errors::Result;
use futures::future::FutureExt;
use futures::stream::FuturesOrdered;
use futures::TryStreamExt;
use hopr_crypto_types::types::OffchainPublicKey;
use hopr_internal_types::channels::ChannelStatus;
use hopr_internal_types::protocol::PeerAddressResolver;
use hopr_primitive_types::primitives::Address;
use hopr_primitive_types::traits::ToHex;
use libp2p_identity::PeerId;
use std::collections::hash_map::RandomState;
use std::collections::HashSet;
use std::fmt::{Display, Formatter};
use std::hash::Hash;

/// Base implementation of an abstract path.
/// Must contain always at least a single entry.
pub trait Path<N>: Display + Clone + Eq + PartialEq
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

    /// Checks if all the hops in this path are to distinct addresses.
    /// Returns `true` if there are duplicate Addresses on this path.
    /// Note that the duplicate Addresses can never be adjacent.
    fn contains_cycle(&self) -> bool {
        let set = HashSet::<&N, RandomState>::from_iter(self.hops().iter());
        set.len() != self.hops().len()
    }
}

/// Represents an on-chain path in the `ChannelGraph`.
/// This path is never allowed to be empty and is always constructed from
/// `Addresses` that must be known to have open channels between them (at the time of construction).
/// This path is not useful for transport, because it *does never contain the last hop*
/// to the destination (which does not require and open channel).
/// To make it useful for transport, it must be converted to a `TransportPath` via `to_path`.
/// The `ChannelPath` does not allow simple loops (same adjacent hops)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChannelPath {
    pub(crate) hops: Vec<Address>,
}

impl ChannelPath {
    /// Creates a new path by validating the list of addresses using the channel graph.
    /// The given list of `hops` *must not* contain the sender node as the first entry,
    /// since this node is always assumed to be the sender.
    /// The list of `hops` also *must not* contain the destination, because an open
    /// channel is not required for the last hop.
    pub fn new(hops: Vec<Address>, graph: &ChannelGraph) -> Result<Self> {
        if hops.is_empty() || hops[0] == graph.my_address() {
            return Err(PathNotValid);
        }

        let mut ticket_receiver;
        let mut ticket_issuer = graph.my_address();

        for hop in hops.iter() {
            ticket_receiver = *hop;

            // Check for loops
            if ticket_issuer == ticket_receiver {
                return Err(LoopsNotAllowed(ticket_receiver.to_hex()));
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

    /// For internal use and testing only
    pub(crate) fn new_valid(hops: Vec<Address>) -> Self {
        assert!(!hops.is_empty(), "must not be empty");
        Self { hops }
    }

    /// Resolves this on-chain `ChannelPath` into the off-chain `TransportPath` and adds the final hop
    /// to the given `destination` (which does not require an open channel).
    pub async fn to_path<R: PeerAddressResolver>(&self, resolver: &R, destination: Address) -> Result<TransportPath> {
        let mut hops = self
            .hops
            .iter()
            .map(|addr| {
                resolver
                    .resolve_packet_key(addr)
                    .map(move |opt| opt.map(|k| PeerId::from(k.clone())).ok_or(InvalidPeer(addr.to_string())))
            })
            .collect::<FuturesOrdered<_>>()
            .try_collect::<Vec<PeerId>>()
            .await?;

        let last_hop = resolver
            .resolve_packet_key(&destination)
            .await
            .ok_or(InvalidPeer(destination.to_string()))?;

        hops.push(last_hop.into());
        Ok(TransportPath::new_valid(hops))
    }
}

impl Path<Address> for ChannelPath {
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
/// The path is never allowed to be empty and *always contains the destination*.
/// In case of the direct path, this path contains only the destination.
/// In case o multiple hops, it also must represent a valid `ChannelPath`, therefore
/// open channels must exist (at the time of construction) except for the last hop.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransportPath {
    hops: Vec<PeerId>,
}

impl TransportPath {
    /// Resolves vector of `PeerId`s into the corresponding `TransportPath` and optionally an associated `ChannelPath`.
    /// - If `peers` contains only a single entry (destination), the resulting `TransportPath` contains just this entry.
    /// Since in this case the `ChannelPath` would be empty (0-hop), it is `None`. This case is equivalent to construction with `direct()`.
    /// - If `peers` contains more than a single entry, it first resolves `PeerId`s into `Address`es and then validates and returns
    ///  also the multi-hop `ChannelPath`.
    /// To do an inverse resolution, from `Address`es to `PeerId`s, construct the `ChannelPath` and use `ChannelPath::to_path()` to resolve the
    /// on-chain path.
    pub async fn resolve<R: PeerAddressResolver>(
        peers: Vec<PeerId>,
        resolver: &R,
        graph: &ChannelGraph,
    ) -> Result<(Self, Option<ChannelPath>)> {
        if peers.is_empty() {
            Err(PathNotValid)
        } else if peers.len() == 1 {
            Ok((Self { hops: peers }, None))
        } else {
            let (mut addrs, hops): (Vec<Address>, Vec<PeerId>) = peers
                .into_iter()
                .map(|peer| async move {
                    let key = OffchainPublicKey::try_from(peer)?;
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

            addrs.pop(); // remove the last hop
            Ok((Self { hops }, Some(ChannelPath::new(addrs, graph)?)))
        }
    }

    /// Constructs a direct `TransportPath` (= 0-hop `ChannelPath`)
    pub fn direct(destination: PeerId) -> Self {
        Self {
            hops: vec![destination],
        }
    }

    /// Used for testing only.
    pub(crate) fn new_valid(hops: Vec<PeerId>) -> Self {
        assert!(!hops.is_empty(), "must not be empty");
        Self { hops }
    }
}

impl Path<PeerId> for TransportPath {
    /// The `TransportPath` always returns one extra hop to the destination.
    /// So it contains one more hop than a `ChannelPath` (the final hop does not require a channel).
    fn hops(&self) -> &[PeerId] {
        &self.hops
    }
}

impl<T> TryFrom<&TransportPath> for Vec<T>
where
    T: TryFrom<PeerId>,
{
    type Error = PathError;

    fn try_from(value: &TransportPath) -> std::result::Result<Self, Self::Error> {
        value
            .hops()
            .iter()
            .map(|p| T::try_from(*p).map_err(|_| InvalidPeer(p.to_string())))
            .collect()
    }
}

impl Display for TransportPath {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[ {} ] ({} hops)",
            self.hops.iter().map(|p| p.to_string()).collect::<Vec<_>>().join("->"),
            self.length() - 1
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use hex_literal::hex;
    use hopr_crypto_types::prelude::*;
    use hopr_internal_types::channels::ChannelEntry;
    use hopr_primitive_types::prelude::*;

    const PEERS_PRIVS: [[u8; 32]; 5] = [
        hex!("492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775"),
        hex!("5bf21ea8cccd69aa784346b07bf79c84dac606e00eecaa68bf8c31aff397b1ca"),
        hex!("3477d7de923ba3a7d5d72a7d6c43fd78395453532d03b2a1e2b9a7cc9b61bafa"),
        hex!("db7e3e8fcac4c817aa4cecee1d6e2b4d53da51f9881592c0e1cc303d8a012b92"),
        hex!("0726a9704d56a013980a9077d195520a61b5aed28f92d89c50bca6e0e0c48cfc"),
    ];

    fn dummy_channel(src: Address, dst: Address, status: ChannelStatus) -> ChannelEntry {
        ChannelEntry::new(
            src,
            dst,
            Balance::new_from_str("1", BalanceType::HOPR),
            1u32.into(),
            status,
            1u32.into(),
            0u32.into(),
        )
    }

    fn create_graph_and_resolver_entries(me: Address) -> (ChannelGraph, Vec<(OffchainPublicKey, Address)>) {
        let mut cg = ChannelGraph::new(me);
        let addrs = PEERS_PRIVS
            .iter()
            .map(|p| {
                (
                    OffchainPublicKey::from_privkey(p).unwrap(),
                    PublicKey::from_privkey(p).unwrap().to_address(),
                )
            })
            .collect::<Vec<_>>();

        // Channels: 0 -> 1 -> 2 -> 3 -> 4, 4 /> 0, 3 -> 1
        cg.update_channel(dummy_channel(addrs[0].1, addrs[1].1, ChannelStatus::Open));
        cg.update_channel(dummy_channel(addrs[1].1, addrs[2].1, ChannelStatus::Open));
        cg.update_channel(dummy_channel(addrs[2].1, addrs[3].1, ChannelStatus::Open));
        cg.update_channel(dummy_channel(addrs[3].1, addrs[4].1, ChannelStatus::Open));
        cg.update_channel(dummy_channel(addrs[3].1, addrs[1].1, ChannelStatus::Open));
        cg.update_channel(dummy_channel(addrs[4].1, addrs[0].1, ChannelStatus::PendingToClose));

        (cg, addrs)
    }

    #[test]
    fn test_channel_path_zero_hop_should_fail() {
        let me = PublicKey::from_privkey(&PEERS_PRIVS[0]).unwrap().to_address();
        let (cg, _) = create_graph_and_resolver_entries(me);

        ChannelPath::new(vec![], &cg).expect_err("path must not be constructible");
    }

    #[test]
    fn test_channel_path_one_hop() {
        let me = PublicKey::from_privkey(&PEERS_PRIVS[0]).unwrap().to_address();
        let (cg, peer_addrs) = create_graph_and_resolver_entries(me);
        let (_, addrs): (Vec<OffchainPublicKey>, Vec<Address>) = peer_addrs.into_iter().unzip();

        // path: 0 -> 1
        let cp = ChannelPath::new(vec![addrs[1]], &cg).expect("path must be constructible");
        assert_eq!(1, cp.length(), "must be one hop");
        assert!(!cp.contains_cycle(), "must not be cyclic");
    }

    #[test]
    fn test_channel_path_two_hop() {
        let me = PublicKey::from_privkey(&PEERS_PRIVS[0]).unwrap().to_address();
        let (cg, peer_addrs) = create_graph_and_resolver_entries(me);
        let (_, addrs): (Vec<OffchainPublicKey>, Vec<Address>) = peer_addrs.into_iter().unzip();

        // path: 0 -> 1 -> 2
        let cp = ChannelPath::new(vec![addrs[1], addrs[2]], &cg).expect("path must be constructible");
        assert_eq!(2, cp.length(), "must be two hop");
        assert!(!cp.contains_cycle(), "must not be cyclic");
    }

    #[test]
    fn test_channel_path_three_hop() {
        let me = PublicKey::from_privkey(&PEERS_PRIVS[0]).unwrap().to_address();
        let (cg, peer_addrs) = create_graph_and_resolver_entries(me);
        let (_, addrs): (Vec<OffchainPublicKey>, Vec<Address>) = peer_addrs.into_iter().unzip();

        // path: 0 -> 1 -> 2 -> 3
        let cp = ChannelPath::new(vec![addrs[1], addrs[2], addrs[3]], &cg).expect("path must be constructible");
        assert_eq!(3, cp.length(), "must be three hop");
        assert!(!cp.contains_cycle(), "must not be cyclic");
    }

    #[test]
    fn test_channel_path_must_allow_cyclic() {
        let me = PublicKey::from_privkey(&PEERS_PRIVS[0]).unwrap().to_address();
        let (cg, peer_addrs) = create_graph_and_resolver_entries(me);
        let (_, addrs): (Vec<OffchainPublicKey>, Vec<Address>) = peer_addrs.into_iter().unzip();

        // path: 0 -> 1 -> 2 -> 3 -> 1
        let cp =
            ChannelPath::new(vec![addrs[1], addrs[2], addrs[3], addrs[1]], &cg).expect("path must be constructible");
        assert_eq!(4, cp.length(), "must be four hop");
        assert!(cp.contains_cycle(), "must not be cyclic");
    }

    #[test]
    fn test_channel_path_should_fail_for_non_open_channel() {
        let me = PublicKey::from_privkey(&PEERS_PRIVS[0]).unwrap().to_address();
        let (cg, peer_addrs) = create_graph_and_resolver_entries(me);
        let (_, addrs): (Vec<OffchainPublicKey>, Vec<Address>) = peer_addrs.into_iter().unzip();

        // path: 0 -> 4 -> 0 (channel 4 -> 0 is PendingToClose)
        ChannelPath::new(vec![addrs[4], addrs[0]], &cg).expect_err("path must not be constructible");
    }

    #[test]
    fn test_channel_path_should_fail_for_non_open_channel_from_self() {
        let me = PublicKey::from_privkey(&PEERS_PRIVS[4]).unwrap().to_address();
        let (cg, peer_addrs) = create_graph_and_resolver_entries(me);
        let (_, addrs): (Vec<OffchainPublicKey>, Vec<Address>) = peer_addrs.into_iter().unzip();

        // path: 4 -> 0
        ChannelPath::new(vec![addrs[4], addrs[0]], &cg).expect_err("path must not be constructible");
    }

    #[test]
    fn test_channel_path_should_fail_for_non_existing_channel() {
        let me = PublicKey::from_privkey(&PEERS_PRIVS[0]).unwrap().to_address();
        let (cg, peer_addrs) = create_graph_and_resolver_entries(me);
        let (_, addrs): (Vec<OffchainPublicKey>, Vec<Address>) = peer_addrs.into_iter().unzip();

        // path: 0 -> 1 -> 3 (channel 1 -> 3 does not exist)
        ChannelPath::new(vec![addrs[1], addrs[3]], &cg).expect_err("path 1 must not be constructible");

        // path: 0 -> 1 -> 2 -> 4 (channel 2 -> 4 does not exist)
        ChannelPath::new(vec![addrs[1], addrs[2], addrs[4]], &cg).expect_err("path 2 must not be constructible");
    }

    #[test]
    fn test_channel_path_should_not_allow_loops() {
        let me = PublicKey::from_privkey(&PEERS_PRIVS[0]).unwrap().to_address();
        let (cg, peer_addrs) = create_graph_and_resolver_entries(me);
        let (_, addrs): (Vec<OffchainPublicKey>, Vec<Address>) = peer_addrs.into_iter().unzip();

        // path: 0 -> 1 -> 1 -> 2
        ChannelPath::new(vec![addrs[1], addrs[1], addrs[0]], &cg).expect_err("path must not be constructible");
    }

    struct TestResolver(Vec<(OffchainPublicKey, Address)>);

    #[async_trait]
    impl PeerAddressResolver for TestResolver {
        async fn resolve_packet_key(&self, onchain_key: &Address) -> Option<OffchainPublicKey> {
            self.0
                .iter()
                .find(|(_, addr)| addr.eq(onchain_key))
                .map(|(pk, _)| pk.clone())
        }

        async fn resolve_chain_key(&self, offchain_key: &OffchainPublicKey) -> Option<Address> {
            self.0.iter().find(|(pk, _)| pk.eq(offchain_key)).map(|(_, addr)| *addr)
        }
    }

    #[async_std::test]
    async fn test_transport_path_empty_should_fail() {
        let me = PublicKey::from_privkey(&PEERS_PRIVS[0]).unwrap().to_address();
        let (cg, peer_addrs) = create_graph_and_resolver_entries(me);
        let resolver = TestResolver(peer_addrs.clone());

        TransportPath::resolve(vec![], &resolver, &cg)
            .await
            .expect_err("should not resolve path");
    }

    fn make_address_pairs(peer_addrs: Vec<(OffchainPublicKey, Address)>) -> (Vec<PeerId>, Vec<Address>) {
        peer_addrs.into_iter().map(|(p, a)| (PeerId::from(p), a)).unzip()
    }

    #[async_std::test]
    async fn test_transport_path_resolve_direct() {
        let me = PublicKey::from_privkey(&PEERS_PRIVS[0]).unwrap().to_address();
        let (cg, peer_addrs) = create_graph_and_resolver_entries(me);
        let resolver = TestResolver(peer_addrs.clone());
        let (peers, _) = make_address_pairs(peer_addrs);

        // path 0 -> 1
        let (p, cp) = TransportPath::resolve(vec![peers[1]], &resolver, &cg)
            .await
            .expect("should resolve path");
        assert_eq!(1, p.length(), "must contain destination");
        assert!(cp.is_none(), "no channel path for direct message")
    }

    #[async_std::test]
    async fn test_transport_path_resolve_direct_to_self() {
        let me = PublicKey::from_privkey(&PEERS_PRIVS[0]).unwrap().to_address();
        let (cg, peer_addrs) = create_graph_and_resolver_entries(me);
        let resolver = TestResolver(peer_addrs.clone());
        let (peers, _) = make_address_pairs(peer_addrs);

        // path 0 -> 0
        let (p, cp) = TransportPath::resolve(vec![peers[0]], &resolver, &cg)
            .await
            .expect("should resolve path");
        assert_eq!(1, p.length(), "must contain destination");
        assert!(cp.is_none(), "no channel path for direct message")
    }

    #[async_std::test]
    async fn test_transport_path_resolve_direct_is_allowed_without_channel_to_destination() {
        let me = PublicKey::from_privkey(&PEERS_PRIVS[0]).unwrap().to_address();
        let (cg, peer_addrs) = create_graph_and_resolver_entries(me);
        let resolver = TestResolver(peer_addrs.clone());
        let (peers, _) = make_address_pairs(peer_addrs);

        // path 0 -> 3
        let (p, cp) = TransportPath::resolve(vec![peers[3]], &resolver, &cg)
            .await
            .expect("should resolve path");
        assert_eq!(1, p.length(), "must contain destination");
        assert!(cp.is_none(), "no channel path for direct message");
    }

    #[async_std::test]
    async fn test_transport_path_resolve_direct_is_allowed_with_closed_channel_to_destination() {
        let me = PublicKey::from_privkey(&PEERS_PRIVS[4]).unwrap().to_address();
        let (cg, peer_addrs) = create_graph_and_resolver_entries(me);
        let resolver = TestResolver(peer_addrs.clone());
        let (peers, _) = make_address_pairs(peer_addrs);

        // path 4 -> 0
        let (p, cp) = TransportPath::resolve(vec![peers[0]], &resolver, &cg)
            .await
            .expect("should resolve path");
        assert_eq!(1, p.length(), "must contain destination");
        assert!(cp.is_none(), "no channel path for direct message")
    }

    #[async_std::test]
    async fn test_transport_path_resolve_one_hop() {
        let me = PublicKey::from_privkey(&PEERS_PRIVS[0]).unwrap().to_address();
        let (cg, peer_addrs) = create_graph_and_resolver_entries(me);
        let resolver = TestResolver(peer_addrs.clone());
        let (peers, addrs) = make_address_pairs(peer_addrs);

        // path 0 -> 1 -> 2
        let (p, cp) = TransportPath::resolve(vec![peers[1], peers[2]], &resolver, &cg)
            .await
            .expect("should resolve path");
        assert_eq!(2, p.length(), "must be two hop");
        assert!(!p.contains_cycle(), "transport path must not contain a cycle");

        let cp = cp.expect("must have channel path");
        assert_eq!(1, cp.length(), "channel path must be one hop");
        assert_eq!(vec![addrs[1]], cp.hops(), "channel path address must match");
        assert!(!cp.contains_cycle(), "channel path must not contain a cycle");
    }

    #[async_std::test]
    async fn test_transport_path_resolve_one_hop_without_channel_to_destination() {
        let me = PublicKey::from_privkey(&PEERS_PRIVS[0]).unwrap().to_address();
        let (cg, peer_addrs) = create_graph_and_resolver_entries(me);
        let resolver = TestResolver(peer_addrs.clone());
        let (peers, addrs) = make_address_pairs(peer_addrs);

        // path 0 -> 1 -> 4
        let (p, cp) = TransportPath::resolve(vec![peers[1], peers[4]], &resolver, &cg)
            .await
            .expect("should resolve path");
        assert_eq!(2, p.length(), "must be two hop");
        assert!(!p.contains_cycle(), "transport path must not contain a cycle");

        let cp = cp.expect("must have channel path");
        assert_eq!(1, cp.length(), "channel path must be one hop");
        assert_eq!(vec![addrs[1]], cp.hops(), "channel path address must match");
        assert!(!cp.contains_cycle(), "channel path must not contain a cycle");
    }

    #[async_std::test]
    async fn test_transport_path_resolve_two_hop() {
        let me = PublicKey::from_privkey(&PEERS_PRIVS[0]).unwrap().to_address();
        let (cg, peer_addrs) = create_graph_and_resolver_entries(me);
        let resolver = TestResolver(peer_addrs.clone());
        let (peers, addrs) = make_address_pairs(peer_addrs);

        // path 0 -> 1 -> 2 -> 3
        let (p, cp) = TransportPath::resolve(vec![peers[1], peers[2], peers[3]], &resolver, &cg)
            .await
            .expect("should resolve path");
        assert_eq!(3, p.length(), "must be three hop");
        assert!(!p.contains_cycle(), "transport path must not contain a cycle");

        let cp = cp.expect("must have channel path");
        assert_eq!(2, cp.length(), "channel path must be two hop");
        assert_eq!(vec![addrs[1], addrs[2]], cp.hops(), "channel path address must match");
        assert!(!cp.contains_cycle(), "channel path must not contain a cycle");
    }

    #[async_std::test]
    async fn test_transport_path_resolve_two_hop_without_channel_to_destination() {
        let me = PublicKey::from_privkey(&PEERS_PRIVS[0]).unwrap().to_address();
        let (cg, peer_addrs) = create_graph_and_resolver_entries(me);
        let resolver = TestResolver(peer_addrs.clone());
        let (peers, addrs) = make_address_pairs(peer_addrs);

        // path 0 -> 1 -> 2 -> 4
        let (p, cp) = TransportPath::resolve(vec![peers[1], peers[2], peers[4]], &resolver, &cg)
            .await
            .expect("should resolve path");
        assert_eq!(3, p.length(), "must be three hop");
        assert!(!p.contains_cycle(), "transport path must not contain a cycle");

        let cp = cp.expect("must have channel path");
        assert_eq!(2, cp.length(), "channel path must be two hop");
        assert_eq!(vec![addrs[1], addrs[2]], cp.hops(), "channel path address must match");
        assert!(!cp.contains_cycle(), "channel path must not contain a cycle");
    }

    #[async_std::test]
    async fn test_transport_path_resolve_three_hop() {
        let me = PublicKey::from_privkey(&PEERS_PRIVS[0]).unwrap().to_address();
        let (cg, peer_addrs) = create_graph_and_resolver_entries(me);
        let resolver = TestResolver(peer_addrs.clone());
        let (peers, addrs) = make_address_pairs(peer_addrs);

        // path 0 -> 1 -> 2 -> 3 -> 4
        let (p, cp) = TransportPath::resolve(vec![peers[1], peers[2], peers[3], peers[4]], &resolver, &cg)
            .await
            .expect("should resolve path");
        assert_eq!(4, p.length(), "must have 4 entries");
        assert!(!p.contains_cycle(), "transport path must not contain a cycle");

        let cp = cp.expect("must have channel path");
        assert_eq!(3, cp.length(), "channel path must be two hop");
        assert_eq!(
            vec![addrs[1], addrs[2], addrs[3]],
            cp.hops(),
            "channel path address must match"
        );
        assert!(!cp.contains_cycle(), "channel path must not contain a cycle");
    }

    #[async_std::test]
    async fn test_transport_path_resolve_with_cycle() {
        let me = PublicKey::from_privkey(&PEERS_PRIVS[0]).unwrap().to_address();
        let (cg, peer_addrs) = create_graph_and_resolver_entries(me);
        let resolver = TestResolver(peer_addrs.clone());
        let (peers, addrs) = make_address_pairs(peer_addrs);

        // path 0 -> 1 -> 2 -> 3 -> 1
        let (p, cp) = TransportPath::resolve(vec![peers[1], peers[2], peers[3], peers[1]], &resolver, &cg)
            .await
            .expect("should resolve path");
        assert_eq!(4, p.length(), "must have 4 entries");
        assert!(p.contains_cycle(), "transport path must contain a cycle");

        let cp = cp.expect("must have channel path");
        assert_eq!(3, cp.length(), "channel path must be 3 hop");
        assert_eq!(
            vec![addrs[1], addrs[2], addrs[3]],
            cp.hops(),
            "channel path address must match"
        );
        assert!(!cp.contains_cycle(), "channel path must not contain a cycle");
    }

    #[async_std::test]
    async fn test_transport_path_resolve_with_channel_cycle() {
        let me = PublicKey::from_privkey(&PEERS_PRIVS[0]).unwrap().to_address();
        let (cg, peer_addrs) = create_graph_and_resolver_entries(me);
        let resolver = TestResolver(peer_addrs.clone());
        let (peers, addrs) = make_address_pairs(peer_addrs);

        // path 0 -> 1 -> 2 -> 3 -> 1 -> 2
        let (p, cp) = TransportPath::resolve(vec![peers[1], peers[2], peers[3], peers[1], peers[2]], &resolver, &cg)
            .await
            .expect("should resolve path");
        assert_eq!(5, p.length(), "must be 5 hop");
        assert!(p.contains_cycle(), "transport path must contain a cycle");

        let cp = cp.expect("must have channel path");
        assert_eq!(4, cp.length(), "channel path must be 4 hop");
        assert_eq!(
            vec![addrs[1], addrs[2], addrs[3], addrs[1]],
            cp.hops(),
            "channel path address must match"
        );
        assert!(cp.contains_cycle(), "channel path must not contain a cycle");
    }

    #[async_std::test]
    async fn test_transport_path_should_not_resolve_for_non_existing_intermediate_channel() {
        let me = PublicKey::from_privkey(&PEERS_PRIVS[0]).unwrap().to_address();
        let (cg, peer_addrs) = create_graph_and_resolver_entries(me);
        let resolver = TestResolver(peer_addrs.clone());
        let (peers, _) = make_address_pairs(peer_addrs);

        // path 0 -> 2 -> 3
        TransportPath::resolve(vec![peers[2], peers[3]], &resolver, &cg)
            .await
            .expect_err("should not resolve path 1");

        // path 0 -> 1 -> 3 -> 1
        TransportPath::resolve(vec![peers[1], peers[3], peers[1]], &resolver, &cg)
            .await
            .expect_err("should not resolve path 2");
    }

    #[async_std::test]
    async fn test_transport_path_should_not_resolve_for_non_open_intermediate_channel() {
        let me = PublicKey::from_privkey(&PEERS_PRIVS[2]).unwrap().to_address();
        let (cg, peer_addrs) = create_graph_and_resolver_entries(me);
        let resolver = TestResolver(peer_addrs.clone());
        let (peers, _) = make_address_pairs(peer_addrs);

        // path 2 -> 3 -> 4 -> 0 -> 1
        TransportPath::resolve(vec![peers[3], peers[4], peers[0], peers[1]], &resolver, &cg)
            .await
            .expect_err("should not resolve path");
    }

    #[async_std::test]
    async fn test_channel_path_to_transport_path() {
        let me = PublicKey::from_privkey(&PEERS_PRIVS[0]).unwrap().to_address();
        let (cg, peer_addrs) = create_graph_and_resolver_entries(me);
        let resolver = TestResolver(peer_addrs.clone());
        let (peers, addrs) = make_address_pairs(peer_addrs);

        // path: 0 -> 1 -> 2 -> 3
        let cp = ChannelPath::new(vec![addrs[1], addrs[2], addrs[3]], &cg).expect("path must be constructible");

        // path: 0 -> 1 -> 2 -> 3 -> 4
        let tp = cp
            .to_path(&resolver, addrs[4])
            .await
            .expect("should convert to transport path");
        assert_eq!(
            tp.length(),
            cp.length() + 1,
            "transport path must have extra hop to destination"
        );
        assert_eq!(
            vec![peers[1], peers[2], peers[3], peers[4]],
            tp.hops(),
            "must contain all peer ids"
        );
    }
}
