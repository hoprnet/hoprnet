use async_trait::async_trait;
use futures::TryStreamExt;
use hopr_db_entity::prelude::{Ticket, TicketStatistics};
use hopr_db_entity::ticket_statistics;
use hopr_primitive_types::prelude::{Balance, BalanceType, U256};
use sea_orm::{EntityTrait, QueryOrder, QuerySelect};
use std::ops::Add;
use std::str::FromStr;
use std::time::SystemTime;

use crate::db::HoprDb;
use crate::errors::{DbError, Result};

#[async_trait]
pub trait HoprDbTicketOperations {
    async fn get_ticket_statistics(&self) -> Result<AllTicketStatistics>;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct AllTicketStatistics {
    pub last_updated: SystemTime,
    pub losing_tickets: u64,
    pub neglected_tickets: u64,
    pub neglected_value: Balance,
    pub redeemed_tickets: u64,
    pub redeemed_value: Balance,
    pub unredeemed_tickets: u64,
    pub unredeemed_value: Balance,
    pub rejected_tickets: u64,
    pub rejected_value: Balance,
}

#[async_trait]
impl HoprDbTicketOperations for HoprDb {
    async fn get_ticket_statistics(&self) -> Result<AllTicketStatistics> {
        let stats: ticket_statistics::Model = TicketStatistics::find()
            .order_by_desc(ticket_statistics::Column::LastUpdated)
            .limit(1)
            .one(&self.db)
            .await?
            .ok_or(DbError::NotFound)?;

        let (unredeemed_tickets, unredeemed_value) = Ticket::find()
            .stream(&self.db)
            .await?
            .try_fold(
                (0_u64, Balance::zero(BalanceType::HOPR)),
                |(count, amount), x| async move {
                    Ok((
                        count + 1,
                        amount.add(Balance::new(U256::from_big_endian(&x.amount), BalanceType::HOPR)),
                    ))
                },
            )
            .await?;

        Ok(AllTicketStatistics {
            last_updated: chrono::DateTime::<chrono::Utc>::from_str(&stats.last_updated)
                .map_err(|_| DbError::CorruptedData)?
                .into(),
            losing_tickets: stats.losing_tickets as u64,
            neglected_tickets: stats.neglected_tickets as u64,
            neglected_value: Balance::new(U256::from_big_endian(&stats.neglected_value), BalanceType::HOPR),
            redeemed_tickets: stats.redeemed_tickets as u64,
            redeemed_value: Balance::new(U256::from_big_endian(&stats.redeemed_value), BalanceType::HOPR),
            unredeemed_tickets,
            unredeemed_value,
            rejected_tickets: stats.rejected_tickets as u64,
            rejected_value: Balance::new(U256::from_big_endian(&stats.rejected_value), BalanceType::HOPR),
        })
    }
}
