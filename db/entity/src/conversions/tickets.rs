use hopr_crypto_types::prelude::*;
use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::*;
use sea_orm::Set;

use crate::{errors::DbEntityError, ticket};

impl TryFrom<&ticket::Model> for RedeemableTicket {
    type Error = DbEntityError;

    fn try_from(value: &ticket::Model) -> Result<Self, Self::Error> {
        let response = Response::try_from(value.response.as_ref())?;

        let ticket = TicketBuilder::default()
            .counterparty(Address::from_hex(&value.counterparty)?)
            .amount(U256::from_be_bytes(&value.amount))
            .index(u64::from_be_bytes(value.index.clone().try_into().map_err(|_| {
                DbEntityError::ConversionError("invalid ticket index".into())
            })?))
            .win_prob(
                value
                    .winning_probability
                    .as_slice()
                    .try_into()
                    .map_err(|_| DbEntityError::ConversionError("invalid winning probability".into()))?,
            )
            .channel_epoch(u32::from_be_bytes(
                value
                    .channel_epoch
                    .clone()
                    .try_into()
                    .map_err(|_| DbEntityError::ConversionError("invalid channel epoch".into()))?,
            ))
            .challenge(response.to_challenge()?)
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
            .map_err(|e| DbEntityError::ConversionError(format!("invalid ticket in the db: {e}")))?;

        Ok(RedeemableTicket {
            ticket,
            response,
            vrf_params: VrfParameters::try_from(value.vrf_params.as_slice())?,
            channel_dst: Hash::try_from(value.channel_dst.as_slice())?,
        })
    }
}

impl TryFrom<ticket::Model> for RedeemableTicket {
    type Error = DbEntityError;

    fn try_from(value: ticket::Model) -> Result<Self, Self::Error> {
        Self::try_from(&value)
    }
}

impl From<RedeemableTicket> for ticket::ActiveModel {
    fn from(value: RedeemableTicket) -> Self {
        ticket::ActiveModel {
            channel_id: Set(hex::encode(value.ticket.channel_id())), // serialize without 0x prefix
            counterparty: Set(hex::encode(value.verified_ticket().counterparty)), // serialize without 0x prefix
            amount: Set(value.verified_ticket().amount.amount().to_be_bytes().to_vec()),
            index: Set(value.verified_ticket().index.to_be_bytes().to_vec()),
            winning_probability: Set(value.verified_ticket().encoded_win_prob.to_vec()),
            channel_epoch: Set(value.verified_ticket().channel_epoch.to_be_bytes().to_vec()),
            signature: Set(value.verified_ticket().signature.unwrap().as_ref().to_vec()),
            response: Set(value.response.as_ref().to_vec()),
            vrf_params: Set(value.vrf_params.into_encoded().to_vec()),
            channel_dst: Set(value.channel_dst.as_ref().to_vec()),
            hash: Set(value.ticket.verified_hash().as_ref().to_vec()),
            ..Default::default() // State is always set to 0 = Untouched
        }
    }
}
