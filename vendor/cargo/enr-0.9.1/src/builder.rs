use crate::{Enr, EnrError, EnrKey, EnrPublicKey, Key, NodeId, MAX_ENR_SIZE};
use bytes::{Bytes, BytesMut};
use rlp::{Encodable, RlpStream};
use std::{
    collections::BTreeMap,
    marker::PhantomData,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
};

/// The base builder for generating ENR records with arbitrary signing algorithms.
pub struct EnrBuilder<K: EnrKey> {
    /// The identity scheme used to build the ENR record.
    id: String,

    /// The starting sequence number for the ENR record.
    seq: u64,

    /// The key-value pairs for the ENR record.
    /// Values are stored as RLP encoded bytes.
    content: BTreeMap<Key, Bytes>,

    /// Pins the generic key types.
    phantom: PhantomData<K>,
}

impl<K: EnrKey> EnrBuilder<K> {
    /// Constructs a minimal `EnrBuilder` providing only a sequence number.
    /// Currently only supports the id v4 scheme and therefore disallows creation of any other
    /// scheme.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            seq: 1,
            content: BTreeMap::new(),
            phantom: PhantomData,
        }
    }

    /// Modifies the sequence number of the builder.
    pub fn seq(&mut self, seq: u64) -> &mut Self {
        self.seq = seq;
        self
    }

    /// Adds an arbitrary key-value to the `ENRBuilder`.
    pub fn add_value<T: Encodable>(&mut self, key: impl AsRef<[u8]>, value: &T) -> &mut Self {
        self.add_value_rlp(key, rlp::encode(value).freeze())
    }

    /// Adds an arbitrary key-value where the value is raw RLP encoded bytes.
    pub fn add_value_rlp(&mut self, key: impl AsRef<[u8]>, rlp: Bytes) -> &mut Self {
        self.content.insert(key.as_ref().to_vec(), rlp);
        self
    }

    /// Adds an `ip`/`ip6` field to the `ENRBuilder`.
    pub fn ip(&mut self, ip: IpAddr) -> &mut Self {
        match ip {
            IpAddr::V4(addr) => self.ip4(addr),
            IpAddr::V6(addr) => self.ip6(addr),
        }
    }

    /// Adds an `ip` field to the `ENRBuilder`.
    pub fn ip4(&mut self, ip: Ipv4Addr) -> &mut Self {
        self.add_value("ip", &ip.octets().as_ref());
        self
    }

    /// Adds an `ip6` field to the `ENRBuilder`.
    pub fn ip6(&mut self, ip: Ipv6Addr) -> &mut Self {
        self.add_value("ip6", &ip.octets().as_ref());
        self
    }

    /*
     * Removed from the builder as only the v4 scheme is currently supported.
     * This is set as default in the builder.

    /// Adds an `Id` field to the `ENRBuilder`.
    pub fn id(&mut self, id: &str) -> &mut Self {
        self.add_value("id", &id.as_bytes());
        self
    }
    */

    /// Adds a `tcp` field to the `ENRBuilder`.
    pub fn tcp4(&mut self, tcp: u16) -> &mut Self {
        self.add_value("tcp", &tcp);
        self
    }

    /// Adds a `tcp6` field to the `ENRBuilder`.
    pub fn tcp6(&mut self, tcp: u16) -> &mut Self {
        self.add_value("tcp6", &tcp);
        self
    }

    /// Adds a `udp` field to the `ENRBuilder`.
    pub fn udp4(&mut self, udp: u16) -> &mut Self {
        self.add_value("udp", &udp);
        self
    }

    /// Adds a `udp6` field to the `ENRBuilder`.
    pub fn udp6(&mut self, udp: u16) -> &mut Self {
        self.add_value("udp6", &udp);
        self
    }

    /// Generates the rlp-encoded form of the ENR specified by the builder config.
    fn rlp_content(&self) -> BytesMut {
        let mut stream = RlpStream::new_with_buffer(BytesMut::with_capacity(MAX_ENR_SIZE));
        stream.begin_list(self.content.len() * 2 + 1);
        stream.append(&self.seq);
        for (k, v) in &self.content {
            stream.append(k);
            // The values are stored as raw RLP encoded bytes
            stream.append_raw(v, 1);
        }
        stream.out()
    }

    /// Signs record based on the identity scheme. Currently only "v4" is supported.
    fn signature(&self, key: &K) -> Result<Vec<u8>, EnrError> {
        match self.id.as_str() {
            "v4" => key
                .sign_v4(&self.rlp_content())
                .map_err(|_| EnrError::SigningError),
            // unsupported identity schemes
            _ => Err(EnrError::SigningError),
        }
    }

    /// Adds a public key to the ENR builder.
    fn add_public_key(&mut self, key: &K::PublicKey) {
        self.add_value(key.enr_key(), &key.encode().as_ref());
    }

    /// Constructs an ENR from the `EnrBuilder`.
    ///
    /// # Errors
    /// Fails if the identity scheme is not supported, or the record size exceeds `MAX_ENR_SIZE`.
    pub fn build(&mut self, key: &K) -> Result<Enr<K>, EnrError> {
        // add the identity scheme to the content
        if self.id != "v4" {
            return Err(EnrError::UnsupportedIdentityScheme);
        }

        // Sanitize all data, ensuring all RLP data is correctly formatted.
        for (key, value) in &self.content {
            if rlp::Rlp::new(value).data().is_err() {
                return Err(EnrError::InvalidRlpData(
                    String::from_utf8_lossy(key).into(),
                ));
            }
        }

        self.add_value_rlp("id", rlp::encode(&self.id.as_bytes()).freeze());

        self.add_public_key(&key.public());
        let rlp_content = self.rlp_content();

        let signature = self.signature(key)?;

        // check the size of the record
        if rlp_content.len() + signature.len() + 8 > MAX_ENR_SIZE {
            return Err(EnrError::ExceedsMaxSize);
        }

        Ok(Enr {
            seq: self.seq,
            node_id: NodeId::from(key.public()),
            content: self.content.clone(),
            signature,
            phantom: PhantomData,
        })
    }
}
