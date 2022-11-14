.POSIX:

# Gets all packages that include a Rust crates
WORKSPACES_WITH_RUST_MODULES := $(wildcard $(addsuffix /crates, $(wildcard ./packages/*)))

# Gets all individual crates such that they can get built
CRATES := $(foreach crate,${WORKSPACES_WITH_RUST_MODULES},$(dir $(wildcard $(crate)/*/Cargo.toml)))

# add local Cargo install path
PATH := "${PATH}:`pwd`/.bin"

all: help

.PHONY: $(CRATES) ## builds all Rust crates
$(CRATES):
# --out-dir is relative to working directory
	wasm-pack build --target=bundler --out-dir ./pkg $@

.PHONY: $(WORKSPACES_WITH_RUST_MODULES) ## builds all WebAssembly modules
$(WORKSPACES_WITH_RUST_MODULES):
	$(MAKE) -C $@ install

.PHONY: deps
deps: ## install dependencies
	# only use corepack on non-nix systems
	[ -n "${NIX_PATH}" ] || corepack enable
	command -v rustup && rustup update || echo "No rustup installed, ignoring"
	command -v wasm-pack || cargo install wasm-pack
	echo ${PATH}
	yarn

.PHONY: build
build: ## build all packages
build: build-hopr-admin build-yarn

.PHONY: build-hopr-admin
build-hopr-admin: ## build hopr admin React frontend
	yarn workspace @hoprnet/hoprd run buildAdmin

.PHONY: build-solidity-types
build-solidity-types: ## generate Solidity typings
	yarn workspace @hoprnet/hopr-ethereum run build:sol:types

.PHONY: build-yarn
build-yarn: ## build yarn packages
build-yarn: build-solidity-types build-cargo
	npx tsc --build tsconfig.build.json

.PHONY: build-yarn-watch
build-yarn-watch: ## build yarn packages (in watch mode)
build-yarn-watch: build-solidity-types build-cargo
	npx tsc --build tsconfig.build.json -w

.PHONY: build-cargo
build-cargo: ## build cargo packages and create boilerplate JS code
# First compile Rust crates and create bindings
	$(MAKE) -j 1 $(CRATES)
# Copy bindings to their destination
	$(MAKE) ${WORKSPACES_WITH_RUST_MODULES}

.PHONY: build-yellowpaper
build-yellowpaper: ## build the yellowpaper in docs/yellowpaper
	make -C docs/yellowpaper

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
	cargo build --tests
	yarn workspace @hoprnet/${package} run test
endif

.PHONY: lint-check
lint-check: ## run linter in check mode
	npx prettier --check .

.PHONY: lint-check
lint-fix: ## run linter in fix mode
	npx prettier --write .

.PHONY: run-hardhat
run-hardhat: ## run local hardhat environment
	cd packages/ethereum && \
		env NODE_OPTIONS="--experimental-wasm-modules" NODE_ENV=development \
		TS_NODE_PROJECT=./tsconfig.hardhat.json \
		HOPR_ENVIRONMENT_ID=hardhat-localhost yarn run hardhat node \
		--network hardhat --show-stack-traces

.PHONY: run-local
run-local: ## run HOPRd from local repo
	env NODE_OPTIONS="--experimental-wasm-modules" NODE_ENV=development node \
		packages/hoprd/lib/main.cjs --admin --init --api \
		--password="local" --identity=`pwd`/.identity-local \
		--environment hardhat-localhost --announce \
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
	TS_NODE_PROJECT=./tsconfig.hardhat.json \
	HOPR_ENVIRONMENT_ID="$(environment)" \
	  yarn workspace @hoprnet/hopr-ethereum run hardhat request-test-tokens \
   --network $(network) \
   --type xhopr \
   --amount 1000000000000000000000 \
   --recipient $(recipient) \
   --privatekey "$(PRIVATE_KEY)"

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
	TS_NODE_PROJECT=./tsconfig.hardhat.json \
	HOPR_ENVIRONMENT_ID="$(environment)" \
	  yarn workspace @hoprnet/hopr-ethereum run hardhat request-test-tokens \
   --network $(network) \
   --type nrnft \
   --nftrank $(nftrank) \
   --recipient $(recipient) \
   --privatekey "$(PRIVATE_KEY)"

.PHONY: stake-funds
stake-funds: ensure-environment-is-set
stake-funds: ## stake funds (idempotent operation)
ifeq ($(origin PRIVATE_KEY),undefined)
	echo "<PRIVATE_KEY> environment variable missing" >&2 && exit 1
endif
	@TS_NODE_PROJECT=./tsconfig.hardhat.json \
	HOPR_ENVIRONMENT_ID="$(environment)" \
		yarn workspace @hoprnet/hopr-ethereum run hardhat stake \
		--network $(network) \
		--type xhopr \
		--amount 1000000000000000000000 \
		--privatekey "$(PRIVATE_KEY)"

.PHONY: stake-nrnft
stake-nrnft: ensure-environment-is-set
stake-nrnft: ## stake Network_registry NFTs (idempotent operation)
ifeq ($(nftrank),)
	echo "parameter <nftrank> missing, it can be either 'developer' or 'community'" >&2 && exit 1
endif
ifeq ($(origin PRIVATE_KEY),undefined)
	@TS_NODE_PROJECT=./tsconfig.hardhat.json \
	HOPR_ENVIRONMENT_ID="$(environment)" \
		yarn workspace @hoprnet/hopr-ethereum run hardhat stake \
		--network $(network) \
		--type nrnft \
		--nftrank $(nftrank)
else
	@TS_NODE_PROJECT=./tsconfig.hardhat.json \
	HOPR_ENVIRONMENT_ID="$(environment)" \
		yarn workspace @hoprnet/hopr-ethereum run hardhat stake \
		--network $(network) \
		--type nrnft \
		--nftrank $(nftrank) \
		--privatekey "$(PRIVATE_KEY)"
endif

enable-network-registry: ensure-environment-is-set
enable-network-registry: ## owner enables network registry (smart contract) globally
	TS_NODE_PROJECT=./tsconfig.hardhat.json \
	HOPR_ENVIRONMENT_ID="$(environment)" \
	  yarn workspace @hoprnet/hopr-ethereum run hardhat register \
   --network $(network) \
   --task enable

disable-network-registry: ensure-environment-is-set
disable-network-registry: ## owner disables network registry (smart contract) globally
	TS_NODE_PROJECT=./tsconfig.hardhat.json \
	HOPR_ENVIRONMENT_ID="$(environment)" \
	  yarn workspace @hoprnet/hopr-ethereum run hardhat register \
   --network $(network) \
   --task disable

force-eligibility-update: ensure-environment-is-set
force-eligibility-update: ## owner forces eligibility update
ifeq ($(native_addresses),)
	echo "parameter <native_addresses> missing" >&2 && exit 1
endif
ifeq ($(eligibility),)
	echo "parameter <eligibility> missing" >&2 && exit 1
endif
	TS_NODE_PROJECT=./tsconfig.hardhat.json \
	HOPR_ENVIRONMENT_ID="$(environment)" \
	  yarn workspace @hoprnet/hopr-ethereum run hardhat register \
   --network $(network) \
   --task force-eligibility-update \
   --native-addresses "$(native_addresses)"\
   --eligibility "$(eligibility)"

sync-eligibility: ensure-environment-is-set
sync-eligibility: ## owner sync eligibility of peers
ifeq ($(peer_ids),)
	echo "parameter <peer_ids> missing" >&2 && exit 1
endif
	TS_NODE_PROJECT=./tsconfig.hardhat.json \
	HOPR_ENVIRONMENT_ID="$(environment)" \
	  yarn workspace @hoprnet/hopr-ethereum run hardhat register \
   --network $(network) \
   --task sync \
   --peer-ids "$(peer_ids)"

register-nodes: ensure-environment-is-set
register-nodes: ## owner register given nodes in network registry contract
ifeq ($(native_addresses),)
	echo "parameter <native_addresses> missing" >&2 && exit 1
endif
ifeq ($(peer_ids),)
	echo "parameter <peer_ids> missing" >&2 && exit 1
endif
	TS_NODE_PROJECT=./tsconfig.hardhat.json \
	HOPR_ENVIRONMENT_ID="$(environment)" \
	  yarn workspace @hoprnet/hopr-ethereum run hardhat register \
   --network $(network) \
   --task add \
   --native-addresses "$(native_addresses)" \
   --peer-ids "$(peer_ids)"

deregister-nodes: ensure-environment-is-set
deregister-nodes: ## owner de-register given nodes in network registry contract
ifeq ($(peer_ids),)
	echo "parameter <peer_ids> missing" >&2 && exit 1
endif
ifeq ($(native_addresses),)
	echo "no parameter <native_addresses>" >&2
	TS_NODE_PROJECT=./tsconfig.hardhat.json \
	HOPR_ENVIRONMENT_ID="$(environment)" \
	  yarn workspace @hoprnet/hopr-ethereum run hardhat register \
   --network $(network) \
   --task remove \
   --peer-ids "$(peer_ids)"
else
	TS_NODE_PROJECT=./tsconfig.hardhat.json \
	HOPR_ENVIRONMENT_ID="$(environment)" \
	  yarn workspace @hoprnet/hopr-ethereum run hardhat register \
   --network $(network) \
   --task remove \
   --peer-ids "$(peer_ids)" \
   --native-addresses "$(native_addresses)"
endif

.PHONY: self-register-node
self-register-node: ensure-environment-is-set
self-register-node: ## staker register a node in network registry contract
ifeq ($(peer_ids),)
	echo "parameter <peer_ids> missing" >&2 && exit 1
endif
ifeq ($(origin PRIVATE_KEY),undefined)
	TS_NODE_PROJECT=./tsconfig.hardhat.json \
	HOPR_ENVIRONMENT_ID="$(environment)" \
	  yarn workspace @hoprnet/hopr-ethereum run hardhat register:self \
   --network $(network) \
   --task add \
   --peer-ids "$(peer_ids)"
else
	TS_NODE_PROJECT=./tsconfig.hardhat.json \
	HOPR_ENVIRONMENT_ID="$(environment)" \
	  yarn workspace @hoprnet/hopr-ethereum run hardhat register:self \
   --network $(network) \
   --task add \
   --peer-ids "$(peer_ids)" \
   --privatekey "$(PRIVATE_KEY)"
endif


.PHONY: self-deregister-node
self-deregister-node: ensure-environment-is-set
self-deregister-node: ## staker deregister a node in network registry contract
ifeq ($(peer_ids),)
	echo "parameter <peer_ids> missing" >&2 && exit 1
endif
	TS_NODE_PROJECT=./tsconfig.hardhat.json \
	HOPR_ENVIRONMENT_ID="$(environment)" \
	  yarn workspace @hoprnet/hopr-ethereum run hardhat register:self \
   --network $(network) \
   --task remove \
   --peer-ids "$(peer_ids)" \
   --privatekey "$(PRIVATE_KEY)"

.PHONY: register-node-when-dummy-proxy
# DEPRECATED. Only use it when a dummy network registry proxy is in use
# Register a node when a dummy proxy is in place of staking proxy
# node_api?=localhost:3001 provide endpoint of hoprd, with a default value 'localhost:3001'
register-node-when-dummy-proxy: ensure-environment-is-set
ifeq ($(endpoint),)
	echo "parameter <endpoint> is default to localhost:3001" >&2
endif
ifeq ($(api_token),)
	echo "parameter <api_token> missing" >&2 && exit 1
endif
ifeq ($(account),)
	echo "parameter <account> missing" >&2 && exit 1
endif
ifeq ($(origin CI_DEPLOYER_PRIVKEY),undefined)
	echo "<CI_DEPLOYER_PRIVKEY> environment variable missing" >&2 && exit 1
endif
	TS_NODE_PROJECT=./tsconfig.hardhat.json \
	HOPR_ENVIRONMENT_ID="$(environment)" \
	  yarn workspace @hoprnet/hopr-ethereum run hardhat register \
   --network $(network) \
   --task add \
   --native-addresses "$(account)" \
   --peer-ids "$(shell eval ./scripts/get-hopr-address.sh "$(api_token)" "$(endpoint)")" \
   --privatekey "$(CI_DEPLOYER_PRIVKEY)"

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
network != jq '.environments."$(environment)".network_id // empty' packages/core/protocol-config.json
ifeq ($(network),)
	echo "could not read environment info from protocol-config.json" >&2 && exit 1
endif
endif

.PHONY: help
help:
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' Makefile | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'
