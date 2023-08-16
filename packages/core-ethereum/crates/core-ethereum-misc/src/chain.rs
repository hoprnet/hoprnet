use crate::errors::{
    CoreEthereumError::{InvalidArguments, InvalidResponseToAcknowledgement, NotAWinningTicket},
    Result,
};
use bindings::{
    hopr_announcements::{BindKeysAnnounceCall, BindKeysAnnounceSafeCall},
    hopr_channels::{
        CloseIncomingChannelCall, CompactSignature, FundChannelCall,
        InitiateOutgoingChannelClosureCall, RedeemTicketCall, RedeemableTicket, TicketData, Vrfparameters,
    },
};
use core_crypto::{
    derivation::derive_vrf_parameters,
    keypairs::{ChainKeypair, Keypair, OffchainKeypair},
    types::{Hash, VrfParameters},
};
use core_ethereum_db::traits::HoprCoreEthereumDbActions;
use core_types::{account::AccountSignature, acknowledgement::AcknowledgedTicket, channels::generate_channel_id};
use ethers::types::{Address as EthereumAddress, H160, H256, U256};
use multiaddr::Multiaddr;
use std::str::FromStr;
use utils_log::debug;
use utils_types::{
    primitives::{Address, Balance, BalanceType},
    traits::{BinarySerializable, PeerIdLike},
};

pub fn announce(
    offchain_keypair: &OffchainKeypair,
    chain_key: &Address,
    announced_multiaddr: &Multiaddr,
) -> Result<BindKeysAnnounceCall> {
    let account_sig = AccountSignature::new(offchain_keypair, chain_key);

    if let Some(ending) = announced_multiaddr.protocol_stack().last() {
        let expected: String = format!("/p2p/{}", offchain_keypair.public().to_peerid_str());
        if ending == "p2p" && !announced_multiaddr.ends_with(&Multiaddr::from_str(expected.as_str())?) {
            return Err(InvalidArguments(format!(
                "Received a multiaddr with incorrect PeerId, got {} but expected {}",
                announced_multiaddr.to_string(),
                expected
            )));
        }
    }

    let serialized_signature = account_sig.signature.to_bytes();

    Ok(BindKeysAnnounceCall {
        base_multiaddr: announced_multiaddr.to_string(),
        ed_25519_pub_key: H256::from_slice(&offchain_keypair.public().to_bytes()).into(),
        ed_25519_sig_0: H256::from_slice(&serialized_signature[0..32]).into(),
        ed_25519_sig_1: H256::from_slice(&serialized_signature[32..64]).into(),
    })
}

pub fn announce_safe(
    offchain_keypair: &OffchainKeypair,
    chain_key: &Address,
    announced_multiaddr: &Multiaddr,
) -> Result<BindKeysAnnounceSafeCall> {
    let account_sig = AccountSignature::new(offchain_keypair, chain_key);

    if let Some(ending) = announced_multiaddr.protocol_stack().last() {
        let expected: String = format!("/p2p/{}", offchain_keypair.public().to_peerid_str());
        if ending == "p2p" && !announced_multiaddr.ends_with(&Multiaddr::from_str(expected.as_str())?) {
            return Err(InvalidArguments(format!(
                "Received a multiaddr with incorrect PeerId, got {} but expected {}",
                announced_multiaddr.to_string(),
                expected
            )));
        }
    }

    let serialized_signature = account_sig.signature.to_bytes();

    Ok(BindKeysAnnounceSafeCall {
        self_: H160::from_slice(&chain_key.to_bytes()),
        base_multiaddr: announced_multiaddr.to_string(),
        ed_25519_pub_key: H256::from_slice(&offchain_keypair.public().to_bytes()).into(),
        ed_25519_sig_0: H256::from_slice(&serialized_signature[0..32]).into(),
        ed_25519_sig_1: H256::from_slice(&serialized_signature[32..64]).into(),
    })
}

pub fn fund_channel(dest: &Address, amount: &Balance) -> Result<FundChannelCall> {
    if amount.balance_type() != BalanceType::HOPR {
        return Err(InvalidArguments(
            "Invalid balance type. Expected a HOPR balance.".into(),
        ));
    }
    Ok(FundChannelCall {
        amount: amount.value().as_u128(),
        account: EthereumAddress::from_slice(&dest.to_bytes()),
    })
}

pub fn fund_channel_safe(dest: &Address, amount: &Balance) -> Result<FundChannelCall> {
    if amount.balance_type() != BalanceType::HOPR {
        return Err(InvalidArguments(
            "Invalid balance type. Expected a HOPR balance.".into(),
        ));
    }
    Ok(FundChannelCall {
        amount: amount.value().as_u128(),
        account: EthereumAddress::from_slice(&dest.to_bytes()),
    })
}

pub fn close_incoming_channel(source: &Address) -> CloseIncomingChannelCall {
    CloseIncomingChannelCall {
        source: EthereumAddress::from_slice(&source.to_bytes()),
    }
}

pub fn initiate_outgoing_channel_closure(destination: &Address) -> InitiateOutgoingChannelClosureCall {
    InitiateOutgoingChannelClosureCall {
        destination: EthereumAddress::from_slice(&destination.to_bytes()),
    }
}

pub fn redeem_ticket(chain_keypair: &ChainKeypair, acked_ticket: &AcknowledgedTicket) -> Result<RedeemTicketCall> {
    let channel_id = generate_channel_id(&acked_ticket.signer, &chain_keypair.public().to_address());

    let serialized_signature = match acked_ticket.ticket.signature {
        Some(ref signature) => signature.to_bytes(),
        None => return Err(InvalidArguments("Acknowledged ticket must be signed".into())),
    };

    // BIG TODO
    let vrf_output = derive_vrf_parameters(&acked_ticket.ticket.get_hash().into(), chain_keypair, &[])?;

    let v = vrf_output.v.to_bytes();
    let s_b = vrf_output.s_b.to_bytes();
    let h_v = vrf_output.h_v.to_bytes();

    Ok(RedeemTicketCall {
        redeemable: RedeemableTicket {
            data: TicketData {
                channel_id: channel_id.into(),
                amount: acked_ticket.ticket.amount.amount().as_u128(),
                ticket_index: acked_ticket.ticket.index.as_u64(),
                index_offset: 1u32,
                epoch: acked_ticket.ticket.channel_epoch.as_u32(),
                win_prob: acked_ticket.ticket.win_prob.as_u64(),
            },
            signature: CompactSignature {
                r: H256::from_slice(&serialized_signature[0..32]).into(),
                vs: H256::from_slice(&serialized_signature[32..64]).into(),
            },
            por_secret: U256::default(),
        },
        params: Vrfparameters {
            vx: U256::from_big_endian(&v[0..32]).into(),
            vy: U256::from_big_endian(&v[32..64]).into(),
            s: U256::from_big_endian(&vrf_output.s.to_bytes()),
            h: U256::from_big_endian(&vrf_output.h.to_bytes()),
            s_bx: U256::from_big_endian(&s_b[0..32]).into(),
            s_by: U256::from_big_endian(&s_b[32..64]).into(),
            h_vx: U256::from_big_endian(&h_v[0..32]).into(),
            h_vy: U256::from_big_endian(&h_v[32..64]).into(),
        },
    })
}

pub async fn prepare_redeem_ticket<T>(
    db: &T,
    counterparty: &Address,
    _channel_id: &Hash,
    acked_ticket: &mut AcknowledgedTicket,
) -> Result<Hash>
where
    T: HoprCoreEthereumDbActions,
{
    acked_ticket
        .verify(counterparty)
        .map_err(|e| InvalidResponseToAcknowledgement(e.to_string()))?;

    todo!("Rewrite acked ticket");

    // if !acked_ticket
    //     .ticket
    //     .is_winning(&pre_image, &acked_ticket.response, acked_ticket.ticket.win_prob)
    // {
    //     debug!(
    //         "Failed to submit ticket {}: 'Not a winning ticket.'",
    //         acked_ticket.response
    //     );

    //     return Err(NotAWinningTicket);
    // }

    Ok(Hash::default())
}

pub async fn after_redeem_ticket<T>(
    db: &mut T,
    channel_id: &Hash,
    pre_image: &Hash,
    acked_ticket: &AcknowledgedTicket,
) -> Result<()>
where
    T: HoprCoreEthereumDbActions,
{
    debug!("Successfully bumped local commitment after {pre_image} for channel {channel_id}");

    db.mark_redeemed(acked_ticket).await?;

    Ok(())
}

#[cfg(test)]
pub mod tests {
    use async_std;
    use bindings::{hopr_announcements::HoprAnnouncements, hopr_node_safe_registry::HoprNodeSafeRegistry};
    use core_crypto::{
        keypairs::{Keypair, OffchainKeypair},
        types::PublicKey,
    };
    use core_ethereum_db::db::CoreEthereumDb;
    use ethers::{
        providers::Middleware,
        types::{transaction::eip2718::TypedTransaction, Eip1559TransactionRequest},
    };
    use hex_literal::hex;
    use multiaddr::Multiaddr;
    use std::{
        path::PathBuf,
        str::FromStr,
        sync::{Arc, Mutex},
    };
    use utils_db::{db::DB, leveldb::rusty::RustyLevelDbShim};
    use utils_types::{primitives::Address, traits::BinarySerializable};

    use ethers::{
        abi::AbiEncode,
        core::utils::Anvil,
        middleware::SignerMiddleware,
        providers::{Http, Provider},
        signers::{LocalWallet, Signer},
    };

    const PRIVATE_KEY: [u8; 32] = hex!("c14b8faa0a9b8a5fa4453664996f23a7e7de606d42297d723fc4a794f375e260");
    const CHAIN_ADDR: [u8; 20] = hex!("2cDD13ddB0346E0F620C8E5826Da5d7230341c6E");

    const COUNTERPARTY_PRIV_KEY: [u8; 32] = hex!("6517e3d3245d7a111ba7be5b911adcdec7078ca5191e114e5d087a3ec936a146");

    fn create_mock_db() -> CoreEthereumDb<RustyLevelDbShim> {
        let opt = rusty_leveldb::in_memory();
        let db = rusty_leveldb::DB::open("test", opt).unwrap();

        CoreEthereumDb::new(
            DB::new(RustyLevelDbShim::new(Arc::new(Mutex::new(db)))),
            Address::from_bytes(&CHAIN_ADDR).unwrap(),
        )
    }

    #[tokio::test]
    async fn test_announce() {
        let anvil = Anvil::new()
            .path(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../../.foundry/bin/anvil"))
            .spawn();
        let wallet: LocalWallet = anvil.keys()[0].clone().into();

        let provider = Provider::<Http>::try_from(anvil.endpoint())
            .unwrap()
            .interval(std::time::Duration::from_millis(10u64));

        let client = SignerMiddleware::new(provider, wallet.with_chain_id(anvil.chain_id()));
        let client = Arc::new(client);

        let hopr_node_safe_registry = HoprNodeSafeRegistry::deploy(client.clone(), ())
            .unwrap()
            .send()
            .await
            .unwrap();

        let hopr_announcements = HoprAnnouncements::deploy(client.clone(), hopr_node_safe_registry.address())
            .unwrap()
            .send()
            .await
            .unwrap();

        let offchain_keypair = OffchainKeypair::from_secret(&PRIVATE_KEY).unwrap();
        let chain_key = PublicKey::from(anvil.keys()[0].public_key());

        let test_multiaddr = Multiaddr::from_str("/ip4/1.2.3.4/tcp/56").unwrap();

        let payload = super::announce(&offchain_keypair, &chain_key.to_address(), &test_multiaddr);

        let mut tx = TypedTransaction::Eip1559(Eip1559TransactionRequest::new());

        tx.set_data(payload.unwrap().encode().into());
        tx.set_to(hopr_announcements.address());

        let receipt = client.send_transaction(tx, None).await.unwrap().await;

        println!("{:?}", receipt);
    }

    #[async_std::test]
    async fn redeem_ticket_workflow() {
        // BIG TODO
        // let mut db = create_mock_db();

        // let counterparty_keypair = ChainKeypair::from_secret(&COUNTERPARTY_PRIV_KEY).unwrap();

        // let self_pubkey = PublicKey::from_privkey(&SELF_PRIV_KEY).unwrap();

        // let response = Response::default();
        // let challenge = response.to_challenge();

        // let channel_id = generate_channel_id(&counterparty_keypair.public().to_address(), &self_pubkey.to_address());

        // let cci = ChannelCommitmentInfo::new(100, Address::random().to_string(), channel_id.clone(), U256::zero());

        // assert!(initialize_commitment(&mut db, &SELF_PRIV_KEY, &cci).await.is_ok());

        // let mut acked_ticket = AcknowledgedTicket {
        //     response,
        //     pre_image: Hash::default(),
        //     ticket: Ticket::new(
        //         counterparty_keypair.public().to_address(),
        //         U256::zero(),
        //         U256::zero(),
        //         Balance::new(U256::zero(), BalanceType::HOPR),
        //         U256::max(),
        //         U256::zero(),
        //         &counterparty_keypair,
        //     ),
        //     signer: counterparty_keypair.public().to_address(),
        // };

        // acked_ticket
        //     .ticket
        //     .set_challenge(challenge.into(), &counterparty_keypair);

        // let pre_image = prepare_redeem_ticket(
        //     &db,
        //     &counterparty_keypair.public().to_address(),
        //     &channel_id,
        //     &mut acked_ticket,
        // )
        // .await
        // .expect("preparing ticket redemption must not fail");

        // assert!(after_redeem_ticket(&mut db, &channel_id, &pre_image, &acked_ticket)
        //     .await
        //     .is_ok());
    }
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use core_crypto::{keypairs::OffchainKeypair, types::Hash};
    use core_ethereum_db::db::wasm::Database;
    use core_types::acknowledgement::AcknowledgedTicket;
    use ethers::abi::AbiEncode;
    use js_sys::{Function, JsString};
    use multiaddr::Multiaddr;
    use std::str::FromStr;
    use utils_log::debug;
    use utils_misc::{ok_or_jserr, utils::wasm::JsResult};
    use utils_types::primitives::{Address, Balance};
    use wasm_bindgen::{prelude::*, JsValue};

    #[wasm_bindgen]
    pub async fn redeem_ticket(
        db: &Database,
        counterparty: &Address,
        channel_id: &Hash,
        acked_ticket: &mut AcknowledgedTicket,
        submit_ticket: &Function, // (counterparty: Address, ackedTicket)
    ) -> JsResult<String> {
        debug!("redeeming ticket for counterparty {counterparty} in channel {channel_id}");

        //debug!(">>> READ prepare_redeem_ticket");
        let pre_image = {
            let val = db.as_ref_counted();
            let g = val.read().await;

            super::prepare_redeem_ticket(&*g, counterparty, channel_id, acked_ticket).await?
        };
        //debug!("<<< READ prepare_redeem_ticket");

        let this = JsValue::undefined();
        debug!("submitting tx for ticket redemption in channel {channel_id} to {counterparty}");
        let res = submit_ticket.call2(
            &this,
            &<JsValue as From<Address>>::from(counterparty.to_owned()),
            &<JsValue as From<AcknowledgedTicket>>::from(acked_ticket.to_owned()),
        )?;

        let promise: js_sys::Promise = js_sys::Promise::from(res);

        let receipt = wasm_bindgen_futures::JsFuture::from(promise)
            .await
            .map_err(|e| format!("Error while trying to submit ticket {:?}", e))?;

        //debug!(">>> WRITE after_redeem_ticket");
        {
            let val = db.as_ref_counted();
            let mut g = val.write().await;
            super::after_redeem_ticket(&mut *g, channel_id, &pre_image, acked_ticket).await?;
        }
        //debug!("<<< WRITE after_redeem_ticket");
        debug!("Successfully submitted ticket {}", acked_ticket.response);

        Ok(JsString::from(receipt).as_string().unwrap_or("no receipt given".into()))
    }

    #[wasm_bindgen]
    pub fn get_announce_payload(
        offchain_keypair: &OffchainKeypair,
        chain_key: &Address,
        announced_multiaddr: &str,
    ) -> JsResult<Vec<u8>> {
        let ma = match Multiaddr::from_str(announced_multiaddr) {
            Ok(ma) => ma,
            Err(e) => return Err(JsValue::from(e.to_string())),
        };
        ok_or_jserr!(super::announce(offchain_keypair, chain_key, &ma).map(|p| p.encode()))
    }

    #[wasm_bindgen]
    pub fn get_announce_safe_payload(
        offchain_keypair: &OffchainKeypair,
        chain_key: &Address,
        announced_multiaddr: &str,
    ) -> JsResult<Vec<u8>> {
        let ma = match Multiaddr::from_str(announced_multiaddr) {
            Ok(ma) => ma,
            Err(e) => return Err(JsValue::from(e.to_string())),
        };
        ok_or_jserr!(super::announce_safe(offchain_keypair, chain_key, &ma).map(|p| p.encode()))
    }

    #[wasm_bindgen]
    pub fn get_fund_channel_payload(dest: &Address, amount: &Balance) -> JsResult<Vec<u8>> {
        ok_or_jserr!(super::fund_channel(dest, amount).map(|p| p.encode()))
    }

    #[wasm_bindgen]
    pub fn get_fund_channel_safe_payload(dest: &Address, amount: &Balance) -> JsResult<Vec<u8>> {
        ok_or_jserr!(super::fund_channel_safe(dest, amount).map(|p| p.encode()))
    }
}
