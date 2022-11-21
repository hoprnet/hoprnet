#!/bin/bash

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
# - Rust (rustc, cargo) -> $HOME/.cargo/bin
# - wasm-pack + wasm-opt, necessary to build WebAssembly modules -> $HOME/.cargo/bin
# 
# Currently tested for x86_64 and Alpine Linux based environments

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

export PATH=${PATH}:${mydir}/../../.cargo/bin

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
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --default-toolchain none -y
    fi
}

function install_cargo() {
    if ! command -v cargo; then
        declare rust_toolchain_toml_path="${mydir}/../../rust-toolchain.toml"
        declare rust_version=$(sed -En 's/^channel = "([0-9a-z.]*)"$/\1/p' ${rust_toolchain_toml_path})
        echo "Installing Cargo, Rust compiler v${rust_version}"
        declare target=$(sed -En 's/^targets = \[ "([a-z0-9-]*)" \]$/\1/p' ${rust_toolchain_toml_path})
        declare profile=$(sed -En 's/^profile = "([a-z]*)"$/\1/p' ${rust_toolchain_toml_path})

        # Always install the version specified in `rust-toolchain.toml`
        $HOME/.cargo/bin/rustup toolchain install ${rust_version} --profile ${profile} --target ${target}
        source "$HOME/.cargo/env"
    fi
}

function install_wasm_pack() {
    if ! command -v wasm-pack; then
        cd ${download_dir}
        echo "Installing wasm-pack"
        declare wasm_pack_release=$(curl 'https://api.github.com/repos/rustwasm/wasm-pack/releases/latest' | jq -r '.tag_name')
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
        local install_dir="${mydir}/../../.cargo"
        mkdir -p "${install_dir}/bin"
        cp "wasm-pack-${wasm_pack_release}-${cputype}-${ostype}/wasm-pack" "${install_dir}/bin"
        cd ${mydir}
    fi
}

# Used by wasm-pack to optimize WebAssembly binaries
function install_wasm_opt() {
    if ! command -v wasm-opt; then
        cd ${download_dir}
        echo "Installing wasm-opt"
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
        # Version 111 has no prebuilt binaries
        declare binaryen_release="version_110"
        #declare binaryen_release=$(curl 'https://api.github.com/repos/WebAssembly/binaryen/releases/latest'| jq -r '.tag_name')
        curl -fsSLO --compressed "https://github.com/WebAssembly/binaryen/releases/download/${binaryen_release}/binaryen-${binaryen_release}-${cputype}-${ostype}.tar.gz"
        tar -xzf "binaryen-${binaryen_release}-${cputype}-${ostype}.tar.gz"
        local install_dir="${mydir}/../../.cargo"
        mkdir -p "${install_dir}"
        mkdir -p "${install_dir}/bin"
        cp "binaryen-${binaryen_release}/bin/wasm-opt" "${install_dir}/bin"
        cp -R "binaryen-${binaryen_release}/lib" "${install_dir}"

        cd ${mydir}
    fi
}

function install_node_js() {
    if ! command -v node; then
        cd ${download_dir}
        # Downloads Node.js version specified in `.nvmrc` file
        declare nvmrc_path="${mydir}/../../.nvmrc"
        declare node_js_version=$(sed -En 's/^v*([0-9.])/\1/p' ${nvmrc_path})
        echo "Installing Node.js v${node_js_version}"
        # Using musl builds for alpine
        declare node_release=$(curl 'https://unofficial-builds.nodejs.org/download/release/index.json' | jq -r "[.[] | .version | select(. | startswith(\"v${node_js_version}\"))][0]")
        curl -fsSLO --compressed "https://unofficial-builds.nodejs.org/download/release/${node_release}/node-${node_release}-linux-x64-musl.tar.xz"
        tar -xJf "node-${node_release}-linux-x64-musl.tar.xz" -C /usr/local --strip-components=1 --no-same-owner
        cd ${mydir}
    fi
}

# Install legacy version of yarn that supports handover to yarn 2+
function install_yarn() {
    if ! command -v yarn; then
        cd ${download_dir}
        declare yarn_release="1.22.19"
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
    install_wasm_pack
    install_wasm_opt
    install_node_js
    install_yarn
    install_javascript_utilities

    # Show some debug output
    cargo --version
    echo "node $(node --version)"
    wasm-pack --version
    wasm-opt --version
    echo "yarn $(yarn --version)"
    echo "Typescript $(npx tsc --version)"
else
    # We only need Node.js
    install_node_js
    if ${with_yarn}; then
        install_yarn
    fi
fi

rm -R ${download_dir}
