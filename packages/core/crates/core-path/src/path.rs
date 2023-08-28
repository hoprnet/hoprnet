use crate::errors::PathError;
use crate::errors::PathError::{ChannelNotOpened, InvalidPeer, LoopsNotAllowed, MissingChannel, PathNotValid};
use crate::errors::Result;
use core_crypto::types::OffchainPublicKey;
use core_ethereum_db::traits::HoprCoreEthereumDbActions;
use core_types::channels::ChannelStatus;
use libp2p_identity::PeerId;
use std::fmt::{Display, Formatter};
use utils_log::warn;
use utils_types::primitives::Address;
use utils_types::traits::{PeerIdLike, ToHex};

/// Represents a path for a packet.
/// The type internally carries an information if the path has been already validated or not (since path validation
/// is potentially an expensive operation).
/// Path validation process checks if all the channels on the path exist and are in an `Open` state.
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
#[derive(Debug, Clone)]
pub struct Path {
    hops: Vec<PeerId>,
    valid: bool,
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
impl Path {
    /// Number of hops in the path.
    pub fn length(&self) -> u32 {
        self.hops.len() as u32
    }

    /// Determines with the path is valid.
    pub fn valid(&self) -> bool {
        !self.hops.is_empty() && self.valid
    }
}

impl Path {
    /// Creates an already pre-validated path.
    pub fn new_valid(validated_path: Vec<PeerId>) -> Self {
        Self {
            hops: validated_path,
            valid: true,
        }
    }

    /// Creates a new path by validating the list of peer ids using the channel database
    /// The given path does not contain the sender node, which is given by `self_addr`
    pub async fn new<T: HoprCoreEthereumDbActions>(
        path: Vec<PeerId>,
        self_addr: &Address,
        allow_loops: bool,
        db: &T,
    ) -> Result<Self> {
        if path.is_empty() {
            return Err(PathNotValid);
        }

        let mut ticket_receiver;
        let mut ticket_issuer = *self_addr;

        // Ignore the last hop in the check, because channels are not required for direct messages
        for hop in path.iter().take(path.len() - 1) {
            ticket_receiver = db
                .get_chain_key(&OffchainPublicKey::from_peerid(hop)?)
                .await?
                .ok_or(InvalidPeer(format!("could not find channel key for {hop}")))?;

            // Check for loops
            if ticket_issuer == ticket_receiver {
                if !allow_loops {
                    return Err(LoopsNotAllowed(ticket_receiver.to_hex()));
                }
                warn!("duplicated adjacent path entries")
            }

            // Check if the channel is opened
            let channel = db
                .get_channel_x(&ticket_issuer, &ticket_receiver)
                .await?
                .ok_or(MissingChannel(ticket_issuer.to_hex(), ticket_receiver.to_hex()))?;

            if channel.status != ChannelStatus::Open {
                return Err(ChannelNotOpened(ticket_issuer.to_hex(), ticket_receiver.to_hex()));
            }

            ticket_issuer = ticket_receiver;
        }

        Ok(Self::new_valid(path))
    }

    /// Individual hops in the path.
    pub fn hops(&self) -> &[PeerId] {
        &self.hops
    }
}

impl<T> TryFrom<&Path> for Vec<T>
where
    T: PeerIdLike,
{
    type Error = PathError;

    fn try_from(value: &Path) -> std::result::Result<Self, Self::Error> {
        if value.valid() {
            value
                .hops()
                .iter()
                .map(|p| T::from_peerid(p).map_err(|_| InvalidPeer(p.to_string())))
                .collect()
        } else {
            Err(PathNotValid)
        }
    }
}

impl Display for Path {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{} ] ({} hops)",
            if self.valid { "[ " } else { "[ !! " },
            self.hops.iter().map(|p| p.to_string()).collect::<Vec<_>>().join("->"),
            self.length()
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::errors::PathError;
    use crate::path::Path;
    use core_crypto::types::{OffchainPublicKey, PublicKey};
    use core_ethereum_db::{db::CoreEthereumDb, traits::HoprCoreEthereumDbActions};
    use core_types::channels::{ChannelEntry, ChannelStatus};
    use hex_literal::hex;
    use libp2p_identity::PeerId;
    use std::sync::{Arc, Mutex};
    use utils_db::db::DB;
    use utils_db::leveldb::rusty::RustyLevelDbShim;
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
        const HOPS: u32 = 5;
        let peer_ids = (0..HOPS).map(|_| PeerId::random()).collect::<Vec<_>>();

        let path = Path::new_valid(peer_ids.clone());
        assert_eq!(HOPS, path.length());
        assert_eq!(&peer_ids, path.hops());
        assert!(path.valid());
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
        let mut db = CoreEthereumDb::new(
            DB::new(RustyLevelDbShim::new(Arc::new(Mutex::new(
                rusty_leveldb::DB::open("test", rusty_leveldb::in_memory()).unwrap(),
            )))),
            last_addr,
        );

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

        // Add a closed channel between 4 -> 0
        let chain_key_0 = PublicKey::from_privkey(&PEERS_PRIVS[0]).unwrap().to_address();
        let chain_key_4 = PublicKey::from_privkey(&PEERS_PRIVS[4]).unwrap().to_address();
        let channel = create_dummy_channel(chain_key_4, chain_key_0, ChannelStatus::Closed);
        db.update_channel_and_snapshot(&channel.get_id(), &channel, &testing_snapshot)
            .await
            .unwrap();

        db
    }

    #[async_std::test]
    async fn test_path_validation() {
        let mut peers = Vec::new();
        let db = create_db_with_channel_topology(&mut peers).await;

        let me = PublicKey::from_privkey(&PEERS_PRIVS[0]).unwrap().to_address();

        let path = Path::new(vec![peers[1], peers[2]], &me, false, &db)
            .await
            .expect("path 0 -> 1 -> 2 must be valid");

        assert_eq!(2, path.length());

        let path = Path::new(vec![peers[2]], &me, false, &db)
            .await
            .expect("path 0 -> 2 must be valid, because channel not needed for direct message");

        assert_eq!(1, path.length());

        let path = Path::new(vec![peers[1], peers[2], peers[3]], &me, false, &db)
            .await
            .expect("path 0 -> 1 -> 2 -> 3 must be valid");

        assert_eq!(3, path.length());

        let path = Path::new(vec![peers[1], peers[2], peers[3], peers[4]], &me, false, &db)
            .await
            .expect("path 0 -> 1 -> 2 -> 3 -> 4 must be valid");

        assert_eq!(4, path.length());

        let path = Path::new(vec![peers[1], peers[2], peers[2], peers[3]], &me, true, &db)
            .await
            .expect("path 0 -> 1 -> 2 -> 2 -> 3 must be valid if loops are allowed");

        assert_eq!(4, path.length()); // loop still counts as a hop

        match Path::new(vec![peers[1], peers[2], peers[2], peers[3]], &me, false, &db)
            .await
            .expect_err("path 0 -> 1 -> 2 -> 2 must be invalid if loops are not allowed")
        {
            PathError::LoopsNotAllowed(_) => {}
            _ => panic!("error must be LoopsNotAllowed"),
        };

        match Path::new(vec![peers[3], peers[4]], &me, false, &db)
            .await
            .expect_err("path 0 -> 3 must be invalid, because channel 0 -> 3 is not opened")
        {
            PathError::MissingChannel(_, _) => {}
            _ => panic!("error must be MissingChannel"),
        };

        match Path::new(vec![peers[1], peers[3], peers[4]], &me, false, &db)
            .await
            .expect_err("path 0 -> 1 -> 3 -> 4 must be invalid, because channel 1 -> 3 is not opened")
        {
            PathError::MissingChannel(_, _) => {}
            _ => panic!("error must be MissingChannel"),
        };

        match Path::new(
            vec![peers[1], peers[2], peers[3], peers[4], peers[0], peers[1]],
            &me,
            false,
            &db,
        )
        .await
        .expect_err("path 0 -> 1 -> 2 -> 3 -> 4 -> 0 -> 1 must be invalid, because channel 4 -> 0 is already closed")
        {
            PathError::ChannelNotOpened(_, _) => {}
            _ => panic!("error must be ChannelNotOpened"),
        };

        let me = PublicKey::from_privkey(&PEERS_PRIVS[4]).unwrap().to_address();
        match Path::new(vec![peers[0], peers[1]], &me, false, &db)
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
    use crate::errors::PathError::InvalidPeer;
    use crate::errors::Result;
    use crate::path::Path;
    use core_ethereum_db::db::wasm::Database;
    use js_sys::JsString;
    use libp2p_identity::PeerId;
    use std::str::FromStr;
    use utils_misc::ok_or_jserr;
    use utils_misc::utils::wasm::JsResult;
    use utils_types::primitives::Address;
    use wasm_bindgen::prelude::wasm_bindgen;

    #[wasm_bindgen]
    impl Path {
        #[wasm_bindgen(constructor)]
        pub fn _new_validated(validated_path: Vec<JsString>) -> JsResult<Path> {
            Ok(Path::new_valid(
                validated_path
                    .into_iter()
                    .map(|p| PeerId::from_str(&p.as_string().unwrap()).map_err(|_| InvalidPeer(p.as_string().unwrap())))
                    .collect::<Result<Vec<PeerId>>>()?,
            ))
        }

        #[wasm_bindgen(js_name = "validated")]
        pub async fn _new(
            path: Vec<JsString>,
            self_addr: &Address,
            allow_loops: bool,
            db: &Database,
        ) -> JsResult<Path> {
            let database = db.as_ref_counted();
            let g = database.read().await;
            ok_or_jserr!(
                Path::new(
                    path.into_iter()
                        .map(|p| PeerId::from_str(&p.as_string().unwrap())
                            .map_err(|_| InvalidPeer(p.as_string().unwrap())))
                        .collect::<Result<Vec<PeerId>>>()?,
                    self_addr,
                    allow_loops,
                    &*g
                )
                .await
            )
        }

        #[wasm_bindgen(js_name = "to_string")]
        pub fn _to_string(&self) -> String {
            self.to_string()
        }
    }
}
