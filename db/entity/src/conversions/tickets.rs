use crate::errors::DbEntityError;
use crate::ticket;
use hopr_crypto_types::prelude::*;
use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::*;
use sea_orm::Set;

/// TODO: implement as TryFrom trait once https://github.com/hoprnet/hoprnet/pull/6018 is merged
pub fn model_to_acknowledged_ticket(
    db_ticket: &ticket::Model,
    domain_separator: Hash,
    chain_keypair: &ChainKeypair,
) -> crate::errors::Result<AcknowledgedTicket> {
    let response = Response::from_bytes(&db_ticket.response)?;

    // To be refactored with https://github.com/hoprnet/hoprnet/pull/6018
    let mut ticket = Ticket::default();
    ticket.channel_id = Hash::from_hex(&db_ticket.channel_id)?;
    ticket.amount = BalanceType::HOPR.balance_bytes(&db_ticket.amount);
    ticket.index = U256::from_be_bytes(&db_ticket.index).as_u64();
    ticket.index_offset = db_ticket.index_offset as u32;
    ticket.channel_epoch = U256::from_be_bytes(&db_ticket.channel_epoch).as_u32();
    ticket.encoded_win_prob = db_ticket
        .winning_probability
        .clone()
        .try_into()
        .map_err(|_| DbEntityError::ConversionError("invalid winning probability".into()))?;
    ticket.challenge = response.to_challenge().to_ethereum_challenge();
    ticket.signature = Some(Signature::from_bytes(&db_ticket.signature)?);

    let signer = ticket.recover_signer(&domain_separator)?.to_address();

    let mut ticket = AcknowledgedTicket::new(
        ticket,
        response,
        signer,
        chain_keypair,
        &domain_separator,
    )?;
    ticket.status = AcknowledgedTicketStatus::try_from(db_ticket.state as u8).map_err(|_| DbEntityError::ConversionError("invalid ticket state".into()))?;

    Ok(ticket)
}

impl From<AcknowledgedTicket> for ticket::ActiveModel {
    fn from(value: AcknowledgedTicket) -> Self {
        ticket::ActiveModel {
            channel_id: Set(value.ticket.channel_id.to_hex()),
            amount: Set(value.ticket.amount.amount().to_be_bytes().to_vec()),
            index: Set(value.ticket.index.to_be_bytes().to_vec()),
            index_offset: Set(value.ticket.index_offset as i32),
            winning_probability: Set(value.ticket.encoded_win_prob.to_vec()),
            channel_epoch: Set(U256::from(value.ticket.channel_epoch).to_be_bytes().to_vec()),
            signature: Set(value.ticket.signature.unwrap().to_bytes().to_vec()),
            response: Set(value.response.to_bytes().to_vec()),
            state: Set(value.status as u8 as i32),
            ..Default::default()
        }
    }
}
