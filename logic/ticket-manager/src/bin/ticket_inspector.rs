use std::{path::PathBuf, str::FromStr};

use clap::{Parser, Subcommand};
use hopr_api::{chain::ChannelId, types::primitive::prelude::HoprBalance};
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
    /// List Channel IDs of all ticket queues in the DB.
    #[command(alias = "lc")]
    ListChannels,
    /// Delete all tickets by the Channel ID.
    #[command(alias = "dq")]
    DeleteQueue {
        /// Channel ID to delete
        #[arg(short, long)]
        channel_id: String,
    },
    /// Pretty-print all tickets in a particular queue.
    #[command(alias = "l")]
    ListTickets {
        /// Channel ID of the queue
        #[arg(short, long)]
        channel_id: String,
    },
    /// Delete all tickets in a queue up to a specified ticket matching the Channel ID and index.
    #[command(alias = "d")]
    DeleteTicket {
        /// Channel ID of the tickets
        #[arg(short, long)]
        channel_id: String,
        /// Index of the target ticket
        #[arg(short, long)]
        index: u64,
    },
    /// Print out the total sum of all ticket amounts for a given channel ID.
    #[command(alias = "t")]
    TotalValue {
        /// Channel ID of the queue
        #[arg(short, long)]
        channel_id: String,
    },
}

#[derive(Serialize, Debug, PartialEq)]
struct ChannelList {
    channels: Vec<String>,
}

#[derive(Serialize, Debug, PartialEq)]
struct DeleteQueueResult {
    channel_id: String,
    deleted_tickets_count: usize,
}

#[derive(Serialize, Debug, PartialEq)]
struct TicketList {
    channel_id: String,
    tickets: Vec<serde_json::Value>,
}

#[derive(Serialize, Debug, PartialEq)]
struct DeleteTicketResult {
    channel_id: String,
    target_index: u64,
    deleted_count: usize,
}

#[derive(Serialize, Debug, PartialEq)]
struct TotalValueResult {
    channel_id: String,
    total_sum: String,
}

#[derive(Serialize, Debug, PartialEq)]
enum CommandResult {
    ListChannels(ChannelList),
    DeleteQueue(DeleteQueueResult),
    ListTickets(TicketList),
    DeleteTicket(DeleteTicketResult),
    TotalValue(TotalValueResult),
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let mut store = RedbStore::new(&cli.db_path)?;
    let is_json = cli.json;
    let result = run_command(cli, &mut store)?;

    if is_json {
        match result {
            CommandResult::ListChannels(res) => println!("{}", serde_json::to_string_pretty(&res)?),
            CommandResult::DeleteQueue(res) => println!("{}", serde_json::to_string_pretty(&res)?),
            CommandResult::ListTickets(res) => println!("{}", serde_json::to_string_pretty(&res)?),
            CommandResult::DeleteTicket(res) => println!("{}", serde_json::to_string_pretty(&res)?),
            CommandResult::TotalValue(res) => println!("{}", serde_json::to_string_pretty(&res)?),
        }
    } else {
        match result {
            CommandResult::ListChannels(res) => {
                println!("Queues found in DB:");
                for channel in res.channels {
                    println!("  {channel}");
                }
            }
            CommandResult::DeleteQueue(res) => {
                println!(
                    "Deleted queue for channel {} with {} tickets.",
                    res.channel_id, res.deleted_tickets_count
                );
            }
            CommandResult::ListTickets(res) => {
                if res.tickets.is_empty() {
                    println!("No queue found for channel {}", res.channel_id);
                } else {
                    println!("Tickets in queue for channel {}:", res.channel_id);
                    for ticket in res.tickets {
                        println!("{ticket:#?}");
                    }
                }
            }
            CommandResult::DeleteTicket(res) => {
                println!(
                    "Deleted {} tickets up to index {} for channel {}",
                    res.deleted_count, res.target_index, res.channel_id
                );
            }
            CommandResult::TotalValue(res) => {
                println!("Total ticket value for channel {}: {}", res.channel_id, res.total_sum);
            }
        }
    }

    Ok(())
}

fn run_command(cli: Cli, store: &mut RedbStore) -> anyhow::Result<CommandResult> {
    match cli.command {
        Commands::ListChannels => {
            let channels: Vec<String> = store.iter_queues()?.map(|c| c.to_string()).collect();
            Ok(CommandResult::ListChannels(ChannelList { channels }))
        }
        Commands::DeleteQueue { channel_id } => {
            let channel = ChannelId::from_str(&channel_id)?;
            let deleted_tickets = store.delete_queue(&channel)?;
            Ok(CommandResult::DeleteQueue(DeleteQueueResult {
                channel_id: channel_id.clone(),
                deleted_tickets_count: deleted_tickets.len(),
            }))
        }
        Commands::ListTickets { channel_id } => {
            let channel = ChannelId::from_str(&channel_id)?;
            if !store.iter_queues()?.any(|c| c == channel) {
                return Ok(CommandResult::ListTickets(TicketList {
                    channel_id: channel_id.clone(),
                    tickets: vec![],
                }));
            }
            let queue = store.open_or_create_queue(&channel)?;
            let tickets: Vec<_> = queue.iter_unordered()?.collect::<Result<Vec<_>, _>>()?;

            let json_tickets: Vec<serde_json::Value> = tickets
                .iter()
                .map(|t| serde_json::to_value(t))
                .collect::<Result<Vec<_>, _>>()?;

            Ok(CommandResult::ListTickets(TicketList {
                channel_id: channel_id.clone(),
                tickets: json_tickets,
            }))
        }
        Commands::DeleteTicket { channel_id, index } => {
            let channel = ChannelId::from_str(&channel_id)?;
            if !store.iter_queues()?.any(|c| c == channel) {
                return Ok(CommandResult::DeleteTicket(DeleteTicketResult {
                    channel_id: channel_id.clone(),
                    target_index: index,
                    deleted_count: 0,
                }));
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

            Ok(CommandResult::DeleteTicket(DeleteTicketResult {
                channel_id: channel_id.clone(),
                target_index: index,
                deleted_count,
            }))
        }
        Commands::TotalValue { channel_id } => {
            let channel = ChannelId::from_str(&channel_id)?;
            if !store.iter_queues()?.any(|c| c == channel) {
                return Ok(CommandResult::TotalValue(TotalValueResult {
                    channel_id: channel_id.clone(),
                    total_sum: "0".to_string(),
                }));
            }
            let queue = store.open_or_create_queue(&channel)?;
            // total_value requires an epoch.
            // The issue description says "total sum of all ticket amounts for a given channel ID".
            // Let's iterate and sum them up manually to be sure it's "all".
            let tickets = queue.iter_unordered()?.collect::<Result<Vec<_>, _>>()?;
            let total_sum = tickets
                .iter()
                .fold(HoprBalance::from(0u64), |acc, t| acc + t.verified_ticket().amount);

            Ok(CommandResult::TotalValue(TotalValueResult {
                channel_id: channel_id.clone(),
                total_sum: total_sum.to_string(),
            }))
        }
    }
}

#[cfg(test)]
mod tests {
    use hopr_api::types::crypto::prelude::{ChainKeypair, Keypair};
    use hopr_ticket_manager::{
        TicketQueueStore,
        testing::{fill_queue, generate_owned_tickets},
    };
    use tempfile::tempdir;

    use super::*;

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

        let result = run_command(cli, &mut store)?;
        match result {
            CommandResult::ListChannels(res) => {
                assert_eq!(res.channels.len(), 2);
                assert!(res.channels.contains(&channel1.to_string()));
                assert!(res.channels.contains(&channel2.to_string()));
            }
            _ => panic!("Expected ListChannels result"),
        }

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

        let result = run_command(cli, &mut store)?;
        match result {
            CommandResult::DeleteQueue(res) => {
                assert_eq!(res.channel_id, channel.to_string());
                assert_eq!(res.deleted_tickets_count, 0); // No tickets were in the queue
            }
            _ => panic!("Expected DeleteQueue result"),
        }

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

        let result = run_command(cli, &mut store)?;
        match result {
            CommandResult::ListTickets(res) => {
                assert_eq!(res.channel_id, channel.to_string());
                assert_eq!(res.tickets.len(), 3);
            }
            _ => panic!("Expected ListTickets result"),
        }

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

        let result = run_command(cli, &mut store)?;
        match result {
            CommandResult::DeleteTicket(res) => {
                assert_eq!(res.channel_id, channel.to_string());
                assert_eq!(res.target_index, 2);
                assert_eq!(res.deleted_count, 3);
            }
            _ => panic!("Expected DeleteTicket result"),
        }

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
        let expected_sum: HoprBalance = tickets
            .iter()
            .map(|t: &hopr_api::chain::RedeemableTicket| t.verified_ticket().amount)
            .fold(HoprBalance::zero(), |acc, x| acc + x);
        fill_queue(&mut queue, tickets.into_iter())?;

        let cli = Cli {
            db_path: db_path.clone(),
            json: true,
            command: Commands::TotalValue {
                channel_id: channel.to_string(),
            },
        };

        let result = run_command(cli, &mut store)?;
        match result {
            CommandResult::TotalValue(res) => {
                assert_eq!(res.channel_id, channel.to_string());
                assert_eq!(res.total_sum, expected_sum.to_string());
            }
            _ => panic!("Expected TotalSum result"),
        }

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

        let result = run_command(cli, &mut store)?;
        match result {
            CommandResult::ListTickets(res) => {
                assert_eq!(res.channel_id, channel.to_string());
                assert!(res.tickets.is_empty());
            }
            _ => panic!("Expected ListTickets result"),
        }

        // The store should still have 0 queues because run_command should check before opening
        assert_eq!(store.iter_queues()?.count(), 0);

        Ok(())
    }
}
