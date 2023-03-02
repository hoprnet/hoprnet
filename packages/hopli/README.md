# Hopli

CLI tool to manage HOPR identity generation, decryption, funding and registering to network registry.

### Run local development

```
cargo run -- -h
cargo run -- identity --action create --password switzerland --directory "./test" --name node_ --number 3
```

```
export PRIVATE_KEY=<bank_private_key>
cargo run -- faucet --environment-name anvil-localhost \
    --environment-type development \
    --password local --use-local-identities --identity-directory "/tmp" \
    --address 0x0aa7420c43b8c1a7b165d216948870c8ecfe1ee1 \
    --private-key $PRIVATE_KEY \
    --contracts-root "../ethereum/contracts"  \
    --hopr-amount 10 --native-amount 1
```

```
export PRIVATE_KEY=<bank_private_key>
cargo run -- network-registry --environment-name anvil-localhost \
    --environment-type development \
    --peer-ids 16Uiu2HAmC9CRFeuF2cTf6955ECFmgDw6d27jLows7bftMqat5Woz,16Uiu2HAmUsJwbECMroQUC29LQZZWsYpYZx1oaM1H9DBoZHLkYn12 \
    --private-key $PRIVATE_KEY \
    --contracts-root "../ethereum/contracts"
```

### Test

```
cargo test -- --nocapture
```

### Installation

```
cargo build --release
cargo install --path .
```

To create some identities

```
hopli identity \
    --action create \
    --password switzerland \
    --directory "./test" \
    --name node_ \
    --number 3
```

To fund nodes

```
hopli faucet \
    --environment-name anvil-localhost --environment-type development \
    --password local --use-local-identities --identity-directory "/tmp" \
    --address 0x0aa7420c43b8c1a7b165d216948870c8ecfe1ee1 \
    --private-key <bank_private_key> \
    --contracts-root "../ethereum/contracts" \
    --hopr-amount 10 --native-amount 1
```

Note that only identity files ending with `.id` are recognized by the CLI

To register nodes

```
hopli network-registry \
    --environment-name anvil-localhost \
    --environment-type development \
    --peer-ids 16Uiu2HAmC9CRFeuF2cTf6955ECFmgDw6d27jLows7bftMqat5Woz,16Uiu2HAmUsJwbECMroQUC29LQZZWsYpYZx1oaM1H9DBoZHLkYn12 \
    --private-key <bank_private_key> \
    --contracts-root "../ethereum/contracts"
```
