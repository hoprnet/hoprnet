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

if [[ "${PATH}" =~ "/usr/local/bin" ]]; then
    echo "Cannot install utilities. \"/usr/local/bin\" is not part of home-path."
fi

if ! [ -w "/usr/local/bin" ]; then
    echo "Cannot install utilities. \"/usr/local/bin\" is not writable."
fi

if ! [ -d "/opt" ] && ! [ -w "/opt" ] || ! [ -w "/" ]; then
    echo "Cannot install utilities. \"/opt\" does not exist or is not writable."
fi

declare download_dir="/tmp/hopr-toolchain/download"
mkdir -p ${download_dir}

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

function install_node_js() {
    if ! command -v node; then
        cd ${download_dir}
        # Downloads Node.js version specified in `.nvmrc` file
        local nvmrc_path="${mydir}/../../.nvmrc"
        local node_js_version=$(sed -En 's/^v*([0-9.])/\1/p' ${nvmrc_path})
        echo "Installing Node.js v${node_js_version}"
        # Using musl builds for alpine
        local node_release=$(curl 'https://unofficial-builds.nodejs.org/download/release/index.json' | jq -r "[.[] | .version | select(. | startswith(\"v${node_js_version}\"))][0]")
        curl -fsSLO --compressed "https://unofficial-builds.nodejs.org/download/release/${node_release}/node-${node_release}-linux-x64-musl.tar.xz"
        tar -xJf "node-${node_release}-linux-x64-musl.tar.xz" -C /usr/local --strip-components=1 --no-same-owner
        cd ${mydir}
    fi
}

# Install legacy version of yarn that supports handover to yarn 2+
function install_yarn() {
    if ! command -v yarn; then
        cd ${download_dir}
        local yarn_release="1.22.19"
        curl -fsSLO --compressed "https://yarnpkg.com/downloads/${yarn_release}/yarn-v${yarn_release}.tar.gz"
        mkdir -p /opt
        tar -xzf "yarn-v${yarn_release}.tar.gz" -C /opt
        ln -s /opt/yarn-v${yarn_release}/bin/yarn /usr/local/bin/yarn
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

    install_wasm_pack
    install_wasm_opt
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
npx --no tsc --version >/dev/null && echo "Typescript $(npx tsc --version)"
echo ""

rm -R ${download_dir}
