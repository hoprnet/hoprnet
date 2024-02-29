use async_trait::async_trait;
use hopr_crypto_types::prelude::*;
use hopr_db_entity::ticket;
use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::*;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

use crate::db::HoprDb;
use crate::errors::Result;

#[async_trait]
pub trait HoprDbTicketOperations {
    // async fn get_tickets(channel_id: &Hash, epoch: u32);

    async fn add_ticket(&self, ticket: &AcknowledgedTicket) -> Result<()>;

    async fn remove_ticket(&self, channel_id: &Hash, epoch: u32, ticket_index: u64) -> Result<()>;

    async fn get_ticket(
        &self,
        channel_id: &Hash,
        epoch: u32,
        ticket_index: u64,
        // To be removed with https://github.com/hoprnet/hoprnet/pull/6018
        domain_separator: &Hash,
        // To be removed with https://github.com/hoprnet/hoprnet/pull/6018
        chain_keypair: &ChainKeypair,
    ) -> Result<Option<AcknowledgedTicket>>;

    // async fn update_ticket_status_range(channel_id: &Hash, epoch: u32, new_status: AcknowledgedTicketStatus);

    // async fn compact_tickets(compacted_ticket: AcknowledgedTicket);

    // async fn get_ticket_stats(channel_id: &Hash, epoch: u32);
}

#[async_trait]
impl HoprDbTicketOperations for HoprDb {
    async fn get_ticket(
        &self,
        channel_id: &Hash,
        epoch: u32,
        ticket_index: u64,
        // To be removed with https://github.com/hoprnet/hoprnet/pull/6018
        domain_separator: &Hash,
        // To be removed with https://github.com/hoprnet/hoprnet/pull/6018
        chain_keypair: &ChainKeypair,
    ) -> Result<Option<AcknowledgedTicket>> {
        // let ticket
        match ticket::Entity::find()
            .filter(ticket::Column::ChannelId.eq(channel_id.to_hex()))
            .filter(ticket::Column::ChannelEpoch.eq(u32_to_i32(epoch)))
            .filter(ticket::Column::Index.eq(u64_to_i64(ticket_index)))
            .one(&self.db)
            .await?
        {
            Some(db_ticket) => Ok(Some(
                AcknowledgedTicket::try_from_with_domain_separator_and_chain_keypair(
                    db_ticket,
                    domain_separator,
                    chain_keypair,
                )?,
            )),
            None => Ok(None),
        }
    }

    async fn remove_ticket(&self, channel_id: &Hash, epoch: u32, ticket_index: u64) -> Result<()> {
        ticket::Entity::delete_many()
            .filter(ticket::Column::ChannelId.eq(channel_id.to_hex()))
            .filter(ticket::Column::ChannelEpoch.eq(u32_to_i32(epoch)))
            .filter(ticket::Column::Index.eq(u64_to_i64(ticket_index)))
            .exec(&self.db)
            .await?;

        Ok(())
    }

    async fn add_ticket(&self, ticket: &AcknowledgedTicket) -> Result<()> {
        // ticket::Entity::insert(todo!()).exec(&self.db).await?;

        // Ok(())
        todo!()
    }
}

#[cfg(test)]
mod tests {}
