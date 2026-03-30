use std::path::PathBuf;
use std::str::FromStr;
use clap::{Parser, Subcommand};
use hopr_api::chain::ChannelId;
use hopr_api::types::primitive::prelude::HoprBalance;
use hopr_ticket_manager::{RedbStore, TicketQueue, TicketQueueStore};
use serde::Serialize;

#[derive(Parser)]
#[command(name = "ticket-inspector")]
#[command(about = "CLI tool to inspect and manipulate hopr-ticket-manager redb database", long_about = None)]
struct Cli {
    /// Path to the redb database file
    #[arg(short, long, value_name = "FILE")]
    db_path: PathBuf,

    /// Output in JSON format
    #[arg(short, long)]
    json: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Enumerate ChannelIds of all ticket queues in the DB
    ListChannels,
    /// Delete a queue by its Channel ID
    DeleteQueue {
        /// Channel ID to delete
        channel_id: String,
    },
    /// Pretty-print all tickets in a particular queue
    ListTickets {
        /// Channel ID of the queue
        channel_id: String,
    },
    /// Delete all tickets in a queue up to a specified ticket matching the Channel ID and index
    DeleteTicket {
        /// Channel ID of the tickets
        channel_id: String,
        /// Index of the target ticket
        index: u64,
    },
    /// Print out the total sum of all ticket amounts for a given channel ID
    TotalSum {
        /// Channel ID of the queue
        channel_id: String,
    },
}

#[derive(Serialize)]
struct ChannelList {
    channels: Vec<String>,
}

#[derive(Serialize)]
struct DeleteQueueResult {
    channel_id: String,
    deleted_tickets_count: usize,
}

#[derive(Serialize)]
struct TicketList {
    channel_id: String,
    tickets: Vec<serde_json::Value>,
}

#[derive(Serialize)]
struct DeleteTicketResult {
    channel_id: String,
    target_index: u64,
    deleted_count: usize,
}

#[derive(Serialize)]
struct TotalSumResult {
    channel_id: String,
    total_sum: String,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let mut store = RedbStore::new(&cli.db_path)?;
    run_command(cli, &mut store)
}

fn run_command(cli: Cli, store: &mut RedbStore) -> anyhow::Result<()> {
    match cli.command {
        Commands::ListChannels => {
            let channels: Vec<String> = store.iter_queues()?.map(|c| c.to_string()).collect();
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&ChannelList { channels })?);
            } else {
                println!("Queues found in DB:");
                for channel in channels {
                    println!("  {}", channel);
                }
            }
        }
        Commands::DeleteQueue { channel_id } => {
            let channel = ChannelId::from_str(&channel_id)?;
            let deleted_tickets = store.delete_queue(&channel)?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&DeleteQueueResult {
                    channel_id: channel_id.clone(),
                    deleted_tickets_count: deleted_tickets.len(),
                })?);
            } else {
                println!("Deleted queue for channel {} with {} tickets.", channel_id, deleted_tickets.len());
            }
        }
        Commands::ListTickets { channel_id } => {
            let channel = ChannelId::from_str(&channel_id)?;
            if !store.iter_queues()?.any(|c| c == channel) {
                if cli.json {
                    println!("{}", serde_json::to_string_pretty(&TicketList {
                        channel_id: channel_id.clone(),
                        tickets: vec![],
                    })?);
                } else {
                    println!("No queue found for channel {}", channel_id);
                }
                return Ok(());
            }
            let queue = store.open_or_create_queue(&channel)?;
            let tickets: Vec<_> = queue.iter_unordered()?.collect::<Result<Vec<_>, _>>()?;
            
            if cli.json {
                let json_tickets: Vec<serde_json::Value> = tickets.iter()
                    .map(|t| serde_json::to_value(t))
                    .collect::<Result<Vec<_>, _>>()?;
                println!("{}", serde_json::to_string_pretty(&TicketList {
                    channel_id: channel_id.clone(),
                    tickets: json_tickets,
                })?);
            } else {
                println!("Tickets in queue for channel {}:", channel_id);
                for ticket in tickets {
                    println!("{:#?}", ticket);
                }
            }
        }
        Commands::DeleteTicket { channel_id, index } => {
            let channel = ChannelId::from_str(&channel_id)?;
            if !store.iter_queues()?.any(|c| c == channel) {
                if cli.json {
                    println!("{}", serde_json::to_string_pretty(&DeleteTicketResult {
                        channel_id: channel_id.clone(),
                        target_index: index,
                        deleted_count: 0,
                    })?);
                } else {
                    println!("No queue found for channel {}", channel_id);
                }
                return Ok(());
            }
            let mut queue = store.open_or_create_queue(&channel)?;
            
            let mut deleted_count = 0;
            while let Some(ticket) = queue.peek()? {
                if ticket.verified_ticket().index <= index {
                    queue.pop()?;
                    deleted_count += 1;
                } else {
                    break;
                }
            }

            if cli.json {
                println!("{}", serde_json::to_string_pretty(&DeleteTicketResult {
                    channel_id: channel_id.clone(),
                    target_index: index,
                    deleted_count,
                })?);
            } else {
                println!("Deleted {} tickets up to index {} for channel {}", deleted_count, index, channel_id);
            }
        }
        Commands::TotalSum { channel_id } => {
            let channel = ChannelId::from_str(&channel_id)?;
            if !store.iter_queues()?.any(|c| c == channel) {
                if cli.json {
                    println!("{}", serde_json::to_string_pretty(&TotalSumResult {
                        channel_id: channel_id.clone(),
                        total_sum: "0".to_string(),
                    })?);
                } else {
                    println!("No queue found for channel {}", channel_id);
                }
                return Ok(());
            }
            let queue = store.open_or_create_queue(&channel)?;
            // total_value requires an epoch. 
            // The issue description says "total sum of all ticket amounts for a given channel ID".
            // Let's iterate and sum them up manually to be sure it's "all".
            let tickets = queue.iter_unordered()?.collect::<Result<Vec<_>, _>>()?;
            let total_sum = tickets.iter().fold(HoprBalance::from(0u64), |acc, t| {
                acc + t.verified_ticket().amount
            });

            if cli.json {
                println!("{}", serde_json::to_string_pretty(&TotalSumResult {
                    channel_id: channel_id.clone(),
                    total_sum: total_sum.to_string(),
                })?);
            } else {
                println!("Total sum for channel {}: {}", channel_id, total_sum);
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use hopr_ticket_manager::TicketQueueStore;
    use tempfile::tempdir;
    use hopr_ticket_manager::testing::{generate_owned_tickets, fill_queue};
    use hopr_api::types::crypto::prelude::{ChainKeypair, Keypair};

    #[test]
    fn test_list_channels() -> anyhow::Result<()> {
        let dir = tempdir()?;
        let db_path = dir.path().join("test_list.db");
        let mut store = RedbStore::new(&db_path)?;
        
        let channel1 = ChannelId::from([1u8; 32]);
        let channel2 = ChannelId::from([2u8; 32]);
        
        store.open_or_create_queue(&channel1)?;
        store.open_or_create_queue(&channel2)?;
        
        let cli = Cli {
            db_path: db_path.clone(),
            json: true,
            command: Commands::ListChannels,
        };
        
        // We can't easily capture stdout, but we can verify it doesn't crash
        run_command(cli, &mut store)?;
        
        Ok(())
    }

    #[test]
    fn test_delete_queue() -> anyhow::Result<()> {
        let dir = tempdir()?;
        let db_path = dir.path().join("test_delete_queue.db");
        let mut store = RedbStore::new(&db_path)?;
        
        let channel = ChannelId::from([1u8; 32]);
        store.open_or_create_queue(&channel)?;
        
        assert_eq!(store.iter_queues()?.count(), 1);
        
        let cli = Cli {
            db_path: db_path.clone(),
            json: true,
            command: Commands::DeleteQueue {
                channel_id: channel.to_string(),
            },
        };
        
        run_command(cli, &mut store)?;
        
        assert_eq!(store.iter_queues()?.count(), 0);
        
        Ok(())
    }

    #[test]
    fn test_list_tickets() -> anyhow::Result<()> {
        let dir = tempdir()?;
        let db_path = dir.path().join("test_list_tickets.db");
        let mut store = RedbStore::new(&db_path)?;
        
        let channel = ChannelId::from([1u8; 32]);
        let mut queue = store.open_or_create_queue(&channel)?;
        
        let src = ChainKeypair::random();
        let dst = ChainKeypair::random();
        let tickets = generate_owned_tickets(&src, &dst, 3, 1..=1)?;
        fill_queue(&mut queue, tickets.into_iter())?;
        
        let cli = Cli {
            db_path: db_path.clone(),
            json: true,
            command: Commands::ListTickets {
                channel_id: channel.to_string(),
            },
        };
        
        run_command(cli, &mut store)?;
        
        Ok(())
    }

    #[test]
    fn test_delete_ticket() -> anyhow::Result<()> {
        let dir = tempdir()?;
        let db_path = dir.path().join("test_delete_ticket.db");
        let mut store = RedbStore::new(&db_path)?;
        
        let channel = ChannelId::from([1u8; 32]);
        let mut queue = store.open_or_create_queue(&channel)?;
        
        let src = ChainKeypair::random();
        let dst = ChainKeypair::random();
        let tickets = generate_owned_tickets(&src, &dst, 5, 1..=1)?;
        fill_queue(&mut queue, tickets.into_iter())?;
        
        assert_eq!(queue.len()?, 5);
        
        let cli = Cli {
            db_path: db_path.clone(),
            json: true,
            command: Commands::DeleteTicket {
                channel_id: channel.to_string(),
                index: 2,
            },
        };
        
        run_command(cli, &mut store)?;
        
        // Should have deleted tickets with index 0, 1, 2
        let queue = store.open_or_create_queue(&channel)?;
        assert_eq!(queue.len()?, 2);
        
        Ok(())
    }

    #[test]
    fn test_total_sum() -> anyhow::Result<()> {
        let dir = tempdir()?;
        let db_path = dir.path().join("test_total_sum.db");
        let mut store = RedbStore::new(&db_path)?;
        
        let channel = ChannelId::from([1u8; 32]);
        let mut queue = store.open_or_create_queue(&channel)?;
        
        let src = ChainKeypair::random();
        let dst = ChainKeypair::random();
        let tickets = generate_owned_tickets(&src, &dst, 3, 1..=1)?;
        let _expected_sum: HoprBalance = tickets.iter().map(|t| t.verified_ticket().amount).fold(HoprBalance::zero(), |acc, x| acc + x);
        fill_queue(&mut queue, tickets.into_iter())?;
        
        let cli = Cli {
            db_path: db_path.clone(),
            json: true,
            command: Commands::TotalSum {
                channel_id: channel.to_string(),
            },
        };
        
        run_command(cli, &mut store)?;
        
        Ok(())
    }

    #[test]
    fn test_open_or_create_queue_inspected() -> anyhow::Result<()> {
        let dir = tempdir()?;
        let db_path = dir.path().join("test_inspected.db");
        let mut store = RedbStore::new(&db_path)?;
        
        let channel = ChannelId::from([4u8; 32]);
        
        // Ensure no queues exist initially
        assert_eq!(store.iter_queues()?.count(), 0);
        
        // Test with ListTickets command on non-existent channel
        let cli = Cli {
            db_path: db_path.clone(),
            json: false,
            command: Commands::ListTickets {
                channel_id: channel.to_string(),
            },
        };
        
        run_command(cli, &mut store)?;
        
        // The store should still have 0 queues because run_command should check before opening
        assert_eq!(store.iter_queues()?.count(), 0);
        
        Ok(())
    }
}
