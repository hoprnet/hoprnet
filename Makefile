.POSIX:

# utility variables
space := $(subst ,, )
mydir := $(dir $(abspath $(firstword $(MAKEFILE_LIST))))

# Gets all packages that include a Rust crates
# Disable automatic compilation of SC bindings. Can still be done manually.
WORKSPACES_WITH_RUST_MODULES := $(filter-out ./packages/ethereum/crates,$(wildcard $(addsuffix /crates, $(wildcard ./packages/*))))

# Gets all individual crates such that they can get built
CRATES := $(foreach crate,${WORKSPACES_WITH_RUST_MODULES},$(dir $(wildcard $(crate)/*/Cargo.toml)))

# base names of all crates
CRATES_NAMES := $(foreach crate,${CRATES},$(shell basename $(crate)))

# define specific crate for hopli which is a native helper
HOPLI_CRATE := ./packages/hopli

# Set local foundry directory (for binaries) and versions
# note: $(mydir) ends with '/'
FOUNDRY_DIR ?= $(mydir).foundry
# Use custom pinned version of foundry from
# https://github.com/hoprnet/foundry/tree/hopr-release
FOUNDRY_REPO := hoprnet/foundry
FOUNDRY_VSN := v0.0.4
FOUNDRYUP_VSN := fc64e18

# Set local cargo directory (for binaries)
# note: $(mydir) ends with '/'
CARGO_DIR := $(mydir).cargo

# use custom foundryup to ensure the local directory is used
foundryup := env FOUNDRY_DIR="${FOUNDRY_DIR}" foundryup

# add local Cargo install path (only once)
PATH := $(subst :${CARGO_DIR}/bin,,$(PATH)):${CARGO_DIR}/bin
# add users home Cargo install path (only once)
PATH := $(subst :${HOME}/.cargo/bin,,$(PATH)):${HOME}/.cargo/bin
# add local Foundry install path (only once)
PATH := $(subst :${FOUNDRY_DIR}/bin,,$(PATH)):${FOUNDRY_DIR}/bin
# use custom PATH in all shell processes, escape spaces
SHELL := env PATH=$(subst $(space),\$(space),$(PATH)) $(shell which bash)

# use custom Cargo config file for each invocation
cargo := cargo --config ${CARGO_DIR}/config.toml

# use custom flags for installing dependencies
YARNFLAGS :=

# Build specific package
ifeq ($(package),)
	YARNFLAGS := ${YARNFLAGS} -A
else
	YARNFLAGS := ${YARNFLAGS} @hoprnet/${package}
endif

# Don't install devDependencies in production
ifneq ($(origin PRODUCTION),undefined)
	YARNFLAGS := ${YARNFLAGS} --production
endif

all: help

.PHONY: init
init: ## initialize repository (idempotent operation)
	for gh in `find .githooks/ -type f`; do \
		ln -sf "../../$${gh}" .git/hooks/; \
	done

.PHONY: $(CRATES)
$(CRATES): ## builds all Rust crates with wasm-pack (except for hopli)
# --out-dir is relative to working directory
	echo "use wasm-pack build"
	wasm-pack build --target=bundler --out-dir ./pkg $@
	#env WASM_BINDGEN_WEAKREF=1 WASM_BINDGEN_EXTERNREF=1 \
		wasm-pack build --target=bundler --out-dir ./pkg $@

.PHONY: $(HOPLI_CRATE)
$(HOPLI_CRATE): ## builds hopli Rust crates with cargo
	echo "use cargo build"
	cargo build --manifest-path $@/Cargo.toml
# install the package
	cargo install --path $@ --force

.PHONY: $(WORKSPACES_WITH_RUST_MODULES)
$(WORKSPACES_WITH_RUST_MODULES): ## builds all WebAssembly modules
	$(MAKE) -C $@ install

.PHONY: deps-ci
deps-ci: ## Installs dependencies when running in CI
# we need to ensure cargo has built its local metadata for vendoring correctly, this is normally a no-op
	$(MAKE) cargo-update
	CI=true yarn workspaces focus ${YARNFLAGS}
# install foundry (cast + forge + anvil)
	$(MAKE) install-foundry

.PHONY: deps-docker
deps-docker: ## Installs dependencies when building Docker images
# Toolchain dependencies are already installed using scripts/install-toolchain.sh script
ifeq ($(origin PRODUCTION),undefined)
# we need to ensure cargo has built its local metadata for vendoring correctly, this is normally a no-op
	$(MAKE) cargo-update
endif
	DEBUG= CI=true yarn workspaces focus ${YARNFLAGS}

.PHONY: deps
deps: ## Installs dependencies for local setup
	if [[ ! "${name}" =~ nix-shell* ]]; then \
		corepack enable; \
		command -v rustup && rustup update || echo "No rustup installed, ignoring"; \
	fi
# we need to ensure cargo has built its local metadata for vendoring correctly, this is normally a no-op
	mkdir -p .cargo/bin
	$(MAKE) cargo-update
	command -v wasm-opt || $(cargo) install wasm-opt
	command -v wasm-pack || $(cargo) install wasm-pack
	yarn workspaces focus ${YARNFLAGS}
# install foundry (cast + forge + anvil)
	$(MAKE) install-foundry

.PHONY: install-foundry
install-foundry: ## install foundry
	mkdir -p "${FOUNDRY_DIR}/bin"
	mkdir -p "${FOUNDRY_DIR}/share/man/man1"
	@if [ -f "${FOUNDRY_DIR}/bin/foundryup" ]; then \
		echo "foundryup already installed under "${FOUNDRY_DIR}/bin", skipping"; \
	else \
		echo "installing foundryup (vsn ${FOUNDRYUP_VSN})"; \
		curl -L "https://raw.githubusercontent.com/${FOUNDRY_REPO}/${FOUNDRYUP_VSN}/foundryup/foundryup" > "${FOUNDRY_DIR}/bin/foundryup"; \
	  chmod +x "${FOUNDRY_DIR}/bin/foundryup"; \
	fi
	@if [ ! -f "${FOUNDRY_DIR}/bin/anvil" ] || [ ! -f "${FOUNDRY_DIR}/bin/cast" ] || [ ! -f "${FOUNDRY_DIR}/bin/forge" ]; then \
		echo "missing foundry binaries, installing via foundryup"; \
		$(foundryup) --repo ${FOUNDRY_REPO} --version ${FOUNDRY_VSN}; \
	else \
	  echo "foundry binaries already installed under "${FOUNDRY_DIR}/bin", skipping"; \
	fi
	@if [[ "${name}" =~ nix-shell* ]]; then \
		echo "Patching foundry binaries"; \
		patchelf --interpreter `cat ${NIX_CC}/nix-support/dynamic-linker` .foundry/bin/anvil || :; \
		patchelf --interpreter `cat ${NIX_CC}/nix-support/dynamic-linker` .foundry/bin/cast || :; \
		patchelf --interpreter `cat ${NIX_CC}/nix-support/dynamic-linker` .foundry/bin/forge || :; \
		patchelf --interpreter `cat ${NIX_CC}/nix-support/dynamic-linker` .foundry/bin/chisel || :; \
	fi
	@forge --version
	@anvil --version
	@chisel --version
	@cast --version

.PHONY: cargo-update
cargo-update: ## update vendored Cargo dependencies
	$(cargo) update

.PHONY: cargo-download
cargo-download: ## download vendored Cargo dependencies
	$(cargo) vendor --versioned-dirs vendor/cargo
	$(cargo) fetch

.PHONY: build
build: ## build all packages
build: build-yarn

.PHONY: build-solidity-types
build-solidity-types: ## generate Solidity typings
	echo "Foundry create binding"
	$(MAKE) -C packages/ethereum/contracts/ overwrite-sc-bindings
# Change git = "http://..." into version = "1.0.2"
# note: $(mydir) ends with '/'
	sed -i -e 's/https:\/\/github.com\/gakonst\/ethers-rs/1.0.2/g' $(mydir)packages/ethereum/crates/bindings/Cargo.toml
	sed -i -e 's/git/version/g' $(mydir)packages/ethereum/crates/bindings/Cargo.toml
# add [lib] as rlib is necessary to run integration tests
# note: $(mydir) ends with '/'
	echo -e "\n[lib] \ncrate-type = [\"cdylib\", \"rlib\"]" >> $(mydir)packages/ethereum/crates/bindings/Cargo.toml

.PHONY: build-yarn
build-yarn: ## build yarn packages
build-yarn: build-cargo
ifeq ($(package),)
	npx tsc --build tsconfig.build.json
else
	npx tsc --build packages/${package}/tsconfig.json
endif

.PHONY: build-yarn-watch
build-yarn-watch: ## build yarn packages (in watch mode)
build-yarn-watch: build-solidity-types build-cargo
	npx tsc --build tsconfig.build.json -w

.PHONY: build-cargo
build-cargo: ## build cargo packages and create boilerplate JS code
# build-cargo: build-solidity-types ## build cargo packages and create boilerplate JS code
# Skip building Rust crates
ifeq ($(origin NO_CARGO),undefined)
# First compile Rust crates and create bindings
# filter out proc-macro crates since they need no compilation
	$(MAKE) -j 1 $(filter-out %proc-macros/,$(CRATES))
# Copy bindings to their destination
# filter out proc-macro crates since they need no compilation
	$(MAKE) $(filter-out %proc-macros/,$(WORKSPACES_WITH_RUST_MODULES))
ifeq ($(origin NO_HOPLI),undefined)
# build hopli
	$(MAKE) $(HOPLI_CRATE)
endif
endif

.PHONY: build-yellowpaper
build-yellowpaper: ## build the yellowpaper in docs/yellowpaper
	$(MAKE) -C docs/yellowpaper

.PHONY: build-docs
build-docs: ## build typedocs, Rest API docs
build-docs: | build-docs-typescript build-docs-api

.PHONY: build-docs-typescript
build-docs-typescript: ## build typedocs
build-docs-typescript: build
	yarn workspaces foreach -pv run docs:generate

.PHONY: build-docs-api
build-docs-api: ## build Rest API docs
build-docs-api: build
	./scripts/build-rest-api-spec.sh

.PHONY: clean
clean: # Cleanup build directories (lib,build, ...etc.)
	cargo clean
	yarn clean

.PHONY: reset
reset: # Performs cleanup & also deletes all "node_modules" directories
reset: clean
	yarn reset

.PHONY: test
test: smart-contract-test ## run unit tests for all packages, or a single package if package= is set
ifeq ($(package),)
	yarn workspaces foreach -pv run test
	cargo test --no-default-features
# disabled until `wasm-bindgen-test-runner` supports ESM
# cargo test --target wasm32-unknown-unknow
else
	yarn workspace @hoprnet/${package} run test
	yarn workspace @hoprnet/${package} run test:wasm
endif

.PHONY: smoke-test
smoke-test: ## run smoke tests
	source .venv/bin/activate && (python3 -m pytest tests/ || (cat /tmp/hopr-smoke-test_integration.log && false))

.PHONY: smart-contract-test
smart-contract-test: # forge test smart contracts
	$(MAKE) -C packages/ethereum/contracts/ sc-test

.PHONY: lint
lint: lint-ts lint-rust lint-python lint-sol
lint: ## run linter for TS, Rust, Python, Solidity

.PHONY: lint-sol
lint-sol: ## run linter for Solidity
	$(foreach f, $(SOLIDITY_FILES), forge fmt --check $(f) && ) echo ""

.PHONY: lint-ts
lint-ts: ## run linter for TS
	npx prettier --check .

.PHONY: lint-rust
lint-rust: ## run linter for Rust
	$(foreach c, $(CRATES_NAMES), cargo fmt --check -p $(c) && ) echo ""

.PHONY: lint-python
lint-python: ## run linter for Python
	source .venv/bin/activate && ruff --fix . && black --check tests/

.PHONY: fmt
fmt: fmt-ts fmt-rust fmt-python
fmt: ## run code formatter for TS, Rust and Python

.PHONY: fmt-ts
fmt-ts: ## run code formatter for TS
	npx prettier --write .

.PHONY: fmt-rust
fmt-rust: ## run code formatter for Rust
	$(foreach c, $(CRATES_NAMES), cargo fmt -p $(c) && ) echo ""

.PHONY: fmt-python
fmt-python: ## run code formatter for Python
	source .venv/bin/activate && black tests/

.PHONY: run-anvil
run-anvil: args=
run-anvil: ## spinup a local anvil instance (daemon) and deploy contracts
	./scripts/run-local-anvil.sh $(args)

.PHONY: run-anvil-foreground
run-anvil-foreground: ## spinup a local anvil instance
	./scripts/run-local-anvil.sh -f -s

.PHONY: kill-anvil
kill-anvil: ## kill process running at port 8545 (default port of anvil)
	# may fail, we can ignore that
	lsof -i :8545 -s TCP:LISTEN -t | xargs -I {} -n 1 kill {} || :

.PHONY: run-local
run-local: args=
run-local: ## run HOPRd from local repo
	env NODE_OPTIONS="--experimental-wasm-modules" NODE_ENV=development DEBUG="hopr*" node \
		packages/hoprd/lib/main.cjs --init --api \
		--password="local" --identity=`pwd`/.identity-local.id \
		--network anvil-localhost --announce \
		--testUseWeakCrypto --testAnnounceLocalAddresses \
		--testPreferLocalAddresses --disableApiAuthentication \
		$(args)

run-local-dev-compose: ## run local development Compose setup
	echo "Starting Anvil on host"
	make kill-anvil
	ETHERSCAN_API_KEY=anykey make run-anvil
	echo "Starting Compose setup (grafana, prometheus)"
	cd scripts/compose && docker compose -f docker-compose.local-dev.yml up -d
	echo "Starting Hoprd from source on host"
	# hoprd must listen on the Docker bridge interface for Prometheus to be able
	# to connect to it
	make run-local args="--apiHost=0.0.0.0"

.PHONY: fund-local-all
fund-local-all: id_dir=/tmp/
fund-local-all: id_password=local
fund-local-all: id_prefix=
fund-local-all: ## use faucet script to fund all the local identities
	ETHERSCAN_API_KEY="" IDENTITY_PASSWORD="${id_password}" PRIVATE_KEY=0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 \
		hopli faucet \
		--network anvil-localhost \
		--identity-prefix "${id_prefix}" \
		--identity-directory "${id_dir}" \
		--contracts-root "./packages/ethereum/contracts"

.PHONY: docker-build-local
docker-build-local: ## build Docker images locally, or single image if image= is set
ifeq ($(image),)
	./scripts/build-docker.sh --local --force
else
	./scripts/build-docker.sh --local --force -i $(image)
endif

.PHONY: request-funds
request-funds: ensure-environment-and-network-are-set
request-funds: ## Request 1000 xHOPR tokens for the recipient
ifeq ($(recipient),)
	echo "parameter <recipient> missing" >&2 && exit 1
endif
ifeq ($(origin PRIVATE_KEY),undefined)
	echo "<PRIVATE_KEY> environment variable missing" >&2 && exit 1
endif
	make -C packages/ethereum/contracts request-funds network=$(network) environment-type=$(environment_type) recipient=$(recipient)

.PHONY: request-nrnft
request-nrnft: ensure-environment-and-network-are-set
request-nrnft: ## Request one HoprBoost Network_registry NFT for the recipient given it has none and hasn't staked Network_registry NFT
ifeq ($(recipient),)
	echo "parameter <recipient> missing" >&2 && exit 1
endif
ifeq ($(nftrank),)
	echo "parameter <nftrank> missing, it can be either 'developer' or 'community'" >&2 && exit 1
endif
ifeq ($(origin PRIVATE_KEY),undefined)
	echo "<PRIVATE_KEY> environment variable missing" >&2 && exit 1
endif
	make -C packages/ethereum/contracts request-nrnft network=$(network) environment-type=$(environment_type) recipient=$(recipient) nftrank=$(nftrank)

.PHONY: stake-funds
stake-funds: ensure-environment-and-network-are-set
stake-funds: ## stake funds (idempotent operation)
ifeq ($(origin PRIVATE_KEY),undefined)
	echo "<PRIVATE_KEY> environment variable missing" >&2 && exit 1
endif
	make -C packages/ethereum/contracts stake-funds network=$(network) environment-type=$(environment_type)

.PHONY: stake-nrnft
stake-nrnft: ensure-environment-and-network-are-set
stake-nrnft: ## stake Network_registry NFTs (idempotent operation)
ifeq ($(nftrank),)
	echo "parameter <nftrank> missing, it can be either 'developer' or 'community'" >&2 && exit 1
endif
	make -C packages/ethereum/contracts stake-nrnft network=$(network) environment-type=$(environment_type) nftrank=$(nftrank)


enable-network-registry: ensure-environment-and-network-are-set
enable-network-registry: ## owner enables network registry (smart contract) globally
	make -C packages/ethereum/contracts enable-network-registry network=$(network) environment-type=$(environment_type)

disable-network-registry: ensure-environment-and-network-are-set
disable-network-registry: ## owner disables network registry (smart contract) globally
	make -C packages/ethereum/contracts disable-network-registry network=$(network) environment-type=$(environment_type)

force-eligibility-update: ensure-environment-and-network-are-set
force-eligibility-update: ## owner forces eligibility update
ifeq ($(native_addresses),)
	echo "parameter <native_addresses> missing" >&2 && exit 1
endif
ifeq ($(eligibility),)
	echo "parameter <eligibility> missing" >&2 && exit 1
endif
	make -C packages/ethereum/contracts force-eligibility-update \
		network=$(network) environment-type=$(environment_type) \
		staking_addresses="$(native_addresses)" eligibility="$(eligibility)"

sync-eligibility: ensure-environment-and-network-are-set
sync-eligibility: ## owner sync eligibility of peers
ifeq ($(peer_ids),)
	echo "parameter <peer_ids> missing" >&2 && exit 1
endif
	make -C packages/ethereum/contracts sync-eligibility \
		network=$(network) environment-type=$(environment_type) \
		peer_ids="$(peer_ids)"

register-nodes: ensure-environment-and-network-are-set
register-nodes: ## owner register given nodes in network registry contract
ifeq ($(native_addresses),)
	echo "parameter <native_addresses> missing" >&2 && exit 1
endif
ifeq ($(peer_ids),)
	echo "parameter <peer_ids> missing" >&2 && exit 1
endif
	make -C packages/ethereum/contracts register-nodes \
		network=$(network) environment-type=$(environment_type) \
		staking_addresses="$(native_addresses)" peer_ids="$(peer_ids)"

deregister-nodes: ensure-environment-and-network-are-set
deregister-nodes: ## owner de-register given nodes in network registry contract
ifeq ($(peer_ids),)
	echo "parameter <peer_ids> missing" >&2 && exit 1
endif
	make -C packages/ethereum/contracts deregister-nodes \
		network=$(network) environment-type=$(environment_type) \
		staking_addresses="$(native_addresses)" peer_ids="$(peer_ids)"

.PHONY: self-register-node
self-register-node: ensure-environment-and-network-are-set
self-register-node: ## staker register a node in network registry contract
ifeq ($(peer_ids),)
	echo "parameter <peer_ids> missing" >&2 && exit 1
endif
	make -C packages/ethereum/contracts self-register-node \
		network=$(network) environment-type=$(environment_type) \
		peer_ids="$(peer_ids)"

.PHONY: self-deregister-node
self-deregister-node: ensure-environment-and-network-are-set
self-deregister-node: ## staker deregister a node in network registry contract
ifeq ($(peer_ids),)
	echo "parameter <peer_ids> missing" >&2 && exit 1
endif
	make -C packages/ethereum/contracts self-deregister-node \
		network=$(network) environment-type=$(environment_type) \
		peer_ids="$(peer_ids)"

.PHONY: register-node-with-nft
# node_api?=localhost:3001 provide endpoint of hoprd, with a default value 'localhost:3001'
register-node-with-nft: ensure-environment-and-network-are-set
ifeq ($(endpoint),)
	echo "parameter <endpoint> is default to localhost:3001" >&2
endif
ifeq ($(api_token),)
	echo "parameter <api_token> missing" >&2 && exit 1
endif
ifeq ($(nftrank),)
	echo "parameter <nftrank> missing, it can be either 'developer' or 'community'" >&2 && exit 1
endif
ifeq ($(account),)
	echo "parameter <account> missing" >&2 && exit 1
endif
ifeq ($(origin ACCOUNT_PRIVKEY),undefined)
	echo "<ACCOUNT_PRIVKEY> environment variable missing" >&2 && exit 1
endif
ifeq ($(origin DEV_BANK_PRIVKEY),undefined)
	echo "<DEV_BANK_PRIVKEY> environment variable missing" >&2 && exit 1
endif
	PRIVATE_KEY=${DEV_BANK_PRIVKEY} make request-nrnft recipient=${account} nftrank=${nftrank}
	PRIVATE_KEY=${ACCOUNT_PRIVKEY} make stake-nrnft nftrank=${nftrank}
	PRIVATE_KEY=${ACCOUNT_PRIVKEY} make self-register-node peer_ids=$(shell eval ./scripts/get-hopr-address.sh "$(api_token)" "$(endpoint)")

.PHONY: register-node-with-stake
# node_api?=localhost:3001 provide endpoint of hoprd, with a default value 'localhost:3001'
register-node-with-stake: ensure-environment-and-network-are-set
ifeq ($(account),)
	echo "parameter <account> missing" >&2 && exit 1
endif
ifeq ($(origin ACCOUNT_PRIVKEY),undefined)
	echo "<ACCOUNT_PRIVKEY> environment variable missing" >&2 && exit 1
endif
ifeq ($(origin DEV_BANK_PRIVKEY),undefined)
	echo "<DEV_BANK_PRIVKEY> environment variable missing" >&2 && exit 1
endif
	PRIVATE_KEY=${DEV_BANK_PRIVKEY} make request-funds recipient=${account}
	PRIVATE_KEY=${ACCOUNT_PRIVKEY} make stake-funds
	PRIVATE_KEY=${ACCOUNT_PRIVKEY} make self-register-node peer_ids=$(shell eval ./scripts/get-hopr-address.sh "$(api_token)" "$(endpoint)")

# These targets needs to be splitted in macOs systems
ensure-environment-and-network-are-set: ensure-network-is-set ensure-environment-is-set

ensure-network-is-set:
ifeq ($(network),)
	echo "parameter <network> missing" >&2 && exit 1
else
environment_type != jq '.networks."$(network)".environment_type // empty' packages/ethereum/contracts/contracts-addresses.json
endif

ensure-environment-is-set:
ifeq ($(environment_type),)
	echo "could not read environment info from packages/ethereum/contracts/contracts-addresses.json" >&2 && exit 1
endif

.PHONY: run-docker-dev
run-docker-dev: ## start a local development Docker container
	docker run -v `pwd`:/src -ti -w "/src" --name hoprd-local-dev nixos/nix nix \
		--extra-experimental-features nix-command \
		--extra-experimental-features flakes \
		develop

.PHONY: run-hopr-admin
run-hopr-admin: version=07aec21b
run-hopr-admin: port=3000
run-hopr-admin: ## launches HOPR Admin in a Docker container, supports port= and version=, use http://host.docker.internal to access the host machine
	docker run -p $(port):3000 --add-host=host.docker.internal:host-gateway \
		gcr.io/hoprassociation/hopr-admin:$(version)

.PHONY: exec-script
exec-script: ## execute given script= with the correct PATH set
ifeq ($(script),)
	echo "parameter <script> missing" >&2 && exit 1
endif
	bash "${script}"

.PHONY: help
help:
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' Makefile | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'
