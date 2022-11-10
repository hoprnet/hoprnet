# prevent sourcing of this script, only allow execution
$(return >/dev/null 2>&1)
test "$?" -eq "0" && { echo "This script should only be executed." >&2; exit 1; }

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -Eeuo pipefail

# set log id and use shared log function for readable logs
declare mydir
mydir=$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd -P)

# Installs all toolchain utilities that are required to build hoprnet monorepo, including
# - Node.js
# - Yarn
# - Typescript + related utilities, such as ts-node
# - Rust (rustc, cargo)
# - wasm-pack + wasm-opt, necessary to build WebAssembly modules

# @TODO adapt this script for macOS arm64 machines

if ! $(command -v rustup); then
    echo "Installing Rustup"
    # Get rustup but install toolchain later
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain none
    source "$HOME/.cargo/env"
fi

if ! $(command -v cargo); then
    declare rust_toolchain_toml_path="${mydir}/../../rust-toolchain.toml"
    declare rust_version=$(sed -En 's/^channel = "([0-9a-z.]*)"$/\1/p' ${rust_toolchain_toml_path})
    echo "Installing Cargo, Rust compiler v${rust_version}"
    declare target=$(sed -En 's/^targets = \[ "([a-z0-9-]*)" \]$/\1/p' ${rust_toolchain_toml_path})
    declare profile=$(sed -En 's/^profile = "([a-z]*)"$/\1/p' ${rust_toolchain_toml_path})

    # Always install the version specified in `rust-toolchain.toml`
    rustup toolchain install ${rust_version} --profile ${profile} --target ${target}
fi

declare download_dir="${mydir}/download"

mkdir ${download_dir}
cd ${download_dir}

if ! $(command -v wasm-pack); then
    echo "Installing wasm-pack"
    declare wasm_pack_release=$(curl 'https://api.github.com/repos/rustwasm/wasm-pack/releases/latest' | jq -r '.tag_name')
    curl -fsSLO --output-dir "${download_dir}" --compressed "https://github.com/rustwasm/wasm-pack/releases/download/${wasm_pack_release}/wasm-pack-${wasm_pack_release}-x86_64-unknown-linux-musl.tar.gz"
    tar -xzf "wasm-pack-${wasm_pack_release}-x86_64-unknown-linux-musl.tar.gz"
    cp "wasm-pack-${wasm_pack_release}-x86_64-unknown-linux-musl/wasm-pack" /usr/local/bin
fi

if ! $(command -v wasm-opt); then
    echo "Installing wasm-opt"
    declare binaryen_release=$(curl 'https://api.github.com/repos/WebAssembly/binaryen/releases/latest'| jq -r '.tag_name')
    curl -fsSLO --output-dir "${download_dir}" --compressed "https://github.com/WebAssembly/binaryen/releases/download/${binaryen_release}/binaryen-${binaryen_release}-x86_64-linux.tar.gz"
    tar -xzf "binaryen-${binaryen_release}-x86_64-linux.tar.gz"
    cp "binaryen-${binaryen_release}/bin/wasm-opt" /usr/local/bin
fi

if ! $(command -v node); then
    # Downloads Node.js version specified in `.nvmrc` file
    declare nvmrc_path="${mydir}/../../.nvmrc"
    declare node_js_version=$(sed -En 's/^v*([0-9.])/\1/p' ${nvmrc_path})
    echo "Installing Node.js v${node_js_version}"
    # Using musl builds for alpine
    declare node_release=$(curl 'https://unofficial-builds.nodejs.org/download/release/index.json' | jq -r "[.[] | .version | select(. | startswith(\"v${node_js_version}\"))][0]")
    curl -fsSLO --output-dir "${download_dir}" --compressed "https://unofficial-builds.nodejs.org/download/release/${node_release}/node-${node_release}-linux-x64-musl.tar.xz"
    tar -xJf "node-${node_release}-linux-x64-musl.tar.xz" -C /usr/local --strip-components=1 --no-same-owner
fi

if ! $(command -v yarn); then
    declare yarn_release="1.22.19"
    curl -fsSLO --output-dir "${download_dir}" --compressed "https://yarnpkg.com/downloads/${yarn_release}/yarn-v${yarn_release}.tar.gz"
    mkdir -p /opt
    tar -xzf "yarn-v${yarn_release}.tar.gz" -C /opt
    ln -s /opt/yarn-v${yarn_release}/bin/yarn /usr/local/bin/yarn
fi

cd ${mydir}
rm -R ${download_dir}

# Show some debug output
cargo --version
echo "node $(node --version)"
wasm-pack --version
wasm-opt --version
echo "yarn $(yarn --version)"

# Install JS Toolchain, i.e. Typescript
echo "Installing Javascript toolchain utilities"
CI=true yarn workspaces focus hoprnet
