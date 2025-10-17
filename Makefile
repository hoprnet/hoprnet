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
test:
	$(cargo) test --features runtime-tokio

.PHONY: stress-test-local-swarm
stress-test-local-swarm: ## run stress tests on a local node swarm
	uv run -m pytest tests/test_stress.py \
		--stress-request-count=3000 \
		--stress-sources='[{"url": "localhost:3011", "token": "e2e-API-token^^"}]' \
		--stress-target='{"url": "localhost:3031", "token": "e2e-API-token^^"}'



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
