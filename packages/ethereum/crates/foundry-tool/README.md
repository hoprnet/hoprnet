# Tool

Complete some missing funcitonalities of the foundry-centered smart contract toolchain. Notably storage of deployment files per environment.


 9650  cargo build --release
 9652  cargo install --path .

### Run local development
```
cargo run -- -h
cargo run -- --environment-name localhost --environment-type 0 files 
cargo run -- --environment-name localhost --environment-type 0 faucet --password local --use-local-identities --amount 300 --token-type native --identity-directory "/tmp"
```

### Test
```
cargo test -- --nocapture
```