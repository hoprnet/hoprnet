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

.PHONY: build-yellowpaper
build-yellowpaper: ## build the yellowpaper in docs/yellowpaper
	$(MAKE) -C docs/yellowpaper

.PHONY: install
install:
	$(cargo) install --path hoprd/hoprd

.PHONY: stress-test-local-swarm
stress-test-local-swarm: ## run stress tests on a local node swarm
	uv run -m pytest tests/test_stress.py \
		--stress-request-count=3000 \
		--stress-sources='[{"url": "localhost:3011", "token": "e2e-API-token^^"}]' \
		--stress-target='{"url": "localhost:3031", "token": "e2e-API-token^^"}'

.PHONY: localcluster
localcluster: args=
localcluster: ## spin up the localcluster using the default configuration file
	@uv run -m sdk.python.localcluster --config ./sdk/python/localcluster.params.yml --fully_connected $(args)

.PHONY: localcluster-exposed
localcluster-exposed: ## spin up the localcluster using the default configuration file, exposing all nodes in the local network
	@make localcluster args="--exposed"

.PHONY: exec-script
exec-script: ## execute given script= with the correct PATH set
ifeq ($(script),)
	echo "parameter <script> missing" >&2 && exit 1
endif
	bash "${script}"

.PHONY: help
help:
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' Makefile | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'
