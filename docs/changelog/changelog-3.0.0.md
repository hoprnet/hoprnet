What's changed

### 🚀 New Features

* feat(hoprd): Switch to mimalloc as default allocator by @tolbrino in #7458
* feat: allow auto-redeeming strategy to work in on-tick mode by @NumberFour8 in #7448
* chore(deps): update github workflow actions by @Teebor-Choka in #7447
* feat: allow reducing organic SURB production by @NumberFour8 in #7429
* feat: extend the size of the KeepAlive's additional data by @NumberFour8 in #7425
* feat: updated Lioness wide-block cipher implementation by @NumberFour8 in #7419
* feat: parametrable localcluster size by @jeandemeusy in #7412
* chore(hopr-lib): lower some logs to more realistic values by @Teebor-Choka in #7398
* chore(deps): bi-monthly update 20250813 by @Teebor-Choka in #7395
* feat: handle event for an unknown channel by @jeandemeusy in #7380
* chore(ci): Remove unused scripts by @tolbrino in #7378
* chore(hopr-lib): improve logging for peer health change and connectivity by @Teebor-Choka in #7367
* feat(nix): Add capture-enabled binaries by @tolbrino in #7365
* chore(docs): add description for opentelemetry streaming by @Teebor-Choka in #7362
* feat: add number of surbs into the dissector by @NumberFour8 in #7357
* chore(nix): update flake.lock by @app/github-actions in #7351
* feat(chain): Add logs snapshot fetching by @tolbrino in #7341
* feat: Add signature and hash to linux packages by @ausias-armesto in #7333
* chore(deps): update dependencies 20250715 by @Teebor-Choka in #7331
* feat: allow pcap upload in workflows by @NumberFour8 in #7327
* feat: add configurable limits for transport packet handling and stream open timeout by @NumberFour8 in #7320
* feat: Add binary signatures and multiple os package improvements by @ausias-armesto in #7297
* feat(hopr-lib): add corrupted state to channels by @jeandemeusy in #7296
* feat(docker): Add info on building docker image by @tolbrino in #7285
* feat(ci): add pre-commit checks by @Teebor-Choka in #7280
* feat(ci): Add package support to archlinux variant by @ausias-armesto in #7277
* chore(deps): Bi-weekly update 20250701 by @Teebor-Choka in #7273
* feat(hopr-lib): Add a reserved flag to the KeepAlive Session message by @NumberFour8 in #7263
* feat(hoprd): introduce Session pooling at the API by @NumberFour8 in #7258
* feat(ci): Create OS packages by @ausias-armesto in #7251
* feat(runtime): make spawned tasks abortable through futures mechanism by @Teebor-Choka in #7247
* feat(ci): Mandate semantic pull requests and commits by @Teebor-Choka in #7231
* chore(deps): Update dependencies (20250610) by @Teebor-Choka in #7229
* chore(nix): update flake.lock by @app/github-actions in #7221
* chore(nix): update flake.lock by @app/github-actions in #7206
* feat(ci): Allow forked PRs to run workflows by @ausias-armesto in #7191
* chore(nix): update flake.lock by @app/github-actions in #7181
* chore(deps): update rust crate oas3 to 0.16.0 by @app/renovate in #7057


### 🐞 Fixes

* fix: simplify EC point validation due co-factor pre-multiplication of the scalars by @NumberFour8 in #7482
* fix: Fix dappnode build package by @ausias-armesto in #7473
* fix: remove TBF saving, use `parking_lot::Mutex` in busy-paths by @NumberFour8 in #7472
* fix(hopr-lib): properly export `HoprKeys` related objects by @Teebor-Choka in #7471
* fix: connection storms and network transport hickups by @Teebor-Choka in #7467
* fix: add missing columns to the NetworkPeer table by @NumberFour8 in #7462
* fix: optimize cryptographic packet processing by @NumberFour8 in #7460
* fix(tests): unique peers in paths by @jeandemeusy in #7459
* fix: update session protocol limits according to rfc-0007 by @NumberFour8 in #7454
* fix: end correctly the socket and processes once the entire UDP connection is dropped by @NumberFour8 in #7453
* fix: cache PeerId -> OffchainPublicKey conversion inside the packet pipeline by @NumberFour8 in #7440
* fix: Adapt profile image by @ausias-armesto in #7434
* fix: use faster ECDSA/ECDH library for secp256k1 by @NumberFour8 in #7432
* fix: v1 tags should be limited to 61-bits by @NumberFour8 in #7413
* fix: serialization of strategy parameter by @jeandemeusy in #7407
* fix: reaching `session.maximum_session` on exit breaks session opening by @jeandemeusy in #7404
* fix(ci): downgrade inconsistently named checkout updates by @Teebor-Choka in #7393
* fix(ci): remove runner incompatible changes by @Teebor-Choka in #7391
* fix: handle update of corrupted channel by @jeandemeusy in #7376
* fix(hopr-lib): Set user-agent in snapshot downloader by @tolbrino in #7374
* fix(hopr-lib): indexer database has managed access to separate reading and writing pools by @Teebor-Choka in #7373
* fix(chain): Raise error on reverted transactions by @tolbrino in #7366
* fix: allow multi-layered reply openers by @NumberFour8 in #7363
* fix: remove ticket manager db connection per-packet by @NumberFour8 in #7361
* fix(hopr-lib): decrease the amount of connections to 1 on the index db by @Teebor-Choka in #7346
* fix: Fix testing process for archlinux package by @ausias-armesto in #7345
* fix: increase reply opener cache timeout and enforce the maximum session limit by @NumberFour8 in #7336
* fix: various minor fixes before 3.0 release by @NumberFour8 in #7323
* fix(ci): Do not use mold on macos by @tolbrino in #7321
* fix: Using wrong branch when creating release  by @ausias-armesto in #7317
* fix(nix): hopli aarch64 build working and add man-page generation by @tolbrino in #7310
* fix(ci): Port deploy_nodes changes by @tolbrino in #7305
* fix(ci): comply with pre-commit by @Teebor-Choka in #7281
* fix(ci): Fixing merge pipeline by @ausias-armesto in #7276
* fix(hopr-lib): Include Network Registry status in peer discovery announce by @QYuQianchen in #7264
* fix(hopr-lib): Connectability issues in the local cluster and production by @Teebor-Choka in #7255
* fix(indexer): key-binding errors by @NumberFour8 in #7254
* fix(tests): add extra funding and increase test message count by @Teebor-Choka in #7246
* fix(ci): Remove the python updates as they break compatibility of pluto by @Teebor-Choka in #7242
* fix: Enforce correct endianness in Segment retransmission request of the Session protocol by @NumberFour8 in #7235


### 🧹 Refactor

* refactor: improve hopr-crypto-packet benches by @NumberFour8 in #7457
* refactor: move ticket ack and ack sending operations outside the main pipeline by @NumberFour8 in #7430
* refactor: optimize acknowlegement processing and point decompression by @NumberFour8 in #7428
* refactor: move Application protocol into its own crate, fix tests by @NumberFour8 in #7401
* refactor: various optimizations to sessions & transport by @NumberFour8 in #7368
* refactor: Canonize openapi.json spec output by @jeandemeusy in #7308
* refactor(hopr-lib): Reduce required on-chain logs in the indexer (revisited) by @tolbrino in #7213


### 🌟 Other

* Session cleanup isn't done properly by @jeandemeusy in #7387
* Update documentation on Nodemanagement module by @QYuQianchen in #7028
* Post-Providence smart contract features (tracker for v3.0) by @QYuQianchen in #7027
* Protocol Feature: Return Path by @tolbrino in #4496
* ci: Add pluto build flag by @ausias-armesto in #7315
* Remove balance update on API query by @NumberFour8 in #7252
* tests: Fix randomly failing win_prob smoke test by @Teebor-Choka in #7230
* Fixed an incorrect calculation of Tag size by @NumberFour8 in #7226
* tests: Simplify smoke test setup by @Teebor-Choka in #7225
* Backport backwards incompatible changes from 3.1 by @NumberFour8 in #7222
* Fix merge workflow by casting variables into numbers by @ausias-armesto in #7218
* api: Fix missing endpoint in OpenAPI by @Teebor-Choka in #7214
* Fix deploy config file by @jeandemeusy in #7212
* v3: Improvements from testing sessions by @Teebor-Choka in #7209
* api: Fix CORS layer in checks by @Teebor-Choka in #7208
* Update renovate.json to group updates by @Teebor-Choka in #7205
* Lower the TX transmission timeout by @NumberFour8 in #7204
* Improve the tag organization for v3 packet payload format by @Teebor-Choka in #7203
* Update all API routes to `v4` by @NumberFour8 in #7200
* deps: Update cargo dependencies by @Teebor-Choka in #7195
* deps: Update older dependencies by @Teebor-Choka in #7188
* metrics: Add hopr_last_time and rename hopr_up by @tolbrino in #7183
* ci: Add reviewers to new PRs by @tolbrino in #7182
* Fix clippy warnings by @tolbrino in #7180
* tests: Create assertion out of flaky test by @tolbrino in #7178
* tests: Fix chain tests in nightly by @tolbrino in #7176
* Introduce new `Balance` type by @NumberFour8 in #7174
* Migrate alloy to v1 by @QYuQianchen in #7171
* session: Add more tracing logs by @tolbrino in #7168
* ci: Add pre build step by @tolbrino in #7167
* Full reformat of Rust code by @tolbrino in #7162
* nix: Full reformat according to RFC 166 by @tolbrino in #7161
* ci: Fix log upload by @tolbrino in #7160
* Add musl-based static binary builds for supported platforms by @tolbrino in #7158
* deps: Remove surf and surf-governor by @Teebor-Choka in #7157
* toolchain: Update Rust to 1.87 by @Teebor-Choka in #7154
* docker: Fix missing SSL library in the image  by @tolbrino in #7153
* Add binding check ci by @QYuQianchen in #7149
* heartbeat: Replace the libp2p protocol based heartbeat with in-HOPR protocol based one by @Teebor-Choka in #7142
* protocol: Bump heartbeat protocol version to disallow connections to previous versions by @Teebor-Choka in #7138
* Close channels in multicall by @jeandemeusy in #7137
* Fix smoke test execution by @tolbrino in #7136
* tests: Allow smoke tests to run in parallel by @tolbrino in #7131
* audit: Fix deps by @Teebor-Choka in #7130
* Remove Message Inbox and refactor E2E tests using Sessions by @NumberFour8 in #7128
* deps: Update and clean by @Teebor-Choka in #7122
* Avoid adding release-latest tag on PR not target to them by @ausias-armesto in #7111
* nix: Add missing lld to shell by @tolbrino in #7110
* deps: Update cargo dependencies by @Teebor-Choka in #7109
* Remove the 'async-std' due to deprecation by @Teebor-Choka in #7094
* Fix pluto build by @jeandemeusy in #7092
* Introduce Return path and SURB balancing in Sessions by @NumberFour8 in #7088
* Migrate `ethers-rs` to `alloy-rs` by @QYuQianchen in #7079
* Improve smart contract documentation by @QYuQianchen in #7078
* Unify the `msg` and `ack` protocol inside a single payload structure by @Teebor-Choka in #7073
* Return only objects from API by @jeandemeusy in #7072
* renovate: Update the ignore list by @Teebor-Choka in #7071
* ci: Cleanup the nix shells and speed up CI builds by @tolbrino in #7070


