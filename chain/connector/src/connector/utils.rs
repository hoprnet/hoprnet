use hopr_internal_types::account::{AccountEntry, AccountType};
use crate::errors::ConnectorError;

pub(crate) fn model_to_account_entry(model: &blokli_client::api::types::Account) -> Result<AccountEntry, ConnectorError> {
    Ok(AccountEntry {
        public_key: model.packet_key.parse()?,
        chain_addr: model.chain_key.parse()?,
        key_id: (model.keyid as u32).into(),
        entry_type: if let Some(maddr) = model.multi_addresses.first() {
            AccountType::Announced {
                multiaddr: maddr.parse().map_err(|_| ConnectorError::TypeConversion)?,
                updated_block: 0,
            }
        } else {
            AccountType::NotAnnounced
        },
    })
}
