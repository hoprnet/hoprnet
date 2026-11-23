#!/usr/bin/env bash
#
# Verifies that an entity's build actually drops the code of the other entities.
#
# The fence is a set of `cfg`s in `hopr-lib` and `hopr-transport`, so a positive build proves only
# that a set compiles, not that it is smaller than the others. This script proves the negative half:
# for each entity it writes a one-line crate that names an item the entity must not have, and
# requires `cargo check` to reject it because that very item is missing.
#
# Every absence check has a control: the same source, built with an entity that does have the item,
# must NOT be rejected for that item. Without the control a typo in the fixture would pass as a
# fence (an unresolvable path is "absent" everywhere).
#
# Usage: scripts/check-entity-fencing.sh [entity ...]
#   Without arguments it checks every entity. Entities: entry, relay, exit — the basic division is
#   entry <-> relay <-> exit.
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

failures=0

# `$1`: fixture name. `$2`: the entity feature set hopr-lib is built with.
# `$3`: the source of the fixture crate. Prints the compiler's stderr and returns its status.
build_fixture() {
  local name=$1 features=$2 code=$3
  local dir="$workdir/$name-$features"

  mkdir -p "$dir/src"
  cat >"$dir/Cargo.toml" <<EOF
[package]
name = "fence-$name-$features"
version = "0.0.0"
edition = "2024"
publish = false

[workspace]

[dependencies]
hopr-lib = { path = "${repo_root}/hopr/hopr-lib", default-features = false, features = ["${features}"] }
EOF
  # The fixture is its own workspace, so it takes the repository's lockfile and `[patch]` section to
  # resolve the same dependency versions as the real build.
  cp "$repo_root/Cargo.lock" "$dir/Cargo.lock"
  sed -n '/^\[patch\./,$p' "$repo_root/Cargo.toml" >>"$dir/Cargo.toml"
  printf '%s\n' "$code" >"$dir/src/lib.rs"
  cargo check --quiet --manifest-path "$dir/Cargo.toml" 2>&1
}

# The compiler error that reports `$1` as missing: an error header that names the item, such as
# "no associated function or constant named `build_relay` found", "cannot find type `X`" or
# "struct `Y` has no field named `z`". Anything else (a type annotation the fixture left out, a trait
# bound) is NOT a report that the item is missing.
reports_missing() {
  grep -qE "^error(\[E[0-9]+\])?: .*(cannot find|no .*named|has no field|unresolved|not found).*\`$1\`" <<<"$2"
}

# True when `$1` (compiler output) holds no error other than "type annotations needed". The fixtures
# name items without their generic parameters, so an item that exists still fails inference; any other
# error means the build broke for an unrelated reason and the item's presence was not shown.
only_inference_errors() {
  local errors
  errors=$(grep -E "^error" <<<"$1" | grep -vE "^error\[E028[234]\]|^error: could not compile" || true)
  [ -z "$errors" ]
}

# `$1`: entity set that must lack the item. `$2`: an entity set that has it (the control).
# `$3`: the item's name, as the compiler reports it. `$4`: source that names the item.
expect_absent() {
  local features=$1 control=$2 item=$3 code=$4
  local name
  name=$(echo "$item" | tr -c 'a-zA-Z0-9\n' '-')

  local with_item
  with_item=$(build_fixture "$name" "$control" "$code" || true)
  if reports_missing "$item" "$with_item" || ! only_inference_errors "$with_item"; then
    echo "FENCE CHECK ERROR: \`$item\` is not shown present with [$control], so the fixture proves nothing:" >&2
    echo "$with_item" >&2
    failures=$((failures + 1))
    return
  fi

  local without_item
  if without_item=$(build_fixture "$name" "$features" "$code"); then
    echo "FENCE BROKEN: \`$item\` compiles with features [$features]" >&2
    failures=$((failures + 1))
    return
  fi
  if ! reports_missing "$item" "$without_item"; then
    echo "FENCE CHECK ERROR: fixture for \`$item\` failed with [$features] for an unrelated reason:" >&2
    echo "$without_item" >&2
    failures=$((failures + 1))
    return
  fi
  echo "ok: \`$item\` is absent with features [$features] and present with [$control]"
}

# `$1`: entity set that must have the item. `$2`: the item's name. `$3`: source that names it.
expect_has() {
  local features=$1 item=$2 code=$3
  local out
  out=$(build_fixture "has-$item" "$features" "$code" || true)
  if reports_missing "$item" "$out" || ! only_inference_errors "$out"; then
    echo "FENCE BROKEN: \`$item\` is not shown present with features [$features]" >&2
    failures=$((failures + 1))
    return
  fi
  echo "ok: \`$item\` is present with features [$features]"
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

# `$1`: entity set. `$2`: crate that must (`linked`) or must not (`absent`) be in its dependency tree.
# Item checks prove a cfg fence; this proves the crate itself is not built, which is what keeps the
# code out of the binary.
expect_crate() {
  local features=$1 mode=$2 crate=$3
  local tree
  tree=$(cargo tree --quiet --manifest-path "$repo_root/Cargo.toml" -p hopr-lib --no-default-features \
    --features "$features,pix-bjj,transport-announce-quic" -e normal --prefix none 2>/dev/null | sed 's/ (.*//' | awk '{print $1}')
  if grep -qx "$crate" <<<"$tree"; then
    if [ "$mode" = absent ]; then
      echo "FENCE BROKEN: \`$crate\` is linked with features [$features]" >&2
      failures=$((failures + 1))
      return
    fi
  elif [ "$mode" = linked ]; then
    echo "FENCE BROKEN: \`$crate\` is not linked with features [$features]" >&2
    failures=$((failures + 1))
    return
  fi
  echo "ok: \`$crate\` is $mode with features [$features]"
}

# `$1`: entity set. `$2`: `on` or `off`. `$3`: crate. `$4`: its feature, which must be so for the set.
expect_feature() {
  local features=$1 state=$2 crate=$3 feature=$4
  local enabled=no
  if cargo tree --quiet --manifest-path "$repo_root/Cargo.toml" -p hopr-lib --no-default-features \
    --features "$features,pix-bjj,transport-announce-quic" -e features -i "$crate" 2>/dev/null |
    grep -q "$crate feature \"$feature\""; then
    enabled=yes
  fi
  if { [ "$state" = on ] && [ "$enabled" = no ]; } || { [ "$state" = off ] && [ "$enabled" = yes ]; }; then
    echo "FENCE BROKEN: $crate feature \`$feature\` is not $state with [$features]" >&2
    failures=$((failures + 1))
    return
  fi
  echo "ok: $crate feature \`$feature\` is $state with features [$features]"
}

# Source snippets shared between the entities.
BUILD_RELAY='pub fn f() { let _ = hopr_lib::builder::HoprBuilderConfigured::build_relay; }'
BUILD_EXIT='pub fn f() { let _ = hopr_lib::builder::HoprBuilderWithSession::build_exit; }'
SESSION_BUILDER='pub fn f() -> Option<hopr_lib::builder::HoprBuilderWithSession> { None }'
BUILD_ENTRY='pub fn f() { let _ = hopr_lib::builder::HoprBuilderConfigured::build_entry; }'
RUN_RELAY='pub fn f() { let _ = hopr_lib::exports::transport::HoprTransport::run_relay; }'
RELAY_CFG='pub fn f() -> hopr_lib::config::HoprLibConfig { hopr_lib::config::HoprLibConfig { out_index_sync_period: core::time::Duration::from_secs(1), ..Default::default() } }'
SESSION_CAP_CFG='pub fn f() -> hopr_lib::config::HoprLibConfig { hopr_lib::config::HoprLibConfig { incoming_session_capacity: 1, ..Default::default() } }'
WITH_SESSION_SERVER='pub fn f() { let _ = hopr_lib::builder::HoprBuilderConfigured::with_session_exit; }'

check_entry() {
  # An entry relays nothing, runs no PIX exit, and admits no Sessions.
  expect_present entry
  expect_feature entry off hopr-transport raw-exit
  expect_feature entry off hopr-protocol-hopr relaying
  expect_absent entry relay build_relay "$BUILD_RELAY"
  expect_absent entry relay run_relay "$RUN_RELAY"
  expect_absent entry relay out_index_sync_period "$RELAY_CFG"
  expect_absent entry exit with_session_exit "$WITH_SESSION_SERVER"
  expect_absent entry exit HoprBuilderWithSession "$SESSION_BUILDER"
  expect_absent entry exit incoming_session_capacity "$SESSION_CAP_CFG"
}

check_relay() {
  # A pure relay carries no Session stack, no PIX exit and no source role.
  expect_present relay
  expect_feature relay off hopr-transport raw-exit
  expect_feature relay on hopr-protocol-hopr relaying
  for crate in hopr-transport-session hopr-protocol-session hopr-protocol-start; do
    expect_crate relay absent "$crate"
    expect_crate entry linked "$crate"
    expect_crate exit linked "$crate"
  done
  expect_absent relay entry build_entry "$BUILD_ENTRY"
  expect_absent relay exit HoprBuilderWithSession "$SESSION_BUILDER"
  expect_absent relay entry HoprSessionEntryConfig \
    'pub fn f() -> Option<hopr_lib::HoprSessionEntryConfig> { None }'
  expect_absent relay entry new_session \
    'pub fn f() { let _ = hopr_lib::exports::transport::HoprTransport::new_session; }'
  expect_absent relay exit incoming_session_capacity "$SESSION_CAP_CFG"
  expect_absent relay exit with_session_exit "$WITH_SESSION_SERVER"
}

check_exit() {
  # The exit keeps the relay's ticket path and adds the Session server, but has no source role.
  expect_present exit
  expect_feature exit off hopr-transport raw-exit
  expect_feature exit on hopr-protocol-hopr relaying
  expect_has exit build_exit "$BUILD_EXIT"
  expect_absent exit entry build_entry "$BUILD_ENTRY"
}

entities=("$@")
if [ ${#entities[@]} -eq 0 ]; then
  entities=(entry relay exit)
fi

for entity in "${entities[@]}"; do
  case "$entity" in
  entry) check_entry ;;
  relay) check_relay ;;
  exit) check_exit ;;
  *)
    echo "unknown entity: $entity (expected one of entry, relay, exit)" >&2
    exit 2
    ;;
  esac
done

if [ "$failures" -ne 0 ]; then
  echo "$failures fence check(s) failed" >&2
  exit 1
fi
echo "all entity fence checks passed"
