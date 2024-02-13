## Summary

This release concludes the migration of `hoprd` from being a NodeJS-based
application to a native application written in Rust. All run targets are now
native binaries. The release features improved resource consumption and better
configurability.

We recommend users upgrade their `2.0.0` nodes to benefit from these changes.

## Important: Breaking Changes

While this release is backwards-compatible with `2.0.0` on the
p2p and on-chain layers, it includes some changes in the HTTP API, CLI and
configuration file which manual intervention to ensure a node keeps working
after upgrading to `2.1.0`.

### CLI

The available CLI arguments were cleaned up and reduced to the minimum arguments
while most of the `hoprd` functionality can now be configured through a separate
configuration file configured through `--configurationFilePath`.

Important changes:

- Deprecation of `--healthCheck`, `--healthCheckHost` and `--healthCheckPort`,
    because the underlying functionality has been removed
- Deprecation of `--dryRun`,
    because the underlying functionality has been removed

For reference check the output of `--help`, see https://hoprnet.github.io/hoprnet/#usage

### Configuration

All of `hoprd` configuration can be done through a single YAML configuration
file. The file can be provided at startup through the CLI argument
`--configurationFilePath`.

A reference configuration file showing the full structure can be seen here:

https://github.com/hoprnet/hoprnet/blob/v2.1.0/hoprd/hoprd/example_cfg.yaml

Some configuration values can be overloaded through their respective CLI
arguments, and further through their respective environment variables.

The configuration file itself can be partial, missing values will be merged with
the internal default configuration.

Important changes:

TODO: add specific changes

### HTTP API

The swagger schema for the v3 API has become stricter in terms of input/output
value definitions. This may break clients which were using values outside of the
supported ranges, requiring clients to be made compatible with the changed
schema.

All supported clients (JS, Python) were fixed and support the HTTP API v3 in
`2.1.0`.

## Changes

### 🚀 New Features

- #5644 - Allow message fetching without popping messages.
- #4922 - Add state monitoring mechanism
- #6014 - Divide the RTT time by 2 to get a single direction value
- #6006 - Add example_cfg.yaml with documentation
- #6002 - Fix the docker-compose setup to comply with 2.1
- #5988 - Improve ChannelStatus logic
- #5980 - Fixes to code exeuction and real-world run optimizations
- #5979 - Optimized getLogs queries
- #5967 - Unify object naming in the hoprd rest api
- #5958 - Add staking hub Ethereum address into onboarding log output
- #5957 - Run the API immediately to allow checks before indexing
- #5956 - rpc: Replace custom finality tracking with TimeLag Provider from ethers.rs
- #5948 - Point to Safe transaction service production endpoint
- #5938 - Update the README.md
- #5933 - Peer version check to Promiscuous strategy
- #5923 - Change the default RPC provider to https://gnosis-provider.rpch.tech/
- #5916 - Improve Ticket related structures
- #5897 - Lint the entire codebase with cargo clippy
- #5896 - Remove `surf` client middleware logger
- #5893 - hoprd-cfg: Add the configuration management utility
- #5887 - Refactoring of basic types
- #5879 - Smoke-test configuration "pythonified"
- #5878 - Split core-crypto into 3 crates
- #5871 - Customize Retry policy on JSON-RPC client
- #5857 - Post-migration repo structure cleanup
- #5856 - Optimize sqlite DB parameters
- #5855 - Metrics improvements
- #5849 - tests: Migrate smoke tests into python
- #5802 - Update Dalek dependencies
- #5770 - Ticket price exposed through the API
- #5766 - Peek messages without popping them
- #5749 - Integrate RPC with migrated Indexer
- #5744 - SMA improvements
- #5727 - Fix docker labeling on merge
- #5721 - Deploy nodes in Kubernetes with Hoprd-operator
- #5714 - Add the last_seen_latency information to API endpoint /node/peers output

### 🐛 Bug

- #5994 - `send-message` body slightly different between 2.0.7 and 2.1.0
- #5991 - Listing full topology fails if some nodes aren't reachable
- #5961 - The smoke test is failing randomly, but often
- #5818 - Indexing process after migration to Rust takes 4x longer than before
- #5676 - hoprd FATAL ERROR, exiting with uncaught exception uncaughtException Error [NotSupportedError]
- #6019 - Annotate Bearer token auth in Swagger
- #6016 - Fix duplicate redeem strategy configuration from the CLI
- #6013 - Do not initialize the identity, if it is passed by the config
- #6001 - libp2p: Extend the new configuration to allow higher outbound connections counts
- #5995 - rlp: Strip first 2 bytes from encoded timestamp
- #5993 - Another batch of fixes to Saint Louis
- #5983 - Do not re-announce on-chain if node has previously announced
- #5981 - Remove deprecated docker images from workflows
- #5978 - Fix parsing panic in `from_hex`
- #5953 - Properly override boolean configuration files with CLI arguments
- #5946 - Switch HTTP client from async-h1 to isahc (curl-based)
- #5942 - Fix retry queue counters and make contract call error logs more specific
- #5932 - Update config to fix issues and allow mergability of configuration files
- #5919 - Fix target/debug exposure in PATH and remove the prefix from tooling
- #5905 - Fix stress tests to run with the refactored hoprd-api and fixtures
- #5877 - Improve path selection algorithm
- #5874 - Fix timestamp e2e test using second precision in peer/send
- #5867 - Add missing commit and push script
- #5853 - Increase RPC page size
- #5781 - Remove reviewers that are not collaborators on dappnode PR creation

### ⚡ Other

- #5854 - Restructure to Rust-oriented monorepo setup
- #5502 - Migrate HTTP API to Rust
- #6009 - Improve the API to be usable with the hopr-sdk in JS
- #6007 - Version string fix for 2.1 metrics
- #5972 - make: Add manager-retry-register-nodes
- #5971 - Fix shell execution script for python sdk generation in the Makefile
- #5968 - docker: Update compose setup to use v2.1 and configuration file
- #5966 - Tweaking tests for timestamp
- #5965 - docker: Add hoprd debug image
- #5955 - Fix readme on env variables
- #5954 - Improve smart contract unit test
- #5936 - Inlay the README.md into the Rust documentation
- #5935 - Limit the weak crypto option to test and debug builds
- #5921 - Generate the workspace index.html
- #5899 - Update deprecated dependencies
- #5895 - bindings: Rewrite generation as cargo build script instead of make targets
- #5875 - Cleanup WASM remnants and fix repo structure
- #5873 - Add websocket support for message sending
- #5869 - Proofreading the docs
- #5868 - Pipeline to auto generate python SDK on cutting release event 
- #5863 - Add nix derivation for hoprd
- #5852 - fix Docker build
- #5832 - Timestamp filtering in `peek-all`
- #5814 - Finish the migration of the HOPR node to Rust
- #5813 - Fix the python-sdk and OpenAPI spec generation
- #5806 - Add support for k8s type application checks
- #5796 - Update rust toolchain to 1.74.1
- #5787 - tests: Add integration tests for core-ethereum Indexer
- #5756 - api: Balances are shown as base values without unit
- #5738 - Refactoring of `core-ethereum` package
- #5734 - tests: Migrate fixtures to python
- #5716 - Migrate the indexer functionality
- #5715 - One safe multi nodes support
