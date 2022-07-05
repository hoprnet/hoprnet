.POSIX:

all: help

.PHONY: deps
deps: ## install dependencies
	yarn
	command -v rustup && rustup update || echo "No rustup installed, ignoring"

.PHONY: build
build: ## build all packages
build: build-hopr-admin build-yarn

.PHONY: build-hopr-admin
build-hopr-admin: ## build hopr admin React frontend
	yarn workspace @hoprnet/hoprd run buildAdmin

.PHONY: build-solidity-types
build-solidity-types: ## generate Solidity typings
build-solidity-types: build-cargo
	npx tsc -p packages/utils/tsconfig.json
	yarn workspace @hoprnet/hopr-ethereum run build:sol:types

.PHONY: build-yarn
build-yarn: ## build yarn packages
build-yarn: build-solidity-types build-cargo
	npx tsc --build tsconfig.build.json

.PHONY: build-cargo
build-cargo: ## build cargo packages
	cargo build --release --target wasm32-unknown-unknown
	yarn workspaces foreach --exclude hoprnet --exclude hopr-docs run build:wasm

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

.PHONY: test
test: ## run unit tests for all packages, or a single package if package= is set
ifeq ($(package),)
	yarn workspaces foreach -pv run test
else
	yarn workspace @hoprnet/${package} run test
endif

.PHONY: lint-check
lint-check: ## run linter in check mode
	npx prettier --check .

.PHONY: lint-check
lint-fix: ## run linter in fix mode
	npx prettier --write .

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

.PHONY: stake-funds
stake-funds: ## stake funds (idempotent operation)
ifeq ($(privkey),)
	echo "parameter <privkey> missing" >&2 && exit 1
endif
ifeq ($(environment),)
	echo "parameter <environment> missing" >&2 && exit 1
endif
	@TS_NODE_PROJECT=./tsconfig.hardhat.json \
	HOPR_ENVIRONMENT_ID="$(environment)" \
		yarn workspace @hoprnet/hopr-ethereum run hardhat stake \
		--network goerli \
		--type xhopr \
		--amount 1000000000000000000000 \
		--privatekey "$(privkey)"

.PHONY: stake-devnft
stake-devnft: ## stake Dev NFTs (idempotent operation)
ifeq ($(privkey),)
	echo "parameter <privkey> missing" >&2 && exit 1
endif
ifeq ($(environment),)
	echo "parameter <environment> missing" >&2 && exit 1
endif
	@TS_NODE_PROJECT=./tsconfig.hardhat.json \
	HOPR_ENVIRONMENT_ID="$(environment)" \
		yarn workspace @hoprnet/hopr-ethereum run hardhat stake \
		--network goerli \
		--type devnft \
		--privatekey "$(privkey)"

.PHONY: help
help:
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' Makefile | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'
