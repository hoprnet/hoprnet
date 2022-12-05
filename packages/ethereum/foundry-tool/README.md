# Tool

Complete some missing funcitonalities of the foundry-centered smart contract toolchain. Notably storage of deployment files per environment.

### Run local development

```
cargo run -- -h
cargo run -- --environment-name localhost --environment-type development files --list

cargo run -- --environment-name localhost --environment-type development faucet --password local --use-local-identities --token-type native --identity-directory "/tmp" --address 0x0aa7420c43b8c1a7b165d216948870c8ecfe1ee1 --private-key <bank_private_key> --make-root "../contracts"
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
foundry-tool --environment-name anvil-localhost --environment-type development faucet --password local --use-local-identities --token-type native --identity-directory "/tmp" --address 0x0aa7420c43b8c1a7b165d216948870c8ecfe1ee1 --private-key <bank_private_key> --make-root "../contracts"
```

Note that only identity files ending with `.id` are recognized by the CLI
