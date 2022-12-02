.POSIX:

# Gets all packages that include a Rust crates
WORKSPACES_WITH_RUST_MODULES := $(wildcard $(addsuffix /crates, $(wildcard ./packages/*)))

# Gets all individual crates such that they can get built
CRATES := $(foreach crate,${WORKSPACES_WITH_RUST_MODULES},$(dir $(wildcard $(crate)/*/Cargo.toml)))

# add local Cargo install path and users' Cargo install path and use it as custom shell PATH (only once)
PATH := $(subst :${CURDIR}/.cargo/bin,,$(subst :${HOME}/.cargo/bin,,$(PATH))):${HOME}/.cargo/bin:${CURDIR}/.cargo/bin
SHELL := env PATH=$(PATH) $(shell which bash)

# use custom Cargo config file for each invocation
cargo := cargo --config ${CURDIR}/.cargo/config.toml

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


.PHONY: $(CRATES) ## builds all Rust crates with wasm-pack (except for foundry-tool)
$(CRATES):
# --out-dir is relative to working directory
	echo "use wasm-pack build"
	wasm-pack build --target=bundler --out-dir ./pkg $@

.PHONY: ./packages/ethereum/crates/foundry-tool/ ## builds foundry-tool Rust crates with cargo
./packages/ethereum/crates/foundry-tool/:
	echo "use cargo build" 
	cargo build --manifest-path $@/Cargo.toml
# install the package
	cargo install --path $@

.PHONY: $(WORKSPACES_WITH_RUST_MODULES) ## builds all WebAssembly modules
$(WORKSPACES_WITH_RUST_MODULES):
	$(MAKE) -C $@ install

.PHONY: deps-ci
deps-ci: ## Installs dependencies when running in CI
# GitHub Actions: fetch prebuilt sources
	${CURDIR}/scripts/toolchain/install-toolchain.sh
# we need to ensure cargo has built its local metadata for vendoring correctly, this is normally a no-op
	$(MAKE) cargo-update
	CI=true yarn workspaces focus ${YARNFLAGS}

.PHONY: deps-docker
deps-docker: ## Installs dependencies when building Docker images
# Toolchain dependencies are already installed using scripts/install-toolchain.sh script
ifeq ($(origin PRODUCTION),undefined)
# we need to ensure cargo has built its local metadata for vendoring correctly, this is normally a no-op
	$(MAKE) cargo-update
endif
	DEBUG= CI=true yarn workspaces focus ${YARNFLAGS}

.PHONY: deps
deps: ## Installs dependencies for development setup
	[ -n "${NIX_PATH}" ] || corepack enable
	command -v rustup && rustup update || echo "No rustup installed, ignoring"
# we need to ensure cargo has built its local metadata for vendoring correctly, this is normally a no-op
	mkdir -p .cargo/bin
	$(MAKE) cargo-update
	command -v wasm-pack || $(cargo) install wasm-pack
	command -v wasm-opt || $(cargo) install wasm-opt
	yarn workspaces focus ${YARNFLAGS}
# install foundry (cast + forge + anvil)
	$(MAKE) install-foundry


.PHONY: install-foundry
install-foundry: ## install foundry
	curl -L https://foundry.paradigm.xyz | bash
	foundryup

.PHONY: cargo-update
cargo-update: ## update vendored Cargo dependencies
	$(cargo) update

.PHONY: cargo-download
cargo-download: ## download vendored Cargo dependencies
	$(cargo) vendor --versioned-dirs vendor/cargo
	$(cargo) fetch

.PHONY: build
build: ## build all packages
build: build-hopr-admin build-yarn

.PHONY: build-hopr-admin
build-hopr-admin: ## build hopr admin React frontend
# Don't build hopr-admin e.g. for cover-traffic-daemon
ifeq ($(origin NO_NEXT),undefined)	
	yarn workspace @hoprnet/hoprd run buildAdmin
endif

.PHONY: build-solidity-types
build-solidity-types: ## generate Solidity typings
# FIXME: replace this with foundry binding generation
	echo "TODO: Foundry create binding"
# yarn workspace @hoprnet/hopr-ethereum run build:sol:types

.PHONY: build-yarn
build-yarn: ## build yarn packages
build-yarn: build-solidity-types build-cargo
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
# Skip building Rust crates
ifeq ($(origin NO_CARGO),undefined)
# First compile Rust crates and create bindings
	$(MAKE) -j 1 $(CRATES)
# Copy bindings to their destination
	$(MAKE) ${WORKSPACES_WITH_RUST_MODULES}
endif

.PHONY: build-yellowpaper
build-yellowpaper: ## build the yellowpaper in docs/yellowpaper
	$(MAKE) -C docs/yellowpaper

.PHONY: build-docs
build-docs: ## build typedocs, Rest API docs, and docs website
build-docs: build-docs-typescript build-docs-website build-docs-api

.PHONY: build-docs-typescript
build-docs-typescript: ## build typedocs
build-docs-typescript: build
	yarn workspaces foreach -pv run docs:generate

.PHONY: build-docs-website
build-docs-website: ## build docs website
	yarn workspace hopr-docs build

.PHONY: build-docs-api
build-docs-api: ## build Rest API docs
build-docs-api: build
	./scripts/build-rest-api-spec.sh

.PHONY: clean
clean: # Cleanup build directories (lib,build, ...etc.)
	yarn clean

.PHONY: reset
reset: # Performs cleanup & also deletes all "node_modules" directories
reset: clean
	yarn reset

.PHONY: test
test: ## run unit tests for all packages, or a single package if package= is set
ifeq ($(package),)
	yarn workspaces foreach -pv run test
else
# Prebuild Rust unit tests
	$(cargo) --frozen --offline build --tests
	yarn workspace @hoprnet/${package} run test
endif

.PHONY: lint-check
lint-check: ## run linter in check mode
	npx prettier --check .

.PHONY: lint-check
lint-fix: ## run linter in fix mode
	npx prettier --write .

.PHONY: run-anvil
run-anvil: ## spinup a local anvil instance (daemon) and deploy contracts
	echo "Spin up a local anvil instance and deploy all the contracts, incl. Erc1820Registry, to the local environment"
	anvil & \
	$(MAKE) -C packages/ethereum/contracts/ -j anvil-deploy-all

.PHONY: kill-anvil
kill-anvil: ## kill process running at port 8545 (default port of anvil)
	kill $(shell lsof -i :8545 | grep -o '\d*' | head -1)

s.PHONY: run-local
run-local: ## run HOPRd from local repo
	env NODE_OPTIONS="--experimental-wasm-modules" NODE_ENV=development DEBUG="hopr*" node \
		packages/hoprd/lib/main.cjs --admin --init --api \
		--password="local" --identity=`pwd`/.identity-local \
		--environment anvil-localhost --announce \
		--testUseWeakCrypto --testAnnounceLocalAddresses \
		--testPreferLocalAddresses --testNoAuthentication

.PHONY: docker-build-local
docker-build-local: ## build Docker images locally, or single image if image= is set
ifeq ($(image),)
	./scripts/build-docker.sh --local --force
else
	./scripts/build-docker.sh --local --force -i $(image)
endif

.PHONY: docker-build-gcb
docker-build-gcb: ## build Docker images on Google Cloud Build
	./scripts/build-docker.sh --no-tags --force

.PHONY: request-funds
request-funds: ensure-environment-is-set
request-funds: ## Request 1000 xHOPR tokens for the recipient
ifeq ($(recipient),)
	echo "parameter <recipient> missing" >&2 && exit 1
endif
ifeq ($(origin PRIVATE_KEY),undefined)
	echo "<PRIVATE_KEY> environment variable missing" >&2 && exit 1
endif
	make -C packages/ethereum/contracts request-funds environment-name=$(environment) environment-type=$(environment_type) recipient=$(recipient)

.PHONY: request-nrnft
request-nrnft: ensure-environment-is-set
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
	make -C packages/ethereum/contracts request-nrnft environment-name=$(environment) environment-type=$(environment_type) recipient=$(recipient) nftrank=$(nftrank)

.PHONY: stake-funds
stake-funds: ensure-environment-is-set
stake-funds: ## stake funds (idempotent operation)
ifeq ($(origin PRIVATE_KEY),undefined)
	echo "<PRIVATE_KEY> environment variable missing" >&2 && exit 1
endif
	make -C packages/ethereum/contracts stake-funds environment-name=$(environment) environment-type=$(environment_type)

.PHONY: stake-nrnft
stake-nrnft: ensure-environment-is-set
stake-nrnft: ## stake Network_registry NFTs (idempotent operation)
ifeq ($(nftrank),)
	echo "parameter <nftrank> missing, it can be either 'developer' or 'community'" >&2 && exit 1
endif
	make -C packages/ethereum/contracts stake-nrnft environment-name=$(environment) environment-type=$(environment_type) nftrank=$(nftrank)


enable-network-registry: ensure-environment-is-set
enable-network-registry: ## owner enables network registry (smart contract) globally
	make -C packages/ethereum/contracts enable-network-registry environment-name=$(environment) environment-type=$(environment_type)

disable-network-registry: ensure-environment-is-set
disable-network-registry: ## owner disables network registry (smart contract) globally
	make -C packages/ethereum/contracts disable-network-registry environment-name=$(environment) environment-type=$(environment_type)

force-eligibility-update: ensure-environment-is-set
force-eligibility-update: ## owner forces eligibility update
ifeq ($(native_addresses),)
	echo "parameter <native_addresses> missing" >&2 && exit 1
endif
ifeq ($(eligibility),)
	echo "parameter <eligibility> missing" >&2 && exit 1
endif
	make -C packages/ethereum/contracts force-eligibility-update \
	environment-name=$(environment) environment-type=$(environment_type) \
	staking_addresses="$(native_addresses)" eligibility="$(eligibility)"

sync-eligibility: ensure-environment-is-set
sync-eligibility: ## owner sync eligibility of peers
ifeq ($(peer_ids),)
	echo "parameter <peer_ids> missing" >&2 && exit 1
endif
	make -C packages/ethereum/contracts sync-eligibility \
	environment-name=$(environment) environment-type=$(environment_type) \
	peer_ids="$(peer_ids)"

register-nodes: ensure-environment-is-set
register-nodes: ## owner register given nodes in network registry contract
ifeq ($(native_addresses),)
	echo "parameter <native_addresses> missing" >&2 && exit 1
endif
ifeq ($(peer_ids),)
	echo "parameter <peer_ids> missing" >&2 && exit 1
endif
	make -C packages/ethereum/contracts register-nodes \
	environment-name=$(environment) environment-type=$(environment_type) \
	staking_addresses="$(native_addresses)" peer_ids="$(peer_ids)"

deregister-nodes: ensure-environment-is-set
deregister-nodes: ## owner de-register given nodes in network registry contract
ifeq ($(peer_ids),)
	echo "parameter <peer_ids> missing" >&2 && exit 1
endif
	make -C packages/ethereum/contracts deregister-nodes \
	environment-name=$(environment) environment-type=$(environment_type) \
	staking_addresses="$(native_addresses)" peer_ids="$(peer_ids)"

.PHONY: self-register-node
self-register-node: ensure-environment-is-set
self-register-node: ## staker register a node in network registry contract
ifeq ($(peer_ids),)
	echo "parameter <peer_ids> missing" >&2 && exit 1
endif
	make -C packages/ethereum/contracts self-register-node \
	environment-name=$(environment) environment-type=$(environment_type) \
	peer_ids="$(peer_ids)"

.PHONY: self-deregister-node
self-deregister-node: ensure-environment-is-set
self-deregister-node: ## staker deregister a node in network registry contract
ifeq ($(peer_ids),)
	echo "parameter <peer_ids> missing" >&2 && exit 1
endif
	make -C packages/ethereum/contracts self-deregister-node \
	environment-name=$(environment) environment-type=$(environment_type) \
	peer_ids="$(peer_ids)"

# .PHONY: register-node-when-dummy-proxy
# # DEPRECATED. Only use it when a dummy network registry proxy is in use
# # Register a node when a dummy proxy is in place of staking proxy
# # node_api?=localhost:3001 provide endpoint of hoprd, with a default value 'localhost:3001'
# register-node-when-dummy-proxy: ensure-environment-is-set
# ifeq ($(endpoint),)
# 	echo "parameter <endpoint> is default to localhost:3001" >&2
# endif
# ifeq ($(api_token),)
# 	echo "parameter <api_token> missing" >&2 && exit 1
# endif
# ifeq ($(account),)
# 	echo "parameter <account> missing" >&2 && exit 1
# endif
# ifeq ($(origin CI_DEPLOYER_PRIVKEY),undefined)
# 	echo "<CI_DEPLOYER_PRIVKEY> environment variable missing" >&2 && exit 1
# endif
# 	TS_NODE_PROJECT=./tsconfig.hardhat.json \
# 	HOPR_ENVIRONMENT_ID="$(environment)" \
# 	  yarn workspace @hoprnet/hopr-ethereum run hardhat register \
#    --network $(network) \
#    --task add \
#    --native-addresses "$(account)" \
#    --peer-ids "$(shell eval ./scripts/get-hopr-address.sh "$(api_token)" "$(endpoint)")" \
#    --privatekey "$(CI_DEPLOYER_PRIVKEY)"

.PHONY: register-node-with-nft
# node_api?=localhost:3001 provide endpoint of hoprd, with a default value 'localhost:3001'
register-node-with-nft: ensure-environment-is-set
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
register-node-with-stake: ensure-environment-is-set
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

ensure-environment-is-set:
ifeq ($(environment),)
	echo "parameter <environment> missing" >&2 && exit 1
else
environment_type != jq '.environments."$(environment)".environment_type // empty' packages/ethereum/contracts/contracts-addresses.json
ifeq ($(environment_type),)
	echo "could not read environment type info from contracts-addresses.json" >&2 && exit 1
endif
endif

.PHONY: help
help:
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' Makefile | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'
