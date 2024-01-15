#!/usr/bin/env bash

# prevent sourcing of this script, only allow execution
$(return >/dev/null 2>&1)
test "$?" -eq "0" && { echo "This script should only be executed." >&2; exit 1; }

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -Eeuo pipefail

# set log id and use shared log function for readable logs
declare mydir
mydir=$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd -P)

# Installs all toolchain utilities that are required to build hoprnet monorepo, including
# - protoc -> /usr/local/bin
# - Rust (rustc, cargo) -> ./../../.cargo/bin
#
# Supposed to work for
#   x86_64: Docker + Alpine
#   x86_64: Debian-based
#   x86_64: macOS
#   x86_64: GitHub Actions images: macos-latest + ubuntu-latest

# Set PATH such that `cargo` is available within this script
export CARGO_BIN_DIR="${mydir}/../../.cargo/bin"
export PATH=${PATH}:${CARGO_BIN_DIR}

declare usr_local="/usr/local"
declare usr_local_bin="${usr_local}/bin"

while (( "$#" )); do
  case "$1" in
    -h|--help)
      # return early with help info when requested
      usage
      exit 0
      ;;
    -*|--*=)
      usage
      exit 1
      ;;
    *)
      shift
      ;;
  esac
done

# move /usr/local/bin to beginning of PATH
export PATH="${usr_local_bin}:${PATH//:\/usr\/local\/bin/}"

if ! [ -w "${usr_local_bin}" ]; then
    echo "Cannot install utilities. \"${usr_local_bin}\" is not writable."
    exit 1
fi

if ! [ -d "/opt" ] && ! [ -w "/opt" ] || ! [ -w "/" ]; then
    echo "Cannot install utilities. \"/opt\" does not exist or is not writable."
    exit 1
fi

declare download_dir="/tmp/hopr-toolchain/download"
mkdir -p ${download_dir}

function is_alpine() {
    local os_id=$(grep -E '^ID=' /etc/os-release | tr -d 'ID=')
    [ "${os_id}" = "alpine" ]
}

function install_rustup() {
    if ! command -v rustup; then
        echo "Installing Rustup"
        # Get rustup but install toolchain later
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --no-modify-path --default-toolchain none -y
    fi
}

function install_cargo() {
    if ! command -v cargo; then
        local rust_toolchain_toml_path="${mydir}/../../rust-toolchain.toml"
        local rust_version=$(sed -En 's/^channel = "([0-9a-z.]*)"$/\1/p' ${rust_toolchain_toml_path})
        echo "Installing Cargo, Rust compiler v${rust_version}"
        local target=$(sed -En 's/^targets = \[ "([a-z0-9_-]*)" \]$/\1/p' ${rust_toolchain_toml_path})
        local profile=$(sed -En 's/^profile = "([a-z]*)"$/\1/p' ${rust_toolchain_toml_path})

        # Always install the version specified in `rust-toolchain.toml`
        $HOME/.cargo/bin/rustup toolchain install ${rust_version} --profile ${profile} --target ${target}
        source "$HOME/.cargo/env"
    fi
}

# used by rust libp2p to compile objects
function install_protobuf() {
  local protobuf_version=21.12
  echo "Installing protobuf version 3.${protobuf_version}"
  local ostype="$(uname -s)"
  local cputype="$(uname -m)"
  case "${ostype}" in
      Linux | linux)
          ostype="linux"
          ;;
      Darwin)
          ostype="macos"
          ;;
      *)
          echo "no precompiled binaries available for OS: ${ostype}"
      ;;
  esac
  PB_REL="https://github.com/protocolbuffers/protobuf/releases"
  curl -o /tmp/protoc-${protobuf_version}.zip -fsSLO "${PB_REL}/download/v${protobuf_version}/protoc-${protobuf_version}-${ostype}-${cputype}.zip"
  mkdir -p /opt/protoc-${protobuf_version}-${ostype}-${cputype}
  unzip /tmp/protoc-${protobuf_version}.zip -d /opt/protoc-${protobuf_version}-${ostype}-${cputype}
  ln -sf /opt/protoc-${protobuf_version}-${ostype}-${cputype}/bin/protoc /usr/local/bin/protoc
}


install_rustup
install_cargo

# launch rustc once so it installs updated components
rustc --version

install_protobuf

# Show some debug output
echo ""
echo "Checking installed tool versions"
echo "================================"
echo ""
  command -v rustc >/dev/null && rustc --version
  command -v cargo >/dev/null && cargo --version
  command -v protoc >/dev/null && protoc --version
echo ""

rm -R ${download_dir}
