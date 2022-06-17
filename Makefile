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
	npx tsc -p packages/utils/tsconfig.json
	yarn workspace @hoprnet/hopr-ethereum run build:sol:types

.PHONY: build-yarn
build-yarn: ## build yarn packages
build-yarn: build-solidity-types build-cargo
	npx tsc --build tsconfig.build.json

.PHONY: build-yarn-utils
build-yarn-utils: ## build yarn package 'hopr-utils
	npx tsc -p packages/utils/tsconfig.json

.PHONY: build-cargo
build-cargo: ## build cargo packages
build-cargo: build-yarn-utils
	cargo build --release --target wasm32-unknown-unknown
	yarn workspaces foreach -p --exclude hoprnet --exclude hopr-docs run build:wasm

.PHONY: build-yellowpaper
build-yellowpaper: ## build the yellowpaper in docs/yellowpaper
	make -C docs/yellowpaper

.PHONY: test
test: ## run unit tests for all packages, or a single package if package= is set
ifdef package
	yarn workspace @hoprnet/${package} run test
else
	yarn workspaces foreach -pv run test
endif

.PHONY: lint-check
lint-check: ## run linter in check mode
	npx prettier --check .

.PHONY: lint-check
lint-fix: ## run linter in fix mode
	npx prettier --write .

.PHONY: help
help:
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' Makefile | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'
