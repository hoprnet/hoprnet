# Ticket Inspector

`ticket-inspector` is a CLI tool designed to inspect and manipulate the HOPR redeemable tickets database. It allows users to list channel IDs, display tickets in specific queues, delete queues or individual tickets, and calculate the total value of tickets for a given channel.

## Build and Installation

To build the `ticket-inspector` binary, use the following command from the project root:

```bash
cargo build -p hopr-ticket-manager --bin ticket-inspector --features redb,serde
```

Note: The `redb` feature is required for the tool to work with the default database implementation. The `serde` feature enables JSON output formatting.

## Usage

### Global Options

- `--db-file <FILE>` or `-d <FILE>`: (Required) Path to the database file.
- `--format <FORMAT>` or `-f <FORMAT>`: Output format. Options: `plain` (default), `json` (requires `serde` feature).

### Subcommands

#### `list-channels` (alias: `-c`)

List Channel IDs of all ticket queues currently in the database.

```bash
ticket-inspector --db-file /path/to/db list-channels
```

#### `list-tickets` (alias: `-l`)

Display all tickets in a particular queue in-order for a given Channel ID.

```bash
ticket-inspector --db-file /path/to/db list-tickets --channel-id <CHANNEL_ID>
```

#### `delete-queue` (alias: `dq`)

Delete all tickets associated with a specific Channel ID.

```bash
ticket-inspector --db-file /path/to/db delete-queue --channel-id <CHANNEL_ID>
```

#### `delete-ticket` (alias: `-e`)

Delete all tickets in a queue up to and including a specified ticket index for a given Channel ID.

```bash
ticket-inspector --db-file /path/to/db delete-ticket --channel-id <CHANNEL_ID> --index <INDEX>
```

#### `total-value` (alias: `-t`)

Calculate and print the total sum of all ticket amounts for a given Channel ID.

```bash
ticket-inspector --db-file /path/to/db total-value --channel-id <CHANNEL_ID>
```

## Output Formats

The tool supports two output formats:

1.  **Plain (default)**: Human-readable text format.
2.  **JSON**: Machine-readable JSON format. Requires the `serde` feature during compilation.

Example of JSON output:

```bash
ticket-inspector --db-file /path/to/db --format json list-channels
```
