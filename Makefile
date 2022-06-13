.POSIX:

all: help

.PHONY: deps
deps: ## install dependencies
	yarn

.PHONY: build
build: ## build all packages
build: | build-yarn-utils build-cargo build-yarn

.PHONY: build-yarn
build-yarn: ## build yarn packages
	yarn workspace @hoprnet/hopr-ethereum run build:sol:types
	npx tsc --build tsconfig.build.json
	yarn workspace @hoprnet/hoprd run buildAdmin

.PHONY: build-yarn-utils
build-yarn-utils: ## build yarn package 'hopr-utils' only
	npx tsc -p packages/utils/tsconfig.json

.PHONY: build-cargo
build-cargo: ## build cargo packages
build-cargo: build-yarn-utils
	rustup target add wasm32-unknown-unknown
	cargo build --release --target wasm32-unknown-unknown
	yarn workspaces foreach -p --exclude hoprnet --exclude hopr-docs run build:wasm

.PHONY: help
help:
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' Makefile | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'
