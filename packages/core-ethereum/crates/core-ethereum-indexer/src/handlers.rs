use crate::errors::Result;
use bindings::hopr_announcements::HoprAnnouncementsEvents;
use core_ethereum_db::traits::HoprCoreEthereumDbActions;
use ethers::{contract::EthLogDecode, core::abi::RawLog};

pub async fn on_announce<T>(db: &T, log: RawLog) -> Result<()>
where
    T: HoprCoreEthereumDbActions,
{
    HoprAnnouncementsEvents::decode_log(&log)?;
    // hopr_announcements::HoprAnnouncements::events(&self)

    todo!()
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use core_ethereum_db::db::wasm::Database;
    use ethers::core::abi::RawLog;
    use hex::decode_to_slice;
    use js_sys::{Array, Uint8Array};
    use wasm_bindgen::prelude::*;
    use wasm_bindgen_futures;

    #[wasm_bindgen]
    pub async fn on_announcement_event(db: &Database, topics: Array, data: String) {
        let mut decoded_data = Vec::with_capacity(data.len() * 2);
        decode_to_slice(data, &mut decoded_data);

        let val = db.as_ref_counted();
        let g = val.read().await;

        super::on_announce(
            &*g,
            RawLog {
                topics: topics
                    .to_vec()
                    .iter()
                    .map(|topic| {
                        let mut decoded = [0u8; 32];

                        decode_to_slice(Uint8Array::from(topic.to_owned()).to_vec(), &mut decoded);

                        decoded.into()
                    })
                    .collect(),
                data: decoded_data,
            },
        )
        .await;
    }
}
