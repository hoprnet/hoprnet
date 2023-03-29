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
# - Node.js -> /usr/local/bin
# - Yarn -> /usr/local/bin + /opt/yarn-v${version}
# - protoc -> /usr/local/bin
# - Typescript + related utilities, such as ts-node -> ${mydir}/node_modules
# - Rust (rustc, cargo) -> ./../../.cargo/bin
# - wasm-pack + wasm-opt, necessary to build WebAssembly modules -> ./../../.cargo/bin
#
# Supposed to work for
#   x86_64: Docker + Alpine
#   x86_64: Debian-based
#   x86_64: macOS
#   x86_64: GitHub Actions images: macos-latest + ubuntu-latest
# @TODO adapt this script for macOS arm64 machines

function usage() {
  msg
  msg "Usage: $0 [-h|--help] [--runtime-only] [--with-yarn]"
  msg
  msg "This script installs all required toolchain utilities to build hoprnet monorepo"
  msg
  msg "Use --runtime-only to install only those utilities that are necessary at runtime"
  msg "Use --with-yarn to install yarn"
  msg
}

# Set PATH such that `cargo` is available within this script
export CARGO_BIN_DIR="${mydir}/../../.cargo/bin"
export PATH=${PATH}:${CARGO_BIN_DIR}

declare usr_local="/usr/local"
declare usr_local_bin="${usr_local}/bin"

declare install_all with_yarn
install_all="true"
with_yarn="false"

while (( "$#" )); do
  case "$1" in
    -h|--help)
      # return early with help info when requested
      usage
      exit 0
      ;;
    --runtime-only)
      install_all="false"
      shift
      ;;
    --with-yarn)
      with_yarn="true"
      shift
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

if [[ "${PATH}" =~ "${usr_local_bin}" ]]; then
    echo "Cannot install utilities. \"${usr_local_bin}\" is not part of home-path. ${PATH}"
    exit 1
fi

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
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- ----no-modify-path --default-toolchain none -y
    fi
}

function install_cargo() {
    if ! command -v cargo; then
        local rust_toolchain_toml_path="${mydir}/../../rust-toolchain.toml"
        local rust_version=$(sed -En 's/^channel = "([0-9a-z.]*)"$/\1/p' ${rust_toolchain_toml_path})
        echo "Installing Cargo, Rust compiler v${rust_version}"
        local target=$(sed -En 's/^targets = \[ "([a-z0-9-]*)" \]$/\1/p' ${rust_toolchain_toml_path})
        local profile=$(sed -En 's/^profile = "([a-z]*)"$/\1/p' ${rust_toolchain_toml_path})

        # Always install the version specified in `rust-toolchain.toml`
        $HOME/.cargo/bin/rustup toolchain install ${rust_version} --profile ${profile} --target ${target}
        source "$HOME/.cargo/env"
    fi
}

function install_wasm_pack() {
    if ! command -v wasm-pack; then
        cd ${download_dir}
        local rust_cargo_toml_path="${mydir}/../../Cargo.toml"
        local wasm_pack_release=$(sed -En 's/^wasm-pack = \"([0-9.]*)\"$/v\1/p' ${rust_cargo_toml_path})
        echo "Installing wasm-pack ${wasm_pack_release}"
        local ostype="$(uname -s)"
        local cputype="$(uname -m)"
        case "${ostype}" in
            Linux | linux)
                ostype="unknown-linux-musl"
                ;;
            Darwin)
                ostype="apple-darwin"
                ;;
            *)
                echo "no precompiled binaries available for OS: ${ostype}"
            ;;
        esac
        curl -fsSLO --compressed "https://github.com/rustwasm/wasm-pack/releases/download/${wasm_pack_release}/wasm-pack-${wasm_pack_release}-${cputype}-${ostype}.tar.gz"
        tar -xzf "wasm-pack-${wasm_pack_release}-${cputype}-${ostype}.tar.gz"
        local install_dir="${CARGO_BIN_DIR}"
        mkdir -p "${install_dir}"
        cp "wasm-pack-${wasm_pack_release}-${cputype}-${ostype}/wasm-pack" "${install_dir}"
        cd ${mydir}
    fi
}

# Used by wasm-pack to optimize WebAssembly binaries
function install_wasm_opt() {
    if ! command -v wasm-opt; then
        cd ${download_dir}
        local rust_cargo_toml_path="${mydir}/../../Cargo.toml"
        local wasm_opt_release=$(sed -En 's/^wasm-opt = \"([0-9.]*)\"$/v\1/p' ${rust_cargo_toml_path})
        echo "Installing wasm-opt ${wasm_opt_release}"
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
        local binaryen_release="version_$(echo "${wasm_opt_release}" | awk -F. '{ print $2; }')"
        curl -fsSLO --compressed "https://github.com/WebAssembly/binaryen/releases/download/${binaryen_release}/binaryen-${binaryen_release}-${cputype}-${ostype}.tar.gz"
        tar -xzf "binaryen-${binaryen_release}-${cputype}-${ostype}.tar.gz"
        local install_dir="${CARGO_BIN_DIR}"
        mkdir -p "${install_dir}"
        cp "binaryen-${binaryen_release}/bin/wasm-opt" "${install_dir}"
        cp -R "binaryen-${binaryen_release}/lib" "${install_dir}"
        cd ${mydir}
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


function install_node_js() {
    local nvmrc_path="${mydir}/../../.nvmrc"
    local node_js_version=$(sed -En 's/^v*([0-9.])/\1/p' ${nvmrc_path})
    if ! command -v node || [[ ! "$(node -v)" =~ ^v${node_js_version}\..+$ ]]; then
        cd ${download_dir}
        # Downloads Node.js version specified in `.nvmrc` file
        echo "Installing Node.js v${node_js_version}"
        local node_release=$(curl 'https://unofficial-builds.nodejs.org/download/release/index.json' | jq -r "[.[] | .version | select(. | startswith(\"v${node_js_version}\"))][0]")
        # using linux x64 builds by default
        local node_download_url="https://nodejs.org/download/release/${node_release}/node-${node_release}-linux-x64.tar.xz"
        if is_alpine; then
            # Using musl builds for alpine
            node_download_url="https://unofficial-builds.nodejs.org/download/release/${node_release}/node-${node_release}-linux-x64-musl.tar.xz"
        fi
        curl -fsSL --compressed "${node_download_url}" > node.tar.xz
        # ensure older installations are uninstalled first
        rm -rf \
          /usr/local/bin/node \
          /usr/local/bin/npx \
          /usr/local/bin/npm \
          /usr/local/bin/corepack \
          /usr/local/share/systemtap/tapset/node.stp \
          /usr/local/share/doc/node \
          /usr/local/share/man/man1/node.1 \
          /usr/local/include/node \
          /usr/local/lib/node_modules
        tar -xJf node.tar.xz -C ${usr_local} \
          --strip-components=1 --no-same-owner \
          --exclude README.md \
          --exclude LICENSE \
          --exclude CHANGELOG.md
        cd ${mydir}
    fi
}

# Install legacy version of yarn that supports handover to yarn 2+
function install_yarn() {
    local yarn_release="1.22.19"
    if ! command -v yarn; then
        cd ${download_dir}
        curl -fsSLO --compressed "https://yarnpkg.com/downloads/${yarn_release}/yarn-v${yarn_release}.tar.gz"
        mkdir -p /opt
        tar -xzf "yarn-v${yarn_release}.tar.gz" -C /opt
        ln -s /opt/yarn-v${yarn_release}/bin/yarn ${usr_local_bin}/yarn
        cd ${mydir}
    fi
}

function install_javascript_utilities() {
    # Install JS Toolchain, i.e. Typescript
    echo "Installing Javascript toolchain utilities"
    CI=true yarn workspaces focus hoprnet
}

if ${install_all}; then
    install_rustup
    install_cargo

    # launch rustc once so it installs updated components
    rustc --version

    install_wasm_opt
    install_wasm_pack
    install_protobuf
    install_node_js
    install_yarn
    install_javascript_utilities

    # We got yarn, so let's remove no longer necessary cache
    yarn cache clean --all
else
    # We only need Node.js
    install_node_js
    if ${with_yarn}; then
        install_yarn
        # Installation *with* yarn, so let's remove no longer necessary cache
        yarn cache clean --all
    fi
fi

# Show some debug output
echo ""
echo "Checking installed tool versions"
echo "================================"
echo ""
command -v rustc >/dev/null && rustc --version
command -v cargo >/dev/null && cargo --version
command -v wasm-pack >/dev/null && wasm-pack --version
command -v wasm-opt >/dev/null && wasm-opt --version
command -v node >/dev/null && echo "node $(node --version)"
command -v yarn >/dev/null && echo "yarn $(yarn --version)"
command -v protoc >/dev/null && protoc --version
npx --no tsc --version >/dev/null && echo "Typescript $(npx tsc --version)"
echo ""

rm -R ${download_dir}
