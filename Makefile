.POSIX:

all: help

.PHONY: deps
deps: ## install dependencies
	yarn
	command -v rustup && rustup update || echo "No rustup installed, ignoring"

.PHONY: build
build: ## build all packages
build: | build-yarn-utils build-solidity-types build-hopr-admin build-cargo build-yarn

.PHONY: build-yarn
build-yarn: ## build yarn packages
build-yarn: build-cargo ## requires WASM boilerplate code
	npx tsc --build tsconfig.build.json

.PHONY: build-hopr-admin
build-hopr-admin: ## build hopr admin React frontend
	yarn workspace @hoprnet/hoprd run buildAdmin

.PHONY: build-solidity-types
build-solidity-types: ## generate Solidity typings
build-solidity-types: build-yarn-utils
	yarn workspace @hoprnet/hopr-ethereum run build:sol:types

.PHONY: build-yarn-utils
build-yarn-utils: ## build yarn package 'hopr-utils' only
	npx tsc -p packages/utils/tsconfig.json

.PHONY: build-cargo
build-cargo: ## build cargo packages
build-cargo: build-yarn-utils ## Use cargo workspaces to resolve Rust package dependencies and precompile all dependencies within a single run
	cargo build --release --target wasm32-unknown-unknown
	yarn workspaces foreach -p --exclude hoprnet --exclude hopr-docs run build:wasm

.PHONY: build-yellowpaper
build-yellowpaper: ## build the yellowpaper in docs/yellowpaper
	make -C docs/yellowpaper

.PHONY: help
help:
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' Makefile | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'
