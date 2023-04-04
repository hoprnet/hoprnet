use crate::derivation::derive_mac_key;
use crate::errors::CryptoError;
use crate::errors::CryptoError::TagMismatch;
use crate::parameters::SECRET_KEY_LENGTH;
use crate::prg::{PRG, PRGParameters};
use crate::primitives::{create_tagged_mac, SimpleMac};
use crate::random::random_fill;
use crate::types::PublicKey;
use crate::utils::xor_inplace;
use crate::errors::Result;
use crate::routing::ForwardedHeader::{FinalNode, RelayNode};

const RELAYER_END_PREFIX: u8 = 0xff;

fn generate_filler(max_hops: usize, routing_info_len: usize, routing_info_last_hop_len: usize, secrets: &[&[u8]]) -> Box<[u8]> {
    assert!(secrets.len() >= 2, "too few secrets given");
    assert!(max_hops >= secrets.len(), "too few hops");
    assert!(routing_info_len > 0, "invalid routing info length");

    let header_len = routing_info_last_hop_len + (max_hops - 1) * routing_info_len;
    let padding_len = (max_hops - secrets.len()) * routing_info_len;

    let mut ret = vec![0u8; header_len - padding_len - routing_info_last_hop_len];

    let mut length = routing_info_len;
    let mut start = header_len;

    for &secret in secrets.iter().take(secrets.len() - 1) {
        let prg = PRG::from_parameters(PRGParameters::new(secret));

        let digest = prg.digest(start, header_len + routing_info_len);
        xor_inplace(&mut ret[0..length], digest.as_ref());

        length += routing_info_len;
        start -= routing_info_len;
    }

    ret.into_boxed_slice()
}

pub struct RoutingInfo {
    pub routing_information: Box<[u8]>,
    pub mac: Box<[u8]>
}

impl RoutingInfo {
    pub fn new(max_hops: usize, path: &[PublicKey], secrets: &[&[u8]], additional_data_relayer_len: usize,
               additional_data_relayer: &[&[u8]], additional_data_last_hop: Option<&[u8]>) -> Self {
        assert!(secrets.iter().all(|s| s.len() == SECRET_KEY_LENGTH), "invalid secret length");
        assert!(secrets.len() <= max_hops, "too many secrets given");
        assert!(additional_data_relayer.iter().all(|r| r.len() == additional_data_relayer_len), "invalid relayer data length");
        assert!(additional_data_last_hop.is_none() || !additional_data_last_hop.unwrap().is_empty(), "invalid additional data for last hop");

        // TODO: check the public key and curve point abstraction
        let routing_info_len = additional_data_relayer_len + SimpleMac::SIZE + PublicKey::SIZE_COMPRESSED;
        let last_hop_len = additional_data_last_hop.map(|d| d.len()).unwrap_or(0) + 1; // end prefix length

        let header_len = last_hop_len + (max_hops - 1) * routing_info_len;
        let extended_header_len = last_hop_len + max_hops * routing_info_len;

        let mut extended_header = vec![0u8; extended_header_len];
        let mut mac = [0u8; SimpleMac::SIZE];

        for idx in 0..secrets.len() {
            let inverted_idx = secrets.len() - idx - 1;
            let secret = secrets[idx];
            let prg = PRG::from_parameters(PRGParameters::new(secret));

            if idx == 0 {
                extended_header[0] = RELAYER_END_PREFIX;

                if let Some(data) = additional_data_last_hop {
                    extended_header[1..data.len()].copy_from_slice(data);
                }

                let padding_len = (max_hops - secrets.len()) * routing_info_len;
                if padding_len > 0 {
                    random_fill(&mut extended_header[last_hop_len..padding_len]);
                }

                let key_stream = prg.digest(0, last_hop_len + padding_len);
                xor_inplace(&mut extended_header[0..last_hop_len + padding_len], &key_stream);

                if secrets.len() > 1 {
                    let filler = generate_filler(max_hops, routing_info_len, last_hop_len, secrets);
                    extended_header[last_hop_len + padding_len .. filler.len()].copy_from_slice(&filler);
                }
            } else {
                extended_header.copy_within(0..header_len, routing_info_len);
                extended_header[0..PublicKey::SIZE_COMPRESSED].copy_from_slice(&path[inverted_idx + 1].serialize(true));
                extended_header[PublicKey::SIZE_COMPRESSED..PublicKey::SIZE_COMPRESSED + SimpleMac::SIZE].copy_from_slice(&mac);

                extended_header[PublicKey::SIZE_COMPRESSED + SimpleMac::SIZE..PublicKey::SIZE_COMPRESSED +
                    SimpleMac::SIZE + additional_data_relayer[inverted_idx].len()]
                    .copy_from_slice(additional_data_relayer[inverted_idx]);

                let key_stream = prg.digest(0, header_len);
                xor_inplace(&mut extended_header, &key_stream);
            }

            let mut m = derive_mac_key(secret).and_then(|k| SimpleMac::new(&k)).unwrap();
            m.update(&extended_header[0..header_len]);
            m.finalize_into(&mut mac);
        }

        Self {
            routing_information: Box::from(&extended_header[0..header_len]),
            mac: mac.into()
        }
    }
}

pub enum ForwardedHeader {
    RelayNode {
        header: Box<[u8]>,
        mac: Box<[u8]>,
        next_node: Box<[u8]>,
        additional_info: Box<[u8]>
    },

    FinalNode {
        additional_data: Box<[u8]>
    }
}

pub fn forward_transform_header(secret: &[u8], pre_header: &[u8], mac: &[u8], max_hops: usize,
                                additional_data_relayer_len: usize, additional_data_last_hop_len: usize) -> Result<ForwardedHeader> {
    assert_eq!(SECRET_KEY_LENGTH, secret.len(), "invalid secret length");
    assert_eq!(SimpleMac::SIZE, mac.len(), "invalid mac length");

    let routing_info_len = additional_data_relayer_len + PublicKey::SIZE_COMPRESSED + SimpleMac::SIZE;
    let last_hop_len = additional_data_last_hop_len + 1; // end prefix

    let header_len = last_hop_len + (max_hops - 1) * routing_info_len;
    let mut header: Vec<u8> = pre_header.into();

    assert_eq!(header_len, pre_header.len(), "invalid pre-header length");

    // TODO: make this constant time equal
    if create_tagged_mac(secret, &header).unwrap().as_ref() != mac {
        return Err(TagMismatch)
    }

    // Unmask the header using the keystream
    let prg = PRG::from_parameters(PRGParameters::new(secret));
    let key_stream = prg.digest(0, header_len);
    xor_inplace(&mut header, &key_stream);

    if header[0] != RELAYER_END_PREFIX {
        // Try to deserialize the public key to validate it
        let next_node_pk = PublicKey::deserialize(&header[..PublicKey::SIZE_COMPRESSED])
            .map_err(|_| CryptoError::CalculationError)?;

        let mac: Box<[u8]> = (&header[PublicKey::SIZE_COMPRESSED..PublicKey::SIZE_COMPRESSED+SimpleMac::SIZE]).into();
        let additional_info: Box<[u8]> = (&header[PublicKey::SIZE_COMPRESSED+SimpleMac::SIZE..
            PublicKey::SIZE_COMPRESSED+SimpleMac::SIZE+additional_data_relayer_len]).into();

        header.copy_within(routing_info_len.., 0);
        let key_stream = prg.digest(header_len, header_len + routing_info_len);
        header[header_len - routing_info_len..].copy_from_slice(&key_stream);

        Ok(RelayNode {
            next_node: next_node_pk.serialize(true),
            header: (&header[..header_len]).into(),
            mac,
            additional_info,
        })
    } else {
        Ok(FinalNode {
            additional_data: (&header[1..1+additional_data_last_hop_len]).into()
        })
    }
}

#[cfg(test)]
pub mod tests {

    #[test]
    fn test_filler_generate_verify() {

    }
}