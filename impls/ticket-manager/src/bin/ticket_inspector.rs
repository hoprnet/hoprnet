use std::{fmt, path::PathBuf};

use clap::{Parser, Subcommand, ValueEnum};
use hopr_api::{chain::ChannelId, types::primitive::prelude::HoprBalance};
use hopr_ticket_manager::{RedbStore, TicketQueue, TicketQueueStore};
#[cfg(feature = "serde")]
use serde::Serialize;
use strum::{Display, EnumString, VariantNames};

#[derive(Parser)]
#[command(name = "ticket-inspector")]
#[command(about = "CLI tool to inspect and manipulate HOPR redeemable tickets database", long_about = None)]
struct Cli {
    /// Path to the database file
    #[arg(long, short, value_name = "FILE")]
    db_file: PathBuf,
    /// Output format
    #[arg(short, long, value_enum, default_value_t = OutputFormat::Plain)]
    format: OutputFormat,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug, Display, EnumString, VariantNames)]
#[strum(serialize_all = "lowercase")]
enum OutputFormat {
    /// Plain text output
    Plain,
    /// JSON output
    #[cfg(feature = "serde")]
    Json,
}

#[derive(Subcommand)]
enum Commands {
    /// List Channel IDs of all ticket queues in the DB.
    #[command(short_flag = 'c')]
    ListChannels,
    /// Delete all tickets by the Channel ID.
    #[command(alias = "dq")]
    DeleteQueue {
        /// Channel ID to delete
        #[arg(short, long)]
        channel_id: ChannelId,
    },
    /// Display all tickets in a particular queue in-order.
    #[command(short_flag = 'l')]
    ListTickets {
        /// Channel ID of the queue
        #[arg(short, long)]
        channel_id: ChannelId,
    },
    /// Delete all tickets in a queue up to a specified ticket matching the Channel ID and index.
    #[command(short_flag = 'e')]
    DeleteTicket {
        /// Channel ID of the tickets
        #[arg(short, long)]
        channel_id: ChannelId,
        /// Index of the target ticket
        #[arg(short, long)]
        index: u64,
    },
    /// Print out the total sum of all ticket amounts for a given channel ID.
    #[command(short_flag = 't')]
    TotalValue {
        /// Channel ID of the queue
        #[arg(short, long)]
        channel_id: ChannelId,
    },
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, PartialEq)]
struct ChannelList {
    channels: Vec<String>,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, PartialEq)]
struct DeleteQueueResult {
    channel_id: ChannelId,
    deleted_tickets_count: usize,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, PartialEq)]
struct TicketList {
    channel_id: ChannelId,
    #[cfg(feature = "serde")]
    tickets: Vec<serde_json::Value>,
    #[cfg(not(feature = "serde"))]
    tickets: Vec<String>,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, PartialEq)]
struct DeleteTicketResult {
    channel_id: ChannelId,
    target_index: u64,
    deleted_count: usize,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, PartialEq)]
struct TotalValueResult {
    channel_id: ChannelId,
    total_sum: String,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, PartialEq)]
enum CommandResult {
    ListChannels(ChannelList),
    DeleteQueue(DeleteQueueResult),
    ListTickets(TicketList),
    DeleteTicket(DeleteTicketResult),
    TotalValue(TotalValueResult),
}

impl fmt::Display for CommandResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommandResult::ListChannels(res) => {
                writeln!(f, "Queues found in DB:")?;
                for channel in &res.channels {
                    writeln!(f, "  {channel}")?;
                }
            }
            CommandResult::DeleteQueue(res) => {
                write!(
                    f,
                    "Deleted queue for channel {} with {} tickets.",
                    res.channel_id, res.deleted_tickets_count
                )?;
            }
            CommandResult::ListTickets(res) => {
                if res.tickets.is_empty() {
                    write!(f, "No queue found for channel {}", res.channel_id)?;
                } else {
                    writeln!(f, "Tickets in queue for channel {}:", res.channel_id)?;
                    for (i, ticket) in res.tickets.iter().enumerate() {
                        if i > 0 {
                            writeln!(f)?;
                        }
                        write!(f, "{ticket:#?}")?;
                    }
                }
            }
            CommandResult::DeleteTicket(res) => {
                write!(
                    f,
                    "Deleted {} tickets up to index {} for channel {}",
                    res.deleted_count, res.target_index, res.channel_id
                )?;
            }
            CommandResult::TotalValue(res) => {
                write!(
                    f,
                    "Total ticket value for channel {}: {}",
                    res.channel_id, res.total_sum
                )?;
            }
        }
        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    if !cli.db_file.is_file() {
        return Err(anyhow::anyhow!(
            "Database file does not exist: {}",
            cli.db_file.display()
        ));
    }

    let mut store = RedbStore::new(&cli.db_file)?;
    let format = cli.format;
    let result = run_command(cli, &mut store)?;

    match format {
        #[cfg(feature = "serde")]
        OutputFormat::Json => match result {
            CommandResult::ListChannels(res) => println!("{}", serde_json::to_string_pretty(&res)?),
            CommandResult::DeleteQueue(res) => println!("{}", serde_json::to_string_pretty(&res)?),
            CommandResult::ListTickets(res) => println!("{}", serde_json::to_string_pretty(&res)?),
            CommandResult::DeleteTicket(res) => println!("{}", serde_json::to_string_pretty(&res)?),
            CommandResult::TotalValue(res) => println!("{}", serde_json::to_string_pretty(&res)?),
        },
        OutputFormat::Plain => {
            println!("{result}");
        }
    }

    Ok(())
}

fn run_command(cli: Cli, store: &mut impl TicketQueueStore) -> anyhow::Result<CommandResult> {
    match cli.command {
        Commands::ListChannels => {
            let mut channels: Vec<String> = store.iter_queues()?.map(|c| c.to_string()).collect();
            channels.sort();
            Ok(CommandResult::ListChannels(ChannelList { channels }))
        }
        Commands::DeleteQueue { channel_id } => {
            let deleted_tickets = store.delete_queue(&channel_id)?;
            Ok(CommandResult::DeleteQueue(DeleteQueueResult {
                channel_id,
                deleted_tickets_count: deleted_tickets.len(),
            }))
        }
        Commands::ListTickets { channel_id } => {
            if !store.iter_queues()?.any(|c| c == channel_id) {
                return Ok(CommandResult::ListTickets(TicketList {
                    channel_id,
                    tickets: vec![],
                }));
            }
            let queue = store.open_or_create_queue(&channel_id)?;
            let mut tickets = queue.iter_unordered()?.collect::<Result<Vec<_>, _>>()?;
            tickets.sort();

            #[cfg(feature = "serde")]
            let tickets: Vec<serde_json::Value> = tickets
                .into_iter()
                .map(serde_json::to_value)
                .collect::<Result<Vec<_>, _>>()?;

            #[cfg(not(feature = "serde"))]
            let tickets: Vec<String> = tickets.into_iter().map(|t| format!("{:?}", t)).collect();

            Ok(CommandResult::ListTickets(TicketList { channel_id, tickets }))
        }
        Commands::DeleteTicket { channel_id, index } => {
            if !store.iter_queues()?.any(|c| c == channel_id) {
                return Ok(CommandResult::DeleteTicket(DeleteTicketResult {
                    channel_id,
                    target_index: index,
                    deleted_count: 0,
                }));
            }
            let mut queue = store.open_or_create_queue(&channel_id)?;

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
                channel_id,
                target_index: index,
                deleted_count,
            }))
        }
        Commands::TotalValue { channel_id } => {
            if !store.iter_queues()?.any(|c| c == channel_id) {
                return Ok(CommandResult::TotalValue(TotalValueResult {
                    channel_id,
                    total_sum: "0".to_string(),
                }));
            }
            let queue = store.open_or_create_queue(&channel_id)?;
            let total_sum: HoprBalance = queue
                .iter_unordered()?
                .map(|r| r.map_err(|e| anyhow::anyhow!("error reading ticket: {e}")))
                .try_fold(HoprBalance::zero(), |acc, t| {
                    anyhow::Ok(acc + t?.verified_ticket().amount)
                })?;

            Ok(CommandResult::TotalValue(TotalValueResult {
                channel_id,
                total_sum: total_sum.to_string(),
            }))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::ops::RangeBounds;

    use hopr_api::{
        chain::{RedeemableTicket, WinningProbability},
        types::{
            crypto::prelude::{ChainKeypair, Challenge, HalfKey, Keypair, Response},
            crypto_random::Randomizable,
            internal::prelude::TicketBuilder,
        },
    };
    use hopr_ticket_manager::TicketQueueStore;
    use tempfile::tempdir;

    use super::*;

    pub fn generate_owned_tickets(
        issuer: &ChainKeypair,
        recipient: &ChainKeypair,
        count: usize,
        epochs: impl RangeBounds<u32> + Iterator<Item = u32>,
    ) -> anyhow::Result<Vec<RedeemableTicket>> {
        let mut tickets = Vec::new();
        for epoch in epochs {
            for i in 0..count {
                let hk1 = HalfKey::random();
                let hk2 = HalfKey::random();

                let ticket = TicketBuilder::default()
                    .counterparty(recipient)
                    .index(i as u64)
                    .channel_epoch(epoch)
                    .win_prob(WinningProbability::ALWAYS)
                    .amount(100)
                    .challenge(Challenge::from_hint_and_share(
                        &hk1.to_challenge()?,
                        &hk2.to_challenge()?,
                    )?)
                    .build_signed(issuer, &Default::default())?
                    .into_acknowledged(Response::from_half_keys(&hk1, &hk2)?)
                    .into_redeemable(recipient, &Default::default())?;

                tickets.push(ticket);
            }
        }

        tickets.sort();
        Ok(tickets)
    }

    pub fn fill_queue<Q: TicketQueue, I: Iterator<Item = RedeemableTicket>>(
        queue: &mut Q,
        iter: I,
    ) -> anyhow::Result<()> {
        for ticket in iter {
            queue.push(ticket)?;
        }
        Ok(())
    }

    #[test]
    fn list_channels() -> anyhow::Result<()> {
        let dir = tempdir()?;
        let db_path = dir.path().join("test_list.db");
        let mut store = RedbStore::new(&db_path)?;

        let channel1 = ChannelId::from([1u8; 32]);
        let channel2 = ChannelId::from([2u8; 32]);

        store.open_or_create_queue(&channel1)?;
        store.open_or_create_queue(&channel2)?;

        let cli = Cli {
            db_file: db_path.clone(),
            format: OutputFormat::Plain,
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
    fn delete_queue() -> anyhow::Result<()> {
        let dir = tempdir()?;
        let db_path = dir.path().join("test_delete_queue.db");
        let mut store = RedbStore::new(&db_path)?;

        let channel = ChannelId::from([1u8; 32]);
        store.open_or_create_queue(&channel)?;

        assert_eq!(store.iter_queues()?.count(), 1);

        let cli = Cli {
            db_file: db_path.clone(),
            format: OutputFormat::Plain,
            command: Commands::DeleteQueue { channel_id: channel },
        };

        let result = run_command(cli, &mut store)?;
        match result {
            CommandResult::DeleteQueue(res) => {
                assert_eq!(res.channel_id, channel);
                assert_eq!(res.deleted_tickets_count, 0); // No tickets were in the queue
            }
            _ => panic!("Expected DeleteQueue result"),
        }

        assert_eq!(store.iter_queues()?.count(), 0);

        Ok(())
    }

    #[test]
    fn list_tickets() -> anyhow::Result<()> {
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
            db_file: db_path.clone(),
            format: OutputFormat::Plain,
            command: Commands::ListTickets { channel_id: channel },
        };

        let result = run_command(cli, &mut store)?;
        match result {
            CommandResult::ListTickets(res) => {
                assert_eq!(res.channel_id, channel);
                assert_eq!(res.tickets.len(), 3);
            }
            _ => panic!("Expected ListTickets result"),
        }

        Ok(())
    }

    #[test]
    fn delete_ticket() -> anyhow::Result<()> {
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
            db_file: db_path.clone(),
            format: OutputFormat::Plain,
            command: Commands::DeleteTicket {
                channel_id: channel,
                index: 2,
            },
        };

        let result = run_command(cli, &mut store)?;
        match result {
            CommandResult::DeleteTicket(res) => {
                assert_eq!(res.channel_id, channel);
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
    fn total_sum() -> anyhow::Result<()> {
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
            db_file: db_path.clone(),
            format: OutputFormat::Plain,
            command: Commands::TotalValue { channel_id: channel },
        };

        let result = run_command(cli, &mut store)?;
        match result {
            CommandResult::TotalValue(res) => {
                assert_eq!(res.channel_id, channel);
                assert_eq!(res.total_sum, expected_sum.to_string());
            }
            _ => panic!("Expected TotalSum result"),
        }

        Ok(())
    }

    #[test]
    fn open_or_create_queue_inspected() -> anyhow::Result<()> {
        let dir = tempdir()?;
        let db_path = dir.path().join("test_inspected.db");
        let mut store = RedbStore::new(&db_path)?;

        let channel = ChannelId::from([4u8; 32]);

        // Ensure no queues exist initially
        assert_eq!(store.iter_queues()?.count(), 0);

        // Test with ListTickets command on non-existent channel
        let cli = Cli {
            db_file: db_path.clone(),
            format: OutputFormat::Plain,
            command: Commands::ListTickets { channel_id: channel },
        };

        let result = run_command(cli, &mut store)?;
        match result {
            CommandResult::ListTickets(res) => {
                assert_eq!(res.channel_id, channel);
                assert!(res.tickets.is_empty());
            }
            _ => panic!("Expected ListTickets result"),
        }

        // The store should still have 0 queues because run_command should check before opening
        assert_eq!(store.iter_queues()?.count(), 0);

        Ok(())
    }
}
