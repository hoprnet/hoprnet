use sea_orm::Set;
use crate::ticket;
use hopr_crypto_types::prelude::*;
use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::*;
use crate::errors::DbEntityError;

impl TryFrom<&ticket::Model> for AcknowledgedTicket {
    type Error = DbEntityError;

    fn try_from(value: &ticket::Model) -> Result<Self, Self::Error> {
        let response = Response::try_from(value.response.as_ref())?;

        let ticket = VerifiedTicket::new_trusted(
            Hash::from_hex(&value.channel_id)?,
            BalanceType::HOPR.balance_bytes(&value.amount),
            U256::from_be_bytes(&value.index).as_u64(),
            value.index_offset as u32,
            win_prob_to_f64(value
                .winning_probability
                 .as_slice()
                .try_into()
                .map_err(|_| DbEntityError::ConversionError("invalid winning probability".into()))?
            ),
            U256::from_be_bytes(&value.channel_epoch).as_u32(),
            response.to_challenge().to_ethereum_challenge(),
            Signature::try_from(value.signature.as_ref())?,
            Hash::try_from(value.hash.as_slice()).map_err(|_| DbEntityError::ConversionError("invalid ticket hash".into()))?
        )
        .map_err(|_| DbEntityError::ConversionError("could not validate ticket from the db".into()))?;


        let mut ticket = AcknowledgedTicket::new(ticket, response);
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
