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

To create some identities with password as env variable. Alternatively, a path to the password file can be provided with `--password-path`, e.g. `--password-path ./.pwd`

```
IDENTITY_PASSWORD=local \
    hopli identity \
    --action create \
    --directory "./test" \
    --name node_ \
    --number 3
```

Read ethereum addresses from identities

```
IDENTITY_PASSWORD=switzerland \
    hopli identity \
    --action read \
    --directory "./test" \
    --name node_

```

To fund nodes with password from env variable `IDENTITY_PASSWORD`. Alternatively, a path to the password file can be provided with `--password-path`, e.g. `--password-path ./.pwd`

```
IDENTITY_PASSWORD=local \
PRIVATE_KEY=<bank_private_key> \
hopli faucet \
    --environment-name anvil-localhost \
    --use-local-identities --identity-directory "/tmp" \
    --address 0x0aa7420c43b8c1a7b165d216948870c8ecfe1ee1 \
    --contracts-root "../ethereum/contracts" \
    --hopr-amount 10 --native-amount 1
```

Note that only identity files ending with `.id` are recognized by the CLI

To register nodes

```
export PRIVATE_KEY=<bank_private_key> \
hopli register-in-network-registry \
    --environment-name anvil-localhost \
    --peer-ids 16Uiu2HAmC9CRFeuF2cTf6955ECFmgDw6d27jLows7bftMqat5Woz,16Uiu2HAmUsJwbECMroQUC29LQZZWsYpYZx1oaM1H9DBoZHLkYn12 \
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
cargo run -- identity --action create --password-path ./.pwd --directory "./test" --name node_ --number 3
```

Create 2 identity files in `./test` folder where password is stored as an environment variable `IDENTITY_PASSWORD`

```
IDENTITY_PASSWORD=switzerland \
cargo run -- identity --action create --directory "./test" --name node_ --number 2
```

Read ethereum addresses from identities

```
IDENTITY_PASSWORD=switzerland \
cargo run -- identity --action read --directory "./test" --name node_

```

Fund nodes with password as env variable. Alternatively, a path to the password file can be provided with `--password-path`, e.g. `--password-path ./.pwd`

```
PRIVATE_KEY=<bank_private_key> \
IDENTITY_PASSWORD=local \
    cargo run -- faucet --environment-name anvil-localhost \
    --use-local-identities --identity-directory "/tmp" \
    --address 0x0aa7420c43b8c1a7b165d216948870c8ecfe1ee1 \
    --contracts-root "../ethereum/contracts"  \
    --hopr-amount 10 --native-amount 1
```

Register some peer ids in the network registry contract

```
PRIVATE_KEY=<bank_private_key> \
    cargo run -- register-in-network-registry --environment-name anvil-localhost \
    --peer-ids 16Uiu2HAmC9CRFeuF2cTf6955ECFmgDw6d27jLows7bftMqat5Woz,16Uiu2HAmUsJwbECMroQUC29LQZZWsYpYZx1oaM1H9DBoZHLkYn12 \
    --contracts-root "../ethereum/contracts"
```

### Test

```
cargo test -- --nocapture
```
