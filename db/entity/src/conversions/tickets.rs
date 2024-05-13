use crate::errors::DbEntityError;
use crate::ticket;
use hopr_crypto_types::prelude::*;
use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::*;
use sea_orm::Set;

impl TryFrom<&ticket::Model> for AcknowledgedTicket {
    type Error = DbEntityError;

    fn try_from(value: &ticket::Model) -> Result<Self, Self::Error> {
        let response = Response::try_from(value.response.as_ref())?;

        let mut ticket = TicketBuilder::default()
            .channel_id(Hash::from_hex(&value.channel_id)?)
            .amount(U256::from_be_bytes(&value.amount))
            .index(U256::from_be_bytes(&value.index).as_u64())
            .index_offset(value.index_offset as u32)
            .win_prob_encoded(
                value
                    .winning_probability
                    .as_slice()
                    .try_into()
                    .map_err(|_| DbEntityError::ConversionError("invalid winning probability".into()))?,
            )
            .channel_epoch(U256::from_be_bytes(&value.channel_epoch).as_u32())
            .challenge(response.to_challenge().to_ethereum_challenge())
            .signature(
                value
                    .signature
                    .as_slice()
                    .try_into()
                    .map_err(|_| DbEntityError::ConversionError("invalid signature format".into()))?,
            )
            .build_verified(
                value
                    .hash
                    .as_slice()
                    .try_into()
                    .map_err(|_| DbEntityError::ConversionError("invalid ticket hash".into()))?,
            )
            .map_err(|e| DbEntityError::ConversionError(format!("invalid ticket in the db: {e}")))?
            .into_acknowledged(response);

        ticket.status = AcknowledgedTicketStatus::try_from(value.state as u8)
            .map_err(|_| DbEntityError::ConversionError("invalid ticket state".into()))?;

        Ok(ticket)
    }
}

impl TryFrom<ticket::Model> for AcknowledgedTicket {
    type Error = DbEntityError;

    fn try_from(value: ticket::Model) -> Result<Self, Self::Error> {
        Self::try_from(&value)
    }
}

impl From<AcknowledgedTicket> for ticket::ActiveModel {
    fn from(value: AcknowledgedTicket) -> Self {
        ticket::ActiveModel {
            channel_id: Set(value.verified_ticket().channel_id.to_hex()),
            amount: Set(value.verified_ticket().amount.amount().to_be_bytes().to_vec()),
            index: Set(value.verified_ticket().index.to_be_bytes().to_vec()),
            index_offset: Set(value.verified_ticket().index_offset as i32),
            winning_probability: Set(value.verified_ticket().encoded_win_prob.to_vec()),
            channel_epoch: Set(U256::from(value.verified_ticket().channel_epoch).to_be_bytes().to_vec()),
            signature: Set(value.verified_ticket().signature.unwrap().as_ref().to_vec()),
            response: Set(value.response.as_ref().to_vec()),
            state: Set(value.status as u8 as i32),
            hash: Set(value.ticket.verified_hash().as_ref().to_vec()),
            ..Default::default()
        }
    }
}
