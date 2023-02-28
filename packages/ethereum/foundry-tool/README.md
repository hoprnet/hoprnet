# Tool

Complete some missing funcitonalities of the foundry-centered smart contract toolchain. Notably storage of deployment files per environment.

### Run local development

```
cargo run -- -h
cargo run -- --environment-name localhost --environment-type development files --list
```

```
export PRIVATE_KEY=<bank_private_key>
cargo run -- faucet --environment-name anvil-localhost \
    --environment-type development \
    --password local --use-local-identities --identity-directory "/tmp" \
    --address 0x0aa7420c43b8c1a7b165d216948870c8ecfe1ee1 \
    --private-key $PRIVATE_KEY \
    --make-root "../contracts"  \
    --hopr-amount 10 --native-amount 1
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

```
foundry-tool faucet \
    --environment-name anvil-localhost --environment-type development \
    --password local --use-local-identities --identity-directory "/tmp" \
    --address 0x0aa7420c43b8c1a7b165d216948870c8ecfe1ee1 --private-key <bank_private_key> \
    --make-root "../contracts" \
    --hopr-amount 10 --native-amount 1
```

Note that only identity files ending with `.id` are recognized by the CLI
