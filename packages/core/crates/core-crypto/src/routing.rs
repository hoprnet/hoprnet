use crate::parameters::{MAC_LENGTH, SECRET_KEY_LENGTH};
use crate::prg::{PRG, PRGParameters};
use crate::types::PublicKey;
use crate::utils;

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

    for i in 0..(secrets.len() - 1) {
        let prg = PRG::from_parameters(PRGParameters::new(secrets[i]));

        let digest = prg.digest(start, header_len + routing_info_len);
        utils::xor_inplace(&mut ret[0..length], digest.as_ref());

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

        // TODO: check the public key and curve point abstraction
        let routing_info_len = additional_data_relayer_len + MAC_LENGTH + PublicKey::SIZE_COMPRESSED;
        let last_hop_len = additional_data_last_hop.map(|d| d.len()).unwrap_or(0) + 1; // end prefix length

        let header_len = last_hop_len + (max_hops - 1) * routing_info_len;
        let extended_header_len = last_hop_len + max_hops * routing_info_len;

        let mut extended_header = vec![0u8; extended_header_len];

        let mut mac =
        for idx in 0..secrets.len() {

        }

        Self {
            routing_information: Box::new(&extended_header[0..header_len]),
            mac
        }
    }
}

#[cfg(test)]
pub mod tests {

    #[test]
    fn test_filler_generate_verify() {

    }
}