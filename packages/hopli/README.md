# Hopli

CLI tool to manage HOPR identity generation, decryption, funding and registering to network registry.

## Installation

It uses Foundry's `forge` component to prepare, sign and broadcast transaction.
Please ensure that `forge` binary exists in your PATH.
It should return the version when running

```
forge --version
```

If an error returns, please follow the [Foundry installation guide](https://book.getfoundry.sh/getting-started/installation) to install it on your machine.

```
cargo build --release
cargo install --path .
```

## Commands

### General arguments

#### Identity directory or path

Identities can be read from a directory or directly from a path.

When reading from a directory, identity files MUST contain "id" in their file name to be considered as an identity file.
An additional parameter which specifies its prefix can also be passed.

```
    --identity-directory "./test" \
    --identity-prefix node_ \
```

When reading from a path, use `--identity-from-path "./test/hopr.id"`

Path and directory can be passed at the same time.

Note: when CREATing identities, you must pass `--identity-directory`. `--identity-from-path` is not accepted

#### Password

Password can be passed either as an env variable `IDENTITY_PASSWORD`, or via a path to the password file `--password-path`, e.g. `--password-path ./.pwd`

### Create/Read identities

To create some identities with password as env variable. Alternatively, a path to the password file can be provided with `--password-path`, e.g. `--password-path ./.pwd`

```
IDENTITY_PASSWORD=local \
    hopli identity \
    --action create \
    --identity-directory "./test" \
    --identity-prefix node_ \
    --number 3
```

Read ethereum addresses from identities

```
IDENTITY_PASSWORD=switzerland \
    hopli identity \
    --action read \
    --identity-directory "./test" \
    --identity-prefix node_

```

To fund nodes with password from env variable `IDENTITY_PASSWORD`. Alternatively, a path to the password file can be provided with `--password-path`, e.g. `--password-path ./.pwd`

`--hopr-amount` and `native-amount` can be floating number

```
IDENTITY_PASSWORD=local \
PRIVATE_KEY=<bank_private_key> \
hopli faucet \
    --network anvil-localhost \
    --use-local-identities --identity-directory "/app/.hoprd-db" \
    --address 0x0aa7420c43b8c1a7b165d216948870c8ecfe1ee1 \
    --contracts-root "../ethereum/contracts" \
    --hopr-amount 10 --native-amount 0.1
```

Note that only identity files ending with `.id` are recognized by the CLI

To register nodes

```
export PRIVATE_KEY=<bank_private_key> \
hopli register-in-network-registry \
    --network anvil-localhost \
    --peer-ids 16Uiu2HAmC9CRFeuF2cTf6955ECFmgDw6d27jLows7bftMqat5Woz,16Uiu2HAmUsJwbECMroQUC29LQZZWsYpYZx1oaM1H9DBoZHLkYn12 \
    --contracts-root "../ethereum/contracts"
```

with node identities in the network registry contract

```
PRIVATE_KEY=<bank_private_key> \
IDENTITY_PASSWORD=switzerland \
hopli register-in-network-registry \
    --network anvil-localhost \
    --use-local-identities --identity-directory "/tmp" \
    --peer-ids 16Uiu2HAmC9CRFeuF2cTf6955ECFmgDw6d27jLows7bftMqat5Woz,16Uiu2HAmUsJwbECMroQUC29LQZZWsYpYZx1oaM1H9DBoZHLkYn12 \
    --contracts-root "../ethereum/contracts"
```

Express stake + register + fund

```
PRIVATE_KEY=<bank_private_key> \
hopli initialize-node --network anvil-localhost \
    --identity-directory "/tmp" \
    --password-path "/tmp/.pwd" \
    --hopr-amount 10 --native-amount 0.1 \
    --contracts-root "../ethereum/contracts"
```

Express create a safe and a module instances, then set default permissions

```
PRIVATE_KEY=<bank_private_key> \
hopli create-safe-module --network anvil-localhost \
    --identity-directory "./test" \
    --password-path "/test/.pwd" \
    --hopr-amount 10 --native-amount 0.1 \
    --contracts-root "../ethereum/contracts"
```

Migrate an exising set of node(d) with safe and module to a new network

```
PRIVATE_KEY=<safe_owner_private_key> DEPLOYER_PRIVATE_KEY=<network_registry_manager_key> \
hopli migrate-safe-module --network anvil-localhost \
    --identity-directory "./test" \
    --password-path "./test/.pwd" \
    --safe-address <safe_address> \
    --module-address <module_address> \
    --contracts-root "../ethereum/contracts"
```

## Development

### Run local development

```
cargo run -- -h
```

### Commands

Create 3 identity files in `./test` folder where password is saved in `.pwd` file

```
cargo run -- identity \
    --action create \
    --password-path ./.pwd \
    --identity-directory "./test" \
    --identity-prefix node_ \
    --number 3
```

Create 2 identity files in `./test` folder where password is stored as an environment variable `IDENTITY_PASSWORD`

```
IDENTITY_PASSWORD=switzerland \
cargo run -- identity \
    --action create \
    --identity-directory "./test" \
    --identity-prefix node_ \
    --number 2
```

Read ethereum addresses from identities

```
IDENTITY_PASSWORD=switzerland \
cargo run -- identity \
    --action read \
    --identity-directory "./test" \
    --identity-prefix node_

```

Fund nodes with password as env variable. Alternatively, a path to the password file can be provided with `--password-path`, e.g. `--password-path ./.pwd`

```
PRIVATE_KEY=<bank_private_key> \
IDENTITY_PASSWORD=local \
    cargo run -- faucet --network anvil-localhost \
    --use-local-identities --identity-directory "/tmp" \
    --address 0x0aa7420c43b8c1a7b165d216948870c8ecfe1ee1 \
    --contracts-root "../ethereum/contracts"  \
    --hopr-amount 10 --native-amount 0.1
```

Register some peer ids in the network registry contract

```
PRIVATE_KEY=<bank_private_key> \
    cargo run -- register-in-network-registry --network anvil-localhost \
    --peer-ids 16Uiu2HAmC9CRFeuF2cTf6955ECFmgDw6d27jLows7bftMqat5Woz,16Uiu2HAmUsJwbECMroQUC29LQZZWsYpYZx1oaM1H9DBoZHLkYn12 \
    --contracts-root "../ethereum/contracts"
```

Register some peer ids as well as some node identities in the network registry contract

```
PRIVATE_KEY=<bank_private_key> \
IDENTITY_PASSWORD=local \
    cargo run -- register-in-network-registry --network anvil-localhost \
    --use-local-identities --identity-directory "/tmp" \
    --peer-ids 16Uiu2HAmC9CRFeuF2cTf6955ECFmgDw6d27jLows7bftMqat5Woz,16Uiu2HAmUsJwbECMroQUC29LQZZWsYpYZx1oaM1H9DBoZHLkYn12 \
    --contracts-root "../ethereum/contracts"
```

> If foundry returns error that contains "HoprNetworkRegistry: Registry is disabled", run `cast send $(jq '.networks."anvil-localhost".network_registry_contract_address' ../ethereum/contracts/contracts-addresses.json) 'enableRegistry()' --rpc-url localhost:8545 --private-key $PRIVATE_KEY`

Express stake + registry + fund for node identity

```
PRIVATE_KEY=<bank_private_key> \
    cargo run -- initialize-node --network anvil-localhost \
    --identity-directory "./test" \
    --password-path "/test/.pwd" \
    --hopr-amount 10 --native-amount 0.1 \
    --contracts-root "../ethereum/contracts"
```

Express create a safe and a module instances, then set default permissions

```
PRIVATE_KEY=<bank_private_key> \
    cargo run -- create-safe-module --network anvil-localhost \
    --identity-directory "./test" \
    --password-path "./test/.pwd" \
    --hopr-amount 10 --native-amount 0.1 \
    --contracts-root "../ethereum/contracts"
```

Migrate an exising set of node(d) with safe and module to a new network

```
PRIVATE_KEY=<safe_owner_private_key> DEPLOYER_PRIVATE_KEY=<network_registry_manager_key> \
    cargo run -- migrate-safe-module --network anvil-localhost \
    --identity-directory "./test" \
    --password-path "./test/.pwd" \
    --safe-address <safe_address> \
    --module-address <module_address> \
    --contracts-root "../ethereum/contracts"
```

### Test

```
cargo test -- --nocapture
```

## Note:

1. When ` --use-local-identities`, the identity file should contain "id" in its name, either as part of the extention, or in the file stem.
