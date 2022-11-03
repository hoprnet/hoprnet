$(return >/dev/null 2>&1)
test "$?" -eq "0" && { echo "This script should only be executed." >&2; exit 1; }

# set log id and use shared log function for readable logs
declare mydir
mydir=$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd -P)

if [ -z $(command -v rustup) ]; then
    # Get rustup but install toolchain later
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain none
    source "$HOME/.cargo/env"
fi

if [ -z $(command -v cargo) ]; then
    declare rust_version=$(sed -En 's/^channel = "([0-9a-z.]*)"$/\1/p' ${mydir}/../rust-toolchain.toml)
    declare target=$(sed -En 's/^targets = \[ "([a-z0-9-]*)" \]$/\1/p' ${mydir}/../rust-toolchain.toml)
    declare profile=$(sed -En 's/^profile = "([a-z]*)"$/\1/p' ${mydir}/../rust-toolchain.toml)

    # Always install the version specified in `rust-toolchain.toml`
    rustup toolchain install ${rust_version} --profile ${profile} --target ${target}
fi


declare download_dir="${mydir}/download"

mkdir ${download_dir}

if [ -z $(command -v wasm-pack) ]; then
    declare wasm_pack_release=$(curl 'https://api.github.com/repos/rustwasm/wasm-pack/releases/latest' | jq -r '.tag_name')
    curl -fsSLO --output-dir "${download_dir}" --compressed "https://github.com/rustwasm/wasm-pack/releases/download/${wasm_pack_release}/wasm-pack-${wasm_pack_release}-x86_64-unknown-linux-musl.tar.gz"
    cd ${download_dir}
    tar -xzf "wasm-pack-${wasm_pack_release}-x86_64-unknown-linux-musl.tar.gz"
    cp "wasm-pack-${wasm_pack_release}-x86_64-unknown-linux-musl/wasm-pack" /usr/local/bin
    cd ${mydir}
fi

if [ -z $(command -v wasm-opt) ]; then
    declare binaryen_release=$(curl 'https://api.github.com/repos/WebAssembly/binaryen/releases/latest'| jq -r '.tag_name')
    curl -fsSLO --output-dir "${download_dir}" --compressed "https://github.com/WebAssembly/binaryen/releases/download/${binaryen_release}/binaryen-${binaryen_release}-x86_64-linux.tar.gz"
    cd ${download_dir}
    tar -xzf "binaryen-${binaryen_release}-x86_64-linux.tar.gz"
    cp "binaryen-${binaryen_release}/bin/wasm-opt" /usr/local/bin
    cd ${mydir}
fi

if [ -z $(command -v node) ]; then
    # Downloads Node.js version specified in `.nvmrc` file
    declare node_js_version=$(sed -En 's/^v*([0-9.])/\1/p' ${mydir}/../.nvmrc)
    declare node_release=$(curl 'https://unofficial-builds.nodejs.org/download/release/index.json' | jq -r "[.[] | .version | select(. | startswith(\"v${node_js_version}\"))][0]")
    curl -fsSLO --output-dir "${download_dir}" --compressed "https://unofficial-builds.nodejs.org/download/release/${node_release}/node-${node_release}-linux-x64-musl.tar.xz"
    cd ${download_dir}
    tar -xJf "node-${node_release}-linux-x64-musl.tar.xz" -C /usr/local --strip-components=1 --no-same-owner
    cd ${mydir}
fi

if [ -z $(command -v yarn) ]; then
    declare yarn_release="1.22.19"
    curl -fsSLO --output-dir "${download_dir}" --compressed "https://yarnpkg.com/downloads/${yarn_release}/yarn-v${yarn_release}.tar.gz"
    cd ${download_dir}
    mkdir -p /opt
    tar -xzf "yarn-v${yarn_release}.tar.gz" -C /opt
    ln -s /opt/yarn-v${yarn_release}/bin/yarn /usr/local/bin/yarn
    cd ${mydir}
fi

rm -R ${download_dir}


# Show some debug output
cargo --version
node --version
wasm-pack --version
wasm-opt --version
yarn --version

# JS Toolchain, i.e. Typescript
CI=true yarn workspaces focus hoprnet
