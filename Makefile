.POSIX:

# utility variables
space := $(subst ,, )
mydir := $(dir $(abspath $(firstword $(MAKEFILE_LIST))))

# Set local cargo directory (for binaries)
# note: $(mydir) ends with '/'
CARGO_DIR := $(mydir).cargo

# add local Cargo install path (only once)
PATH := $(subst :${CARGO_DIR}/bin,,$(PATH)):${CARGO_DIR}/bin
# add users home Cargo install path (only once)
PATH := $(subst :${HOME}/.cargo/bin,,$(PATH)):${HOME}/.cargo/bin
# add nix build result path (only once)
PATH := $(subst :$(mydir)/result/bin,,$(PATH)):$(mydir)/result/bin
# use custom PATH in all shell processes
# escape spaces
SHELL := env PATH=$(subst $(space),\$(space),$(PATH)) $(shell which bash)

# use custom Cargo config file for each invocation
cargo := cargo --config ${CARGO_DIR}/config.toml

all: help

.PHONY: init
init: ## initialize repository (idempotent operation)
	for gh in `find .githooks/ -type f`; do \
		ln -sf "../../$${gh}" .git/hooks/; \
	done

.PHONY: build-yellowpaper
build-yellowpaper: ## build the yellowpaper in docs/yellowpaper
	$(MAKE) -C docs/yellowpaper

.PHONY: install
install:
	$(cargo) install --path hopli
	$(cargo) install --path hoprd/hoprd

.PHONY: test
test: smart-contract-test ## run unit tests for all packages, or a single package if package= is set
	$(cargo) test --features runtime-tokio

.PHONY: stress-test-local-swarm
stress-test-local-swarm: ## run stress tests on a local node swarm
	uv run -m pytest tests/test_stress.py \
		--stress-request-count=3000 \
		--stress-sources='[{"url": "localhost:3011", "token": "e2e-API-token^^"}]' \
		--stress-target='{"url": "localhost:3031", "token": "e2e-API-token^^"}'

.PHONY: smart-contract-test
# Remove `--no-match-test` when https://github.com/foundry-rs/foundry/issues/10586 is fixed
smart-contract-test: # forge test smart contracts
	$(MAKE) -C ethereum/contracts/ sc-test

.PHONY: run-anvil
run-anvil: args=
run-anvil: ## spinup a local anvil instance (daemon) and deploy contracts
	./scripts/run-local-anvil.sh $(args)

.PHONY: run-anvil-foreground
run-anvil-foreground: ## spinup a local anvil instance
	./scripts/run-local-anvil.sh -f -s

.PHONY: kill-anvil
kill-anvil: port=8545
kill-anvil: ## kill process running at port 8545 (default port of anvil)
	# may fail, we can ignore that
	lsof -i :$(port) -s TCP:LISTEN -t | xargs -I {} -n 1 kill {} || :

.PHONY: localcluster
localcluster: args=
localcluster: ## spin up the localcluster using the default configuration file
	@uv run -m sdk.python.localcluster --config ./sdk/python/localcluster.params.yml --fully_connected $(args)

.PHONY: localcluster-exposed
localcluster-exposed: ## spin up the localcluster using the default configuration file, exposing all nodes in the local network
	@make localcluster args="--exposed"

.PHONY: create-local-identity
create-local-identity: id_dir=/tmp/
create-local-identity: id_password=local
create-local-identity: id_prefix=.identity-local_
create-local-identity: id_count=1
create-local-identity: ## run HOPRd from local repo
	if [ ! -f "${id_dir}${id_prefix}0.id" ]; then \
		ETHERSCAN_API_KEY="anykey" IDENTITY_PASSWORD="${id_password}" \
		hopli identity create \
		--identity-directory "${id_dir}" \
		--identity-prefix "${id_prefix}" \
		--number ${id_count}; \
	fi

.PHONY: run-local
run-local: id_path=$$(pwd)/.identity-local.id
run-local: network=anvil-localhost
run-local: args=
run-local: ## run HOPRd from local repo
	hoprd --init --api \
		--password="local" --identity="${id_path}" \
		--network "${network}" --announce \
		--testAnnounceLocalAddresses --testPreferLocalAddresses \
		--disableApiAuthentication \
		--protocolConfig $(mydir)scripts/protocol-config-anvil.json \
		--data /tmp/ \
		$(args)

.PHONY: run-local-with-safe
run-local-with-safe: network=anvil-localhost
run-local-with-safe: id_dir=/tmp/
run-local-with-safe: ## run HOPRd from local repo. use the most recently created id file as node. create a safe and a module for the said node
	id_path=`find $(id_dir) -name ".identity-local*.id" | sort -r | head -n 1`; \
	args=`make create-safe-module id_path="$${id_path}" | grep -oE "(--safeAddress.*)"`; \
		 make run-local id_path="$${id_path}" network="${network}" args="$${args}"

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
fund-local-all: provider_url=http://localhost:8545
fund-local-all: ## use faucet script to fund all the local identities
	ETHERSCAN_API_KEY="anykey" IDENTITY_PASSWORD="${id_password}" PRIVATE_KEY=ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 \
		hopli faucet \
		--network anvil-localhost \
		--contracts-root "./ethereum/contracts" \
		--identity-prefix "${id_prefix}" \
		--identity-directory "${id_dir}" \
		--provider-url "${provider_url}"

.PHONY: create-safe-module-all
create-safe-module-all: id_dir=/tmp/
create-safe-module-all: id_password=local
create-safe-module-all: id_prefix=
create-safe-module-all: ## create a safe and a module and add all the nodes from local identities to the module
	ETHERSCAN_API_KEY="anykey" IDENTITY_PASSWORD="${id_password}" PRIVATE_KEY=ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 \
		hopli safe-module create \
		--network anvil-localhost \
		--contracts-root "./ethereum/contracts" \
		--identity-prefix "${id_prefix}" \
		--identity-directory "${id_dir}" \
		--hopr-amount 1000 --native-amount 1 \
		--manager-private-key ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80

.PHONY: create-safe-module
create-safe-module: id_password=local
create-safe-module: id_path=/tmp/local-alice.id
create-safe-module: hopr_amount=10
create-safe-module: native_amount=1
create-safe-module: ## create a safe and a module, and add a node to the module
	ETHERSCAN_API_KEY="anykey" IDENTITY_PASSWORD="${id_password}" PRIVATE_KEY=ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 \
		hopli safe-module create \
		--network anvil-localhost \
		--contracts-root "./ethereum/contracts" \
		--identity-from-path "${id_path}" \
		--hopr-amount ${hopr_amount} --native-amount ${native_amount} \
		--manager-private-key ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80

.PHONY: deploy-safe-module
deploy-safe-module: id_password=local
deploy-safe-module: id_path=/tmp/local-alice.id
deploy-safe-module: network=rotsee
deploy-safe-module: ## Deploy a safe and a module, and add a node to the module
ifeq ($(origin PRIVATE_KEY),undefined)
	echo "<PRIVATE_KEY> environment variable missing" >&2 && exit 1
endif
	ETHERSCAN_API_KEY="anykey" IDENTITY_PASSWORD="${id_password}" PRIVATE_KEY="${PRIVATE_KEY}" \
		hopli safe-module create \
		--network "${network}" \
		--identity-from-path "${id_path}" \
		--contracts-root "./ethereum/contracts" \
		--manager-private-key ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80

.PHONY: request-funds
request-funds: ensure-environment-and-network-are-set
request-funds: ## Request 1000 xHOPR tokens for the recipient
ifeq ($(recipient),)
	echo "parameter <recipient> missing" >&2 && exit 1
endif
ifeq ($(origin PRIVATE_KEY),undefined)
	echo "<PRIVATE_KEY> environment variable missing" >&2 && exit 1
endif
	make -C ethereum/contracts request-funds network=$(network) environment-type=$(environment_type) recipient=$(recipient)

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
	make -C ethereum/contracts request-nrnft network=$(network) environment-type=$(environment_type) recipient=$(recipient) nftrank=$(nftrank)

.PHONY: stake-funds
stake-funds: ensure-environment-and-network-are-set
stake-funds: ## stake funds (idempotent operation)
ifeq ($(origin PRIVATE_KEY),undefined)
	echo "<PRIVATE_KEY> environment variable missing" >&2 && exit 1
endif
	make -C ethereum/contracts stake-funds network=$(network) environment-type=$(environment_type)

.PHONY: stake-nrnft
stake-nrnft: ensure-environment-and-network-are-set
stake-nrnft: ## stake Network_registry NFTs (idempotent operation)
ifeq ($(nftrank),)
	echo "parameter <nftrank> missing, it can be either 'developer' or 'community'" >&2 && exit 1
endif
	make -C ethereum/contracts stake-nrnft network=$(network) environment-type=$(environment_type) nftrank=$(nftrank)

enable-network-registry: ensure-environment-and-network-are-set
enable-network-registry: ## owner enables network registry (smart contract) globally
	make -C ethereum/contracts enable-network-registry network=$(network) environment-type=$(environment_type)

disable-network-registry: ensure-environment-and-network-are-set
disable-network-registry: ## owner disables network registry (smart contract) globally
	make -C ethereum/contracts disable-network-registry network=$(network) environment-type=$(environment_type)

sync-eligibility: ensure-environment-and-network-are-set
sync-eligibility: ## owner sync eligibility of peers
ifeq ($(staking_addresses),)
	echo "parameter <staking_addresses> missing" >&2 && exit 1
endif
	make -C ethereum/contracts sync-eligibility \
		network=$(network) environment-type=$(environment_type) \
		staking_addresses="$(staking_addresses)"

register-nodes: ensure-environment-and-network-are-set
register-nodes: ## manager register nodes and safes in network registry contract
ifeq ($(staking_addresses),)
	echo "parameter <staking_addresses> missing" >&2 && exit 1
endif
ifeq ($(node_addresses),)
	echo "parameter <node_addresses> missing" >&2 && exit 1
endif
	make -C ethereum/contracts register-nodes \
		network=$(network) environment-type=$(environment_type) \
		staking_addresses="$(staking_addresses)" node_addresses="$(node_addresses)"

deregister-nodes: ensure-environment-and-network-are-set
deregister-nodes: ## owner de-register given nodes in network registry contract
ifeq ($(node_addresses),)
	echo "parameter <node_addresses> missing" >&2 && exit 1
endif
	make -C ethereum/contracts deregister-nodes \
		network=$(network) environment-type=$(environment_type) \
		node_addresses="$(node_addresses)"

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
environment_type != jq '.networks."$(network)".environment_type // empty' ethereum/contracts/contracts-addresses.json
endif

ensure-environment-is-set:
ifeq ($(environment_type),)
	echo "could not read environment info from ethereum/contracts/contracts-addresses.json" >&2 && exit 1
endif

.PHONY: run-docker-dev
run-docker-dev: ## start a local development Docker container
	docker run -v `pwd`:/src -ti -w "/src" --name hoprd-local-dev nixos/nix nix \
		--extra-experimental-features nix-command \
		--extra-experimental-features flakes \
		develop

.PHONY: run-hopr-admin
run-hopr-admin: version=latest
run-hopr-admin: port=3000
run-hopr-admin: ## launches HOPR Admin in a Docker container, supports port= and version=, use http://host.docker.internal to access the host machine
	docker run -p $(port):80 --name hopr-admin --platform linux/amd64 \
		europe-west3-docker.pkg.dev/hoprassociation/docker-images/hopr-admin:$(version)

.PHONY: exec-script
exec-script: ## execute given script= with the correct PATH set
ifeq ($(script),)
	echo "parameter <script> missing" >&2 && exit 1
endif
	bash "${script}"

.PHONY: help
help:
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' Makefile | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'
