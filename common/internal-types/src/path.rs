use std::{
    fmt::{Display, Formatter},
    ops::Deref,
};

use hopr_crypto_types::prelude::*;
use hopr_primitive_types::prelude::*;

use crate::{
    NodeId,
    channels::{ChannelEntry, ChannelStatus},
    errors::{
        PathError,
        PathError::{ChannelNotOpened, InvalidPeer, LoopsNotAllowed, MissingChannel, PathNotValid},
    },
    protocol::INTERMEDIATE_HOPS,
};

pub(crate) type PathAddress = NodeId;

/// Base implementation of an abstract path.
pub trait Path<N>: Clone + Eq + PartialEq + Deref<Target = [N]> + IntoIterator<Item = N>
where
    N: Into<PathAddress> + Copy,
{
    /// Individual hops in the path.
    /// There must be always at least one hop.
    fn hops(&self) -> &[N] {
        self.deref()
    }

    /// Shorthand for the number of hops.
    fn num_hops(&self) -> usize {
        self.hops().len()
    }

    /// Returns the path with the hops in reverse order if it is possible.
    fn invert(self) -> Option<Self>;

    /// Checks if the path contains some entry twice.
    fn contains_cycle(&self) -> bool {
        std::collections::HashSet::<_, std::hash::RandomState>::from_iter(self.iter().copied().map(|h| h.into())).len()
            != self.num_hops()
    }
}

/// A [`Path`] that is guaranteed to have at least one hop - the destination.
pub trait NonEmptyPath<N: Into<PathAddress> + Copy>: Path<N> {
    /// Gets the last hop (destination)
    fn last_hop(&self) -> &N {
        self.hops().last().expect("non-empty path must have at least one hop")
    }
}

impl<T: Into<PathAddress> + Copy + PartialEq + Eq> Path<T> for Vec<T> {
    fn invert(self) -> Option<Self> {
        Some(self.into_iter().rev().collect())
    }
}

pub type ChannelPath = Vec<Address>;

/// A [`NonEmptyPath`] that can be used to route packets using [`OffchainPublicKey`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransportPath(Vec<OffchainPublicKey>);

impl TransportPath {
    /// Creates a new instance from the given iterator.
    ///
    /// Fails if the iterator is empty.
    pub fn new<T, I>(path: I) -> Result<Self, PathError>
    where
        T: Into<OffchainPublicKey>,
        I: IntoIterator<Item = T>,
    {
        let hops = path.into_iter().map(|t| t.into()).collect::<Vec<_>>();
        if !hops.is_empty() {
            Ok(Self(hops))
        } else {
            Err(PathNotValid)
        }
    }

    /// Creates a direct path just to the `destination`.
    pub fn direct(destination: OffchainPublicKey) -> Self {
        Self(vec![destination])
    }
}

impl Deref for TransportPath {
    type Target = [OffchainPublicKey];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl IntoIterator for TransportPath {
    type IntoIter = std::vec::IntoIter<Self::Item>;
    type Item = OffchainPublicKey;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl Path<OffchainPublicKey> for TransportPath {
    fn invert(self) -> Option<Self> {
        Some(Self(self.0.into_iter().rev().collect()))
    }
}

impl NonEmptyPath<OffchainPublicKey> for TransportPath {}

/// Represents a [`NonEmptyPath`] that completely specifies a route using [`Addresses`](Address).
///
/// Transport cannot directly use this to deliver packets.
///
/// Note that this is different from [`ChannelPath`], which can be empty and does not contain
/// the address of the destination.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChainPath(Vec<Address>);

impl ChainPath {
    /// Creates a new instance from the given iterator.
    ///
    /// Fails if the iterator is empty.
    pub fn new<T, I>(path: I) -> Result<Self, PathError>
    where
        T: Into<Address>,
        I: IntoIterator<Item = T>,
    {
        let hops = path.into_iter().map(|t| t.into()).collect::<Vec<_>>();
        if !hops.is_empty() {
            Ok(Self(hops))
        } else {
            Err(PathNotValid)
        }
    }

    /// Creates a path using the given [`ChannelPath`] (possibly empty) and the given `destination` address.
    pub fn from_channel_path(mut path: ChannelPath, destination: Address) -> Self {
        path.push(destination);
        Self(path)
    }

    /// Creates a direct path just to the `destination`.
    pub fn direct(destination: Address) -> Self {
        Self(vec![destination])
    }

    /// Converts this chain path into the [`ChainPath`] by removing the destination.
    pub fn into_channel_path(mut self) -> ChannelPath {
        self.0.pop();
        self.0
    }
}

impl Deref for ChainPath {
    type Target = [Address];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for ChainPath {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "chain path [{}]",
            self.0.iter().map(|p| p.to_hex()).collect::<Vec<String>>().join(", ")
        )
    }
}

impl From<ChainPath> for ChannelPath {
    fn from(value: ChainPath) -> Self {
        let len = value.0.len();
        value.0.into_iter().take(len - 1).collect()
    }
}

impl IntoIterator for ChainPath {
    type IntoIter = std::vec::IntoIter<Self::Item>;
    type Item = Address;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl Path<Address> for ChainPath {
    fn invert(self) -> Option<Self> {
        Some(Self(self.0.into_iter().rev().collect()))
    }
}

impl NonEmptyPath<Address> for ChainPath {}

/// Allows resolution of [`OffchainPublicKey`] for a given [`Address`] or vice versa
/// and retrieval of [`ChannelEntry`] based on the parties.
#[cfg_attr(test, mockall::automock)]
#[async_trait::async_trait]
pub trait PathAddressResolver {
    /// Resolve [`OffchainPublicKey`] from the given [`Address`]
    async fn resolve_transport_address(&self, address: &Address) -> Result<Option<OffchainPublicKey>, PathError>;
    /// Resolve [`Address`] from the given [`OffchainPublicKey`]
    async fn resolve_chain_address(&self, key: &OffchainPublicKey) -> Result<Option<Address>, PathError>;
    /// Resolve [`ChannelEntry`] based on the given `src` and `dst` addresses.
    async fn get_channel(&self, src: &Address, dst: &Address) -> Result<Option<ChannelEntry>, PathError>;
}

/// Represents [`NonEmptyPath`] that has been resolved and validated.
///
/// Such a path can be directly used to deliver packets.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidatedPath(TransportPath, ChainPath);

impl ValidatedPath {
    /// Shortcut to create a direct path to a destination with the given addresses.
    pub fn direct(dst_key: OffchainPublicKey, dst_address: Address) -> Self {
        Self(TransportPath(vec![dst_key]), ChainPath(vec![dst_address]))
    }

    /// Turns the given `path` into a [`ValidatedPath`].
    ///
    /// This makes sure that all addresses and channels on the path exist
    /// and do resolve to the corresponding [`OffchainPublicKeys`](OffchainPublicKey) or
    /// [`Addresses`](Address).
    ///
    /// If the given path is empty or unresolvable, an error is returned.
    pub async fn new<N, P, O, R>(origin: O, path: P, resolver: &R) -> Result<ValidatedPath, PathError>
    where
        N: Into<PathAddress> + Copy,
        P: Path<N>,
        O: Into<PathAddress>,
        R: PathAddressResolver + Send,
    {
        if path.is_empty() {
            return Err(PathNotValid);
        }

        let mut ticket_issuer = match origin.into() {
            PathAddress::Chain(addr) => addr,
            PathAddress::Offchain(key) => resolver
                .resolve_chain_address(&key)
                .await?
                .ok_or(InvalidPeer(key.to_peerid_str()))?,
        };

        let mut keys = Vec::with_capacity(path.num_hops());
        let mut addrs = Vec::with_capacity(path.num_hops());

        let num_hops = path.num_hops();
        for (i, hop) in path.into_iter().enumerate() {
            // Resolve the counterpart address
            // and get the chain Address to validate against the channel graph
            let ticket_receiver = match &hop.into() {
                PathAddress::Chain(addr) => {
                    let key = resolver
                        .resolve_transport_address(addr)
                        .await?
                        .ok_or(InvalidPeer(addr.to_hex()))?;
                    keys.push(key);
                    addrs.push(*addr);
                    *addr
                }
                PathAddress::Offchain(key) => {
                    let addr = resolver
                        .resolve_chain_address(key)
                        .await?
                        .ok_or(InvalidPeer(key.to_peerid_str()))?;
                    addrs.push(addr);
                    keys.push(*key);
                    addr
                }
            };

            // Check for loops
            if ticket_issuer == ticket_receiver {
                return Err(LoopsNotAllowed(ticket_receiver.to_hex()));
            }

            // Check if the channel is opened, if not the last hop
            if i < num_hops - 1 {
                let channel = resolver
                    .get_channel(&ticket_issuer, &ticket_receiver)
                    .await?
                    .ok_or(MissingChannel(ticket_issuer.to_hex(), ticket_receiver.to_hex()))?;

                if channel.status != ChannelStatus::Open {
                    return Err(ChannelNotOpened(ticket_issuer.to_hex(), ticket_receiver.to_hex()));
                }
            }

            ticket_issuer = ticket_receiver;
        }

        debug_assert_eq!(keys.len(), addrs.len());

        Ok(ValidatedPath(TransportPath(keys), ChainPath(addrs)))
    }

    /// Valid chain path.
    pub fn chain_path(&self) -> &ChainPath {
        &self.1
    }

    /// Valid transport path.
    pub fn transport_path(&self) -> &TransportPath {
        &self.0
    }
}

impl Deref for ValidatedPath {
    type Target = [OffchainPublicKey];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl IntoIterator for ValidatedPath {
    type IntoIter = std::vec::IntoIter<Self::Item>;
    type Item = OffchainPublicKey;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl Path<OffchainPublicKey> for ValidatedPath {
    /// Returns always `None`.
    ///
    /// A validated path cannot be inverted, as the inverted path could be invalid.
    fn invert(self) -> Option<Self> {
        None
    }
}

impl Display for ValidatedPath {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "validated path [{}]",
            self.1.0.iter().map(|p| p.to_hex()).collect::<Vec<String>>().join(", ")
        )
    }
}

impl NonEmptyPath<OffchainPublicKey> for ValidatedPath {}

/// Trait for implementing a custom path selection algorithm from the channel graph.
#[async_trait::async_trait]
pub trait PathSelector {
    /// Select a path of maximum `max_hops` from `source` to `destination` in the given channel graph.
    /// NOTE: the resulting path does not contain `source` but does contain `destination`.
    /// Fails if no such path can be found.
    async fn select_path(
        &self,
        source: Address,
        destination: Address,
        min_hops: usize,
        max_hops: usize,
    ) -> Result<ChannelPath, PathError>;

    /// Constructs a new valid packet `Path` from source to the given destination.
    /// This method uses `INTERMEDIATE_HOPS` as the maximum number of hops and 1 hop as a minimum.
    async fn select_auto_path(&self, source: Address, destination: Address) -> Result<ChannelPath, PathError> {
        self.select_path(source, destination, 1usize, INTERMEDIATE_HOPS).await
    }
}

/// A path selector that does not resolve any path, always returns [`PathError::PathNotFound`].
#[derive(Debug, Clone, Copy, Default)]
pub struct NoPathSelector;

#[async_trait::async_trait]
impl PathSelector for NoPathSelector {
    async fn select_path(
        &self,
        source: Address,
        destination: Address,
        min_hops: usize,
        _max_hops: usize,
    ) -> Result<ChannelPath, PathError> {
        Err(PathError::PathNotFound(
            min_hops,
            source.to_string(),
            destination.to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use std::{
        iter,
        ops::Add,
        str::FromStr,
        time::{Duration, SystemTime},
    };

    use anyhow::{Context, ensure};
    use async_trait::async_trait;
    use hex_literal::hex;
    use parameterized::parameterized;

    use super::*;

    lazy_static::lazy_static! {
        pub static ref PATH_ADDRS: bimap::BiMap<OffchainPublicKey, Address> = bimap::BiMap::from_iter([
            (OffchainPublicKey::from_privkey(&hex!("e0bf93e9c916104da00b1850adc4608bd7e9087bbd3f805451f4556aa6b3fd6e")).unwrap(), Address::from_str("0x0000c178cf70d966be0a798e666ce2782c7b2288").unwrap()),
            (OffchainPublicKey::from_privkey(&hex!("cfc66f718ec66fb822391775d749d7a0d66b690927673634816b63339bc12a3c")).unwrap(), Address::from_str("0x1000d5786d9e6799b3297da1ad55605b91e2c882").unwrap()),
            (OffchainPublicKey::from_privkey(&hex!("203ca4d3c5f98dd2066bb204b5930c10b15c095585c224c826b4e11f08bfa85d")).unwrap(), Address::from_str("0x200060ddced1e33c9647a71f4fc2cf4ed33e4a9d").unwrap()),
            (OffchainPublicKey::from_privkey(&hex!("fc71590e473b3e64e498e8a7f03ed19d1d7ee5f438c5d41300e8bb228b6b5d3c")).unwrap(), Address::from_str("0x30004105095c8c10f804109b4d1199a9ac40ed46").unwrap()),
            (OffchainPublicKey::from_privkey(&hex!("4ab03f6f75f845ca1bf8b7104804ea5bda18bda29d1ec5fc5d4267feca5fb8e1")).unwrap(), Address::from_str("0x4000a288c38fa8a0f4b79127747257af4a03a623").unwrap()),
            (OffchainPublicKey::from_privkey(&hex!("a1859043a6ae231259ad0bccac9a62377cd2b0700ba2248fcfa52ae1461f7087")).unwrap(), Address::from_str("0x50002f462ec709cf181bbe44a7e952487bd4591d").unwrap()),
        ]);
        pub static ref ADDRESSES: Vec<Address> = sorted_peers().iter().map(|p| p.1).collect();
    }

    pub fn sorted_peers() -> Vec<(OffchainPublicKey, Address)> {
        let mut peers = PATH_ADDRS.iter().map(|(pk, a)| (*pk, *a)).collect::<Vec<_>>();
        peers.sort_by(|a, b| a.1.to_string().cmp(&b.1.to_string()));
        peers
    }

    pub fn dummy_channel(src: Address, dst: Address, status: ChannelStatus) -> ChannelEntry {
        ChannelEntry::new(src, dst, 1.into(), 1, status, 1)
    }

    struct DummyResolver(Vec<ChannelEntry>);

    impl DummyResolver {
        pub fn new(_me: Address) -> (Self, Vec<(OffchainPublicKey, Address)>) {
            let addrs = sorted_peers();

            let ts = SystemTime::now().add(Duration::from_secs(10));

            // Channels: 0 -> 1 -> 2 -> 3 -> 4, 4 /> 0, 3 -> 1
            let cg = vec![
                dummy_channel(addrs[0].1, addrs[1].1, ChannelStatus::Open),
                dummy_channel(addrs[1].1, addrs[2].1, ChannelStatus::Open),
                dummy_channel(addrs[2].1, addrs[3].1, ChannelStatus::Open),
                dummy_channel(addrs[3].1, addrs[4].1, ChannelStatus::Open),
                dummy_channel(addrs[3].1, addrs[1].1, ChannelStatus::Open),
                dummy_channel(addrs[4].1, addrs[0].1, ChannelStatus::PendingToClose(ts)),
            ];

            (Self(cg), addrs)
        }
    }

    #[async_trait]
    impl PathAddressResolver for DummyResolver {
        async fn resolve_transport_address(&self, address: &Address) -> Result<Option<OffchainPublicKey>, PathError> {
            Ok(PATH_ADDRS.get_by_right(address).copied())
        }

        async fn resolve_chain_address(&self, key: &OffchainPublicKey) -> Result<Option<Address>, PathError> {
            Ok(PATH_ADDRS.get_by_left(key).copied())
        }

        async fn get_channel(&self, src: &Address, dst: &Address) -> Result<Option<ChannelEntry>, PathError> {
            Ok(self
                .0
                .iter()
                .find(|c| c.source == *src && c.destination == *dst)
                .cloned())
        }
    }

    #[test]
    fn chain_path_zero_hop_should_fail() -> anyhow::Result<()> {
        ensure!(ChainPath::new::<Address, _>([]).is_err(), "must fail for zero hop");
        Ok(())
    }

    #[test]
    fn transport_path_zero_hop_should_fail() -> anyhow::Result<()> {
        ensure!(
            TransportPath::new::<OffchainPublicKey, _>([]).is_err(),
            "must fail for zero hop"
        );
        Ok(())
    }

    #[parameterized(hops = { 1, 2, 3 })]
    #[parameterized_macro(tokio::test)]
    async fn validated_path_multi_hop_validation(hops: usize) -> anyhow::Result<()> {
        let (cg, peers) = DummyResolver::new(ADDRESSES[0]);

        // path: 0 -> 1 -> 2 -> 3 -> 4
        let chain_path = ChainPath::new(peers.iter().skip(1).take(hops + 1).map(|(_, a)| *a))?;

        assert_eq!(hops + 1, chain_path.num_hops(), "must be a {hops} hop path");
        ensure!(!chain_path.contains_cycle(), "must not be cyclic");

        let validated = ValidatedPath::new(ADDRESSES[0], chain_path.clone(), &cg)
            .await
            .context(format!("must be valid {hops} hop path"))?;

        assert_eq!(
            chain_path.num_hops(),
            validated.num_hops(),
            "validated path must have the same length"
        );
        assert_eq!(
            validated.chain_path(),
            &chain_path,
            "validated path must have the same chain path"
        );

        assert_eq!(
            peers.into_iter().skip(1).take(hops + 1).collect::<Vec<_>>(),
            validated
                .transport_path()
                .iter()
                .copied()
                .zip(validated.chain_path().iter().copied())
                .collect::<Vec<_>>(),
            "validated path must have the same transport path"
        );

        Ok(())
    }

    #[parameterized(hops = { 1, 2, 3 })]
    #[parameterized_macro(tokio::test)]
    async fn validated_path_revalidation_should_be_identity(hops: usize) -> anyhow::Result<()> {
        let (cg, peers) = DummyResolver::new(ADDRESSES[0]);

        // path: 0 -> 1 -> 2 -> 3 -> 4
        let chain_path = ChainPath::new(peers.iter().skip(1).take(hops + 1).map(|(_, a)| *a))?;

        let validated_1 = ValidatedPath::new(ADDRESSES[0], chain_path.clone(), &cg)
            .await
            .context(format!("must be valid {hops} hop path"))?;

        let validated_2 = ValidatedPath::new(ADDRESSES[0], validated_1.clone(), &cg)
            .await
            .context(format!("must be valid {hops} hop path"))?;

        assert_eq!(validated_1, validated_2, "revalidation must be identity");

        Ok(())
    }

    #[parameterized(hops = { 2, 3 })]
    #[parameterized_macro(tokio::test)]
    async fn validated_path_must_allow_cyclic(hops: usize) -> anyhow::Result<()> {
        let (cg, peers) = DummyResolver::new(ADDRESSES[0]);

        // path: 0 -> 1 -> 2 -> 3 -> 1
        let chain_path = ChainPath::new(
            peers
                .iter()
                .skip(1)
                .take(hops)
                .map(|(_, a)| *a)
                .chain(iter::once(peers[1].1)),
        )?;

        assert_eq!(hops + 1, chain_path.num_hops(), "must be a {hops} hop path");
        assert!(chain_path.contains_cycle(), "must be cyclic");

        let validated = ValidatedPath::new(ADDRESSES[0], chain_path.clone(), &cg)
            .await
            .context(format!("must be valid {hops} hop path"))?;

        assert_eq!(
            chain_path.num_hops(),
            validated.num_hops(),
            "validated path must have the same length"
        );
        assert_eq!(
            validated.chain_path(),
            &chain_path,
            "validated path must have the same chain path"
        );

        assert_eq!(
            peers
                .iter()
                .copied()
                .skip(1)
                .take(hops)
                .chain(iter::once(peers[1]))
                .collect::<Vec<_>>(),
            validated
                .transport_path()
                .iter()
                .copied()
                .zip(validated.chain_path().iter().copied())
                .collect::<Vec<_>>(),
            "validated path must have the same transport path"
        );

        Ok(())
    }

    #[tokio::test]
    async fn validated_path_should_allow_zero_hop_with_non_existing_channel() -> anyhow::Result<()> {
        let (cg, peers) = DummyResolver::new(ADDRESSES[0]);

        // path: 0 -> 3 (channel 0 -> 3 does not exist)
        let chain_path = ChainPath::new([peers[3].1])?;

        let validated = ValidatedPath::new(ADDRESSES[0], chain_path.clone(), &cg)
            .await
            .context("must be valid path")?;

        assert_eq!(&chain_path, validated.chain_path(), "path must be the same");

        Ok(())
    }

    #[tokio::test]
    async fn validated_path_should_allow_zero_hop_with_non_open_channel() -> anyhow::Result<()> {
        let (cg, peers) = DummyResolver::new(ADDRESSES[0]);

        // path: 4 -> 0 (channel 4 -> 0 is PendingToClose)
        let chain_path = ChainPath::new([peers[0].1])?;

        let validated = ValidatedPath::new(ADDRESSES[4], chain_path.clone(), &cg)
            .await
            .context("must be valid path")?;

        assert_eq!(&chain_path, validated.chain_path(), "path must be the same");

        Ok(())
    }

    #[tokio::test]
    async fn validated_path_should_allow_non_existing_channel_for_last_hop() -> anyhow::Result<()> {
        let (cg, peers) = DummyResolver::new(ADDRESSES[0]);

        // path: 0 -> 1 -> 3 (channel 1 -> 3 does not exist)
        let chain_path = ChainPath::new([peers[1].1, peers[3].1])?;

        let validated = ValidatedPath::new(ADDRESSES[0], chain_path.clone(), &cg)
            .await
            .context("must be valid path")?;

        assert_eq!(&chain_path, validated.chain_path(), "path must be the same");

        Ok(())
    }

    #[tokio::test]
    async fn validated_path_should_allow_non_open_channel_for_the_last_hop() -> anyhow::Result<()> {
        let (cg, peers) = DummyResolver::new(ADDRESSES[0]);

        // path: 3 -> 4 -> 0 (channel 4 -> 0 is PendingToClose)
        let chain_path = ChainPath::new([peers[4].1, peers[0].1])?;

        let validated = ValidatedPath::new(ADDRESSES[3], chain_path.clone(), &cg)
            .await
            .context("must be valid path")?;

        assert_eq!(&chain_path, validated.chain_path(), "path must be the same");

        Ok(())
    }

    #[tokio::test]
    async fn validated_path_should_fail_for_non_open_channel_not_in_the_last_hop() -> anyhow::Result<()> {
        let (cg, peers) = DummyResolver::new(ADDRESSES[0]);

        // path: 4 -> 0 -> 1 (channel 4 -> 0 is PendingToClose)
        let chain_path = ChainPath::new([peers[0].1, peers[1].1])?;

        ensure!(
            ValidatedPath::new(ADDRESSES[4], chain_path, &cg).await.is_err(),
            "path must not be constructible"
        );

        // path: 3 -> 4 -> 0 -> 1 (channel 4 -> 0 is PendingToClose)
        let chain_path = ChainPath::new([peers[4].1, peers[0].1, peers[1].1])?;

        ensure!(
            ValidatedPath::new(ADDRESSES[3], chain_path, &cg).await.is_err(),
            "path must not be constructible"
        );

        // path: 2 -> 3 -> 4 -> 0 -> 1 (channel 4 -> 0 is PendingToClose)
        let chain_path = ChainPath::new([peers[3].1, peers[4].1, peers[0].1, peers[1].1])?;

        ensure!(
            ValidatedPath::new(ADDRESSES[2], chain_path, &cg).await.is_err(),
            "path must not be constructible"
        );

        Ok(())
    }

    #[tokio::test]
    async fn validated_path_should_fail_for_non_existing_channel_not_in_the_last_hop() -> anyhow::Result<()> {
        let (cg, peers) = DummyResolver::new(ADDRESSES[0]);

        // path: 0 -> 3 -> 4 (channel 0 -> 3 does not exist)
        let chain_path = ChainPath::new([peers[3].1, peers[4].1])?;

        ensure!(
            ValidatedPath::new(ADDRESSES[0], chain_path, &cg).await.is_err(),
            "path must not be constructible"
        );

        // path: 0 -> 1 -> 3 -> 0 (channel 1 -> 3 does not exist)
        let chain_path = ChainPath::new([peers[1].1, peers[3].1, peers[0].1])?;

        ensure!(
            ValidatedPath::new(ADDRESSES[0], chain_path, &cg).await.is_err(),
            "path must not be constructible"
        );

        // path: 0 -> 1 -> 2 -> 4 -> 0 (channel 2 -> 4 does not exist)
        let chain_path = ChainPath::new([peers[1].1, peers[2].1, peers[2].1, peers[0].1])?;

        ensure!(
            ValidatedPath::new(ADDRESSES[0], chain_path, &cg).await.is_err(),
            "path must not be constructible"
        );

        Ok(())
    }

    #[tokio::test]
    async fn validated_path_should_not_allow_simple_loops() -> anyhow::Result<()> {
        let (cg, peers) = DummyResolver::new(ADDRESSES[0]);

        // path: 0 -> 1 -> 1 -> 2
        let chain_path = ChainPath::new([peers[1].1, peers[1].1, peers[2].1])?;

        assert!(chain_path.contains_cycle(), "path must contain a cycle");

        ensure!(
            ValidatedPath::new(ADDRESSES[0], chain_path, &cg).await.is_err(),
            "path must not be constructible"
        );

        Ok(())
    }

    #[tokio::test]
    async fn validated_path_should_allow_long_cycles() -> anyhow::Result<()> {
        let (cg, peers) = DummyResolver::new(ADDRESSES[0]);

        // path 0 -> 1 -> 2 -> 3 -> 1 -> 2
        let chain_path = ChainPath::new([peers[1].1, peers[2].1, peers[3].1, peers[1].1, peers[2].1])?;

        assert!(chain_path.contains_cycle(), "path must contain a cycle");

        let validated = ValidatedPath::new(ADDRESSES[0], chain_path.clone(), &cg)
            .await
            .context("must be valid path")?;

        assert_eq!(&chain_path, validated.chain_path(), "path must be the same");

        Ok(())
    }
}
