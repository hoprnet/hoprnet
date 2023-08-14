use crate::{
    commitment::{bump_commitment, find_commitment_preimage},
    errors::{
        CoreEthereumError::{InvalidResponseToAcknowledgement, NotAWinningTicket},
        Result,
    },
};
use core_crypto::types::Hash;
use core_ethereum_db::traits::HoprCoreEthereumDbActions;
use core_types::acknowledgement::AcknowledgedTicket;
use utils_log::debug;
use utils_types::primitives::Address;

pub async fn prepare_redeem_ticket<T>(
    db: &T,
    counterparty: &Address,
    channel_id: &Hash,
    acked_ticket: &mut AcknowledgedTicket,
) -> Result<Hash>
where
    T: HoprCoreEthereumDbActions,
{
    acked_ticket
        .verify(counterparty)
        .map_err(|e| InvalidResponseToAcknowledgement(e.to_string()))?;

    let pre_image = find_commitment_preimage(db, channel_id).await?;

    acked_ticket.set_preimage(&pre_image);
    debug!(
        "Set preImage {pre_image} for ticket {} in channel to {counterparty}",
        acked_ticket.response
    );

    if !acked_ticket
        .ticket
        .is_winning(&pre_image, &acked_ticket.response, acked_ticket.ticket.win_prob)
    {
        debug!(
            "Failed to submit ticket {}: 'Not a winning ticket.'",
            acked_ticket.response
        );

        return Err(NotAWinningTicket);
    }

    Ok(pre_image)
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
    // bump commitment when on-chain ticket redemption is successful
    // FIXME: bump commitment can fail if channel runs out of commitments
    bump_commitment(db, channel_id, pre_image).await?;
    debug!("Successfully bumped local commitment after {pre_image} for channel {channel_id}");

    db.mark_redeemed(acked_ticket).await?;

    Ok(())
}

#[cfg(test)]
pub mod tests {
    use crate::{
        chain::{after_redeem_ticket, prepare_redeem_ticket},
        commitment::{initialize_commitment, ChannelCommitmentInfo},
    };
    use async_std;
    use core_crypto::keypairs::{ChainKeypair, Keypair};
    use core_crypto::types::{Hash, PublicKey, Response};
    use core_ethereum_db::db::CoreEthereumDb;
    use core_types::acknowledgement::AcknowledgedTicket;
    use core_types::channels::{generate_channel_id, Ticket};
    use hex_literal::hex;
    use std::sync::{Arc, Mutex};
    use utils_db::{db::DB, leveldb::rusty::RustyLevelDbShim};
    use utils_types::primitives::BalanceType;
    use utils_types::primitives::{Address, Balance, U256};

    const SELF_PRIV_KEY: [u8; 32] = hex!("492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775");
    const COUNTERPARTY_PRIV_KEY: [u8; 32] = hex!("6517e3d3245d7a111ba7be5b911adcdec7078ca5191e114e5d087a3ec936a146");

    fn create_mock_db() -> CoreEthereumDb<RustyLevelDbShim> {
        let opt = rusty_leveldb::in_memory();
        let db = rusty_leveldb::DB::open("test", opt).unwrap();

        CoreEthereumDb::new(
            DB::new(RustyLevelDbShim::new(Arc::new(Mutex::new(db)))),
            PublicKey::from_privkey(&SELF_PRIV_KEY).unwrap().to_address(),
        )
    }

    #[async_std::test]
    async fn redeem_ticket_workflow() {
        let mut db = create_mock_db();

        let counterparty_keypair = ChainKeypair::from_secret(&COUNTERPARTY_PRIV_KEY).unwrap();

        let self_pubkey = PublicKey::from_privkey(&SELF_PRIV_KEY).unwrap();

        let response = Response::default();
        let challenge = response.to_challenge();

        let channel_id = generate_channel_id(&counterparty_keypair.public().to_address(), &self_pubkey.to_address());

        let cci = ChannelCommitmentInfo::new(100, Address::random().to_string(), channel_id.clone(), U256::zero());

        assert!(initialize_commitment(&mut db, &SELF_PRIV_KEY, &cci).await.is_ok());

        let mut acked_ticket = AcknowledgedTicket {
            response,
            pre_image: Hash::default(),
            ticket: Ticket::new(
                counterparty_keypair.public().to_address(),
                U256::zero(),
                U256::zero(),
                Balance::new(U256::zero(), BalanceType::HOPR),
                U256::max(),
                U256::zero(),
                &counterparty_keypair,
            ),
            signer: counterparty_keypair.public().to_address(),
        };

        acked_ticket
            .ticket
            .set_challenge(challenge.into(), &counterparty_keypair);

        let pre_image = prepare_redeem_ticket(
            &db,
            &counterparty_keypair.public().to_address(),
            &channel_id,
            &mut acked_ticket,
        )
        .await
        .expect("preparing ticket redemption must not fail");

        assert!(after_redeem_ticket(&mut db, &channel_id, &pre_image, &acked_ticket)
            .await
            .is_ok());
    }
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use core_crypto::types::Hash;
    use core_ethereum_db::db::wasm::Database;
    use core_types::acknowledgement::AcknowledgedTicket;
    use js_sys::{Function, JsString};
    use utils_log::debug;
    use utils_misc::utils::wasm::JsResult;
    use utils_types::primitives::Address;
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
}
