#!/usr/bin/env bash
#
# Verifies that an entity's build actually drops the code of the other entities.
#
# The fence is a set of `cfg`s in `hopr-lib` and `hopr-transport`, so a positive build proves only
# that a set compiles, not that it is smaller than the others. This script proves the negative half:
# for each entity it writes a one-line crate that names an item the entity must not have, and
# requires `cargo check` to reject it.
#
# Usage: scripts/check-entity-fencing.sh [entity ...]
#   Without arguments it checks every entity. Entities: edge, relay, exit, raw-exit — the basic
#   division is edge <-> relay <-> exit, and `raw-exit` is the same Session server without the
#   relay's ticket processing.
#
# The fixture crates live in a temporary directory and are their own workspaces, so they resolve
# `hopr-lib` through the same path dependency this repository uses. They share the workspace target
# directory, so the dependency artifacts built once are reused across fixtures.
set -euo pipefail

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
workdir=$(mktemp -d)
trap 'rm -rf "$workdir"' EXIT

export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-${repo_root}/target}"
export CARGO_TERM_COLOR=never

# `$1`: fixture name. `$2`: the entity feature set hopr-lib is built with.
# `$3`: the source of the fixture crate.
write_fixture() {
  local name=$1 features=$2 code=$3
  local dir="$workdir/$name"

  mkdir -p "$dir/src"
  cat >"$dir/Cargo.toml" <<EOF
[package]
name = "fence-$name"
version = "0.0.0"
edition = "2024"
publish = false

[workspace]

[dependencies]
hopr-lib = { path = "${repo_root}/hopr/hopr-lib", default-features = false, features = ["${features}"] }
EOF
  printf '%s\n' "$code" >"$dir/src/lib.rs"
  echo "$dir"
}

failures=0

# `$1`: entity set. `$2`: item name used in the failure message. `$3`: source that must not compile.
expect_absent() {
  local features=$1 item=$2 code=$3
  local dir
  dir=$(write_fixture "$(echo "$item" | tr -c 'a-zA-Z0-9\n' '-')" "$features" "$code")

  local stderr_file="$dir/stderr"
  if cargo check --quiet --manifest-path "$dir/Cargo.toml" 2>"$stderr_file"; then
    echo "FENCE BROKEN: \`$item\` compiles with features [$features]" >&2
    failures=$((failures + 1))
    return
  fi

  # A rejection for any other reason (a typo in the fixture, a missing dependency) would make
  # this check pass for the wrong reason, so require the item to be what the compiler complains
  # about.
  if ! grep -qE "(cannot find|not found|no method named|no function or associated item|no field named|unknown field)" "$stderr_file"; then
    echo "FENCE CHECK ERROR: fixture for \`$item\` failed for an unrelated reason:" >&2
    cat "$stderr_file" >&2
    failures=$((failures + 1))
    return
  fi
  echo "ok: \`$item\` is absent with features [$features]"
}

# `$1`: entity set. Must compile.
expect_present() {
  local features=$1
  if ! cargo check --quiet -p hopr-lib --no-default-features --features "$features" \
    --manifest-path "$repo_root/Cargo.toml"; then
    echo "FENCE BROKEN: hopr-lib does not build with features [$features]" >&2
    failures=$((failures + 1))
    return
  fi
  echo "ok: hopr-lib builds with features [$features]"
}

check_edge() {
  # An edge relays nothing and admits no Sessions.
  expect_present edge
  expect_absent edge "build_relay" \
    'pub fn f() { let _ = hopr_lib::builder::HoprBuilder::build_relay; }'
  expect_absent edge "HoprTransport::run_relay" \
    'pub fn f() { let _ = hopr_lib::exports::transport::HoprTransport::run_relay; }'
  expect_absent edge "HoprLibConfig::out_index_sync_period" \
    'pub fn f() -> hopr_lib::config::HoprLibConfig { hopr_lib::config::HoprLibConfig { out_index_sync_period: core::time::Duration::from_secs(1), ..Default::default() } }'
  expect_absent edge "with_session_server" \
    'pub fn f() { let _ = hopr_lib::builder::HoprBuilderConfigured::with_session_server; }'
  expect_absent edge "HoprBuilderWithSession" \
    'pub fn f() -> Option<hopr_lib::builder::HoprBuilderWithSession> { None }'
  expect_absent edge "HoprLibConfig::incoming_session_capacity" \
    'pub fn f() -> hopr_lib::config::HoprLibConfig { hopr_lib::config::HoprLibConfig { incoming_session_capacity: 1, ..Default::default() } }'
}

check_relay() {
  # A pure relay carries no Session stack at all.
  expect_present relay
  expect_absent relay "HoprSessionClientConfig" \
    'pub fn f() -> Option<hopr_lib::HoprSessionClientConfig> { None }'
  expect_absent relay "HoprTransport::new_session" \
    'pub fn f() { let _ = hopr_lib::exports::transport::HoprTransport::new_session; }'
  expect_absent relay "HoprLibConfig::incoming_session_capacity" \
    'pub fn f() -> hopr_lib::config::HoprLibConfig { hopr_lib::config::HoprLibConfig { incoming_session_capacity: 1, ..Default::default() } }'
  expect_absent relay "with_session_server" \
    'pub fn f() { let _ = hopr_lib::builder::HoprBuilderConfigured::with_session_server; }'
}

check_exit() {
  # The relaying exit keeps both the relay's ticket path and the Session server, but still has no
  # source (edge) role.
  expect_present exit
  expect_absent exit "build_edge" \
    'pub fn f() { let _ = hopr_lib::builder::HoprBuilderWithSession::build_edge; }'
}

check_raw_exit() {
  # The raw exit is a Session server without ticket processing.
  expect_present raw-exit
  expect_absent raw-exit "build_relay" \
    'pub fn f() { let _ = hopr_lib::builder::HoprBuilderWithSession::build_relay; }'
  expect_absent raw-exit "HoprLibConfig::out_index_sync_period" \
    'pub fn f() -> hopr_lib::config::HoprLibConfig { hopr_lib::config::HoprLibConfig { out_index_sync_period: core::time::Duration::from_secs(1), ..Default::default() } }'
  expect_absent raw-exit "HoprTransport::run_relay" \
    'pub fn f() { let _ = hopr_lib::exports::transport::HoprTransport::run_relay; }'
}

entities=("$@")
if [ ${#entities[@]} -eq 0 ]; then
  entities=(edge relay exit raw-exit)
fi

for entity in "${entities[@]}"; do
  case "$entity" in
  edge) check_edge ;;
  relay) check_relay ;;
  exit) check_exit ;;
  raw-exit) check_raw_exit ;;
  *)
    echo "unknown entity: $entity (expected one of edge, relay, exit, raw-exit)" >&2
    exit 2
    ;;
  esac
done

if [ "$failures" -ne 0 ]; then
  echo "$failures fence check(s) failed" >&2
  exit 1
fi
echo "all entity fence checks passed"
