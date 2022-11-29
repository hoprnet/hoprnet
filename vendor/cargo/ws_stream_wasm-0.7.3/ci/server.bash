#!/usr/bin/bash

# fail fast
#
set -e

# print each command before it's executed
#
set -x

export RUSTFLAGS="-D warnings"

git clone --depth 1 https://github.com/najamelan/ws_stream_tungstenite
cd ws_stream_tungstenite
cargo build --example echo --release
cargo build --example echo_tt --release
cargo run --example echo --release &
cargo run --example echo_tt --release -- "127.0.0.1:3312"  &
