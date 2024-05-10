## Summary

This release concludes the migration of `hoprd` from being a NodeJS-based
application to a native application written in Rust. All run targets are now
native binaries. The release features improved resource consumption and better
configurability.

We recommend users upgrade their `2.0.x` nodes to benefit from these changes.
For up-to-date information on how to run the most recent version of `hoprd`
refer to the user documentation at https://docs.hoprnet.org/node/start-here

## Important: Breaking Changes

While this release is backwards-compatible with `2.0.x` on the
p2p and on-chain layers, it includes some changes in the HTTP API, CLI and
configuration file which require manual intervention to ensure a node keeps working
after upgrading to `2.1.0`.

### CLI

The available CLI arguments were cleaned up and reduced to the minimum arguments
while most of the `hoprd` functionality can now be configured through a separate
configuration file set through `--configurationFilePath`.

Important changes:

Deprecation of `--healthCheck`, `--healthCheckHost` and `--healthCheckPort`
  because the underlying functionality has been removed
Deprecation of `--dryRun`
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

### HTTP API

The swagger schema for the v3 API has become stricter in terms of input/output
value definitions. This may break clients which were using values outside the supported ranges, requiring clients to be made compatible with the changed
schema.

All supported clients (JS, Python) were fixed and support the HTTP API v3 in
`2.1.0`.

The schema differences can be inspected here:
https://gist.github.com/tolbrino/31ec835fdb2be5c88c774ea2fdf2f133

## Changes

### 🚀 New Features

- #4922 - Add state monitoring mechanism
- #5644 - Allow message fetching without popping messages.
- #5714 - Add the last_seen_latency information to API endpoint /node/peers output
- #5721 - Deploy nodes in Kubernetes with Hoprd-operator
- #5727 - Fix docker labeling on merge
- #5744 - SMA improvements
- #5749 - Integrate RPC with migrated Indexer
- #5766 - Peek messages without popping them
- #5770 - Ticket price exposed through the API
- #5802 - Update Dalek dependencies
- #5849 - tests: Migrate smoke tests into python
- #5855 - Metrics improvements
- #5856 - Optimize sqlite DB parameters
- #5857 - Post-migration repo structure cleanup
- #5871 - Customize Retry policy on JSON-RPC client
- #5878 - Split core-crypto into 3 crates
- #5879 - Smoke-test configuration "pythonified"
- #5887 - Refactoring of basic types
- #5893 - hoprd-cfg: Add the configuration management utility
- #5896 - Remove `surf` client middleware logger
- #5897 - Lint the entire codebase with cargo clippy
- #5916 - Improve Ticket related structures
- #5923 - Change the default RPC provider to https://gnosis-provider.rpch.tech/
- #5933 - Peer version check to Promiscuous strategy
- #5938 - Update the README.md
- #5948 - Point to Safe transaction service production endpoint
- #5956 - rpc: Replace custom finality tracking with TimeLag Provider from ethers.rs
- #5957 - Run the API immediately to allow checks before indexing
- #5958 - Add staking hub Ethereum address into onboarding log output
- #5967 - Unify object naming in the hoprd rest api
- #5979 - Optimized getLogs queries
- #5980 - Fixes to code exeuction and real-world run optimizations
- #5988 - Improve ChannelStatus logic
- #6002 - Fix the docker-compose setup to comply with 2.1
- #6006 - Add example_cfg.yaml with documentation
- #6014 - Divide the RTT time by 2 to get a single direction value
- #6033 - Use smart default wherever reasonable
- #6042 - docker: Add bash to docker images
- #6047 - Documentation updates and minor code cleanup
- #6052 - Change batch size default and small config refactoring
- #6058 - Soft-restart indexer loop on error, expose `max_block_range` on the CLI
- #6059 - Decrease the scope of some RwLocks
- #6065 - Better handling of missing block errors
- #6066 - Improve visbility into code hot paths using metrics and logs
- #6067 - Separate `get_peer_with_quality` expression
- #6071 - Use Sqlite backend for Network peers storage to prevent high-level locking
- #6074 - Add more logs to debug double transaction transmission in mempool
- #6079 - Use exactly the amount of hops specified by the caller
- #6084 - Auto-redeem single tickets of value on closure initiation
- #6090 - Logging and db scoping fixes
- #6092 - Introduce ORM and database migrations
- #6096 - Add DB API traits and implementations
- #6113 - Fix return code for set alias API call
- #6129 - Improve chain event processing from logs
- #6166 - Add log information when node gets registered
- #6181 - Add PeerID <-> Public key converter to hopli
- #6197 - Add log transaction checksum into Indexer
- #6198 - Add metrics for logs processed by the Indexer
- #6203 - Squash additional migrations to prior ones
- #6220 - Increase size of the DB connection pool
- #6226 - Do not use ethers-rs pagination of `eth_getLogs`

### 🐛 Bug

- #5676 - hoprd FATAL ERROR, exiting with uncaught exception uncaughtException Error [NotSupportedError]
- #5781 - Remove reviewers that are not collaborators on dappnode PR creation
- #5818 - Indexing process after migration to Rust takes 4x longer than before
- #5853 - Increase RPC page size
- #5867 - Add missing commit and push script
- #5874 - Fix timestamp e2e test using second precision in peer/send
- #5877 - Improve path selection algorithm
- #5905 - Fix stress tests to run with the refactored hoprd-api and fixtures
- #5919 - Fix target/debug exposure in PATH and remove the prefix from tooling
- #5932 - Update config to fix issues and allow mergability of configuration files
- #5942 - Fix retry queue counters and make contract call error logs more specific
- #5946 - Switch HTTP client from async-h1 to isahc (curl-based)
- #5953 - Properly override boolean configuration files with CLI arguments
- #5961 - The smoke test is failing randomly, but often
- #5978 - Fix parsing panic in `from_hex`
- #5981 - Remove deprecated docker images from workflows
- #5983 - Do not re-announce on-chain if node has previously announced
- #5991 - Listing full topology fails if some nodes aren't reachable
- #5993 - Another batch of fixes to Saint Louis
- #5994 - `send-message` body slightly different between 2.0.7 and 2.1.0
- #5995 - rlp: Strip first 2 bytes from encoded timestamp
- #6001 - libp2p: Extend the new configuration to allow higher outbound connections counts
- #6013 - Do not initialize the identity, if it is passed by the config
- #6016 - Fix duplicate redeem strategy configuration from the CLI
- #6019 - Annotate Bearer token auth in Swagger
- #6037 - Fixes for issues found in the hopr-sdk testing
- #6057 - Terminate Indexer processing loop on hard failure
- #6088 - Collection of fixes to scoping and locks for the DB
- #6089 - Fix multistrategy not defaulting values for hoprd config properly
- #6100 - Fix commit hash labeling docker images
- #6114 - fix: missing serialization info in `ClosureFinalizerStrategyConfig`
- #6150 - Reorder multistrategy to first start to happen only once the node is fully running
- #6162 - Deprecate CLI configuration of strategies
- #6171 - Fix incorrect VRF signer in Ticket Aggregation
- #6174 - Workaround version matching issue in Promiscuous strategy
- #6188 - Fix updating node info table
- #6202 - Fix comparison of invalid balance types
- #6206 - Do not allow duplicate peer ID aliasing
- #6211 - Fix off-by-1 in the Indexer
- #6212 - Node cannot not start after synced due to a failure getting connection pool
- #6215 - Make set_alias e2e test stable again
- #6218 - Fix Indexer checksum computation
- #6227 - chain: do not fetch logs if no topics are set
- #6230 - Fail hard on failure to load the block number from RPC provider
- #6235 - Path finding should only consider nodes that proven to be reliable
- #6237 - The `/node/peers` API endpoint sometimes returned empty multiaddreses as `announced`

### ⚡ Other

- #5502 - Migrate HTTP API to Rust
- #5715 - One safe multi nodes support
- #5716 - Migrate the indexer functionality
- #5734 - tests: Migrate fixtures to python
- #5738 - Refactoring of `core-ethereum` package
- #5756 - api: Balances are shown as base values without unit
- #5787 - tests: Add integration tests for core-ethereum Indexer
- #5796 - Update rust toolchain to 1.74.1
- #5806 - Add support for k8s type application checks
- #5813 - Fix the python-sdk and OpenAPI spec generation
- #5814 - Finish the migration of the HOPR node to Rust
- #5832 - Timestamp filtering in `peek-all`
- #5852 - fix Docker build
- #5854 - Restructure to Rust-oriented monorepo setup
- #5863 - Add nix derivation for hoprd
- #5868 - Pipeline to auto generate python SDK on cutting release event
- #5869 - Proofreading the docs
- #5873 - Add websocket support for message sending
- #5875 - Cleanup WASM remnants and fix repo structure
- #5895 - bindings: Rewrite generation as cargo build script instead of make targets
- #5899 - Update deprecated dependencies
- #5902 - Rust docs improvements
- #5921 - Generate the workspace index.html
- #5935 - Limit the weak crypto option to test and debug builds
- #5936 - Inlay the README.md into the Rust documentation
- #5954 - Improve smart contract unit test
- #5955 - Fix readme on env variables
- #5965 - docker: Add hoprd debug image
- #5966 - Tweaking tests for timestamp
- #5968 - docker: Update compose setup to use v2.1 and configuration file
- #5971 - Fix shell execution script for python sdk generation in the Makefile
- #5972 - make: Add manager-retry-register-nodes
- #6007 - Version string fix for 2.1 metrics
- #6009 - Improve the API to be usable with the hopr-sdk in JS
- #6027 - Allow websockets to parse the auth from a websocket protocol
- #6029 - Update the nix environment and the package lock
- #6046 - Improve documentation across different crates
- #6048 - Add documentation in hopli
- #6055 - Relax the limitation to strictly check existence of the hoprd identity from the configuration
- #6056 - Enforce strict configuration file parsing to deny unknown keys
- #6062 - Replace the log infrastructure with tracing
- #6063 - Fix the hoprd-api-schema generation by directly executing the binary
- #6069 - Remove `Address::random()` function from production code
- #6107 - `chain-actions` and `core-strategy` migration to the new DB
- #6119 - Add support for PR labeling
- #6124 - Saint Louis fixes from practical testing session
- #6131 - Add node configuration endpoint
- #6135 - Improvements to ticket metrics and deprecation of ticket displaying API endpoints
- #6145 - Fix minor logging issues to make the info output more readable
- #6151 - Reduce hopli default provider tx poll interval from 7s to 10 ms
- #6165 - Fix error codes on certain API endpoints
- #6167 - hopli: Configure contracts root in docker image by default
- #6189 - Fix the display of sync metric
- #6190 - Fix issues with alias urldecoding
- #6192 - Add support for yamux as default mux to libp2p swarm
- #6193 - Allow RPC provider to feed past blocks
- #6194 - Fix hoprd persistence directory paths
- #6201 - Add commit hash to log output in hoprd
- #6210 - Improve descriptions in the 2.1 API docs
- #6213 - Improves the aliases e2e test
- #6223 - Log RPC request/response JSON
- #6233 - Fix subtraction operations on Durations to be saturating
- #6234 - Update checksum printout
- #6238 - Increase the connection pool size of Peers DB
- #6242 - Add more connection pool parameters
- #6244 - Update metrics
