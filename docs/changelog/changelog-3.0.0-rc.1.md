What's changed

### 🚀 New Features

* feat(docker): Add info on building docker image by @tolbrino in #7285
* feat(ci): add pre-commit checks by @Teebor-Choka in #7280
* feat(ci): Add package support to archlinux variant by @ausias-armesto in #7277
* feat(hopr-lib): Add a reserved flag to the KeepAlive Session message by @NumberFour8 in #7263
* feat(hoprd): introduce Session pooling at the API by @NumberFour8 in #7258
* feat(ci): Create OS packages by @ausias-armesto in #7251
* feat(runtime): make spawned tasks abortable through futures mechanism by @Teebor-Choka in #7247
* chore(deps): Update dependencies (20250610) by @Teebor-Choka in #7229
* chore(nix): update flake.lock by @app/github-actions in #7221
* chore(nix): update flake.lock by @app/github-actions in #7206
* chore(nix): update flake.lock by @app/github-actions in #7181
* chore(deps): update rust crate oas3 to 0.16.0 by @app/renovate in #7057


### 🐞 Fixes

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

* refactor(hopr-lib): Reduce required on-chain logs in the indexer (revisited) by @tolbrino in #7213


### 🌟 Other

* Update documentation on Nodemanagement module by @QYuQianchen in #7028
* Post-Providence smart contract features (tracker for v3.0) by @QYuQianchen in #7027
* Protocol Feature: Return Path by @tolbrino in #4496
* feat: add configurable limits for transport packet handling and stream open timeout by @NumberFour8 in #7320
* ci: Add pluto build flag by @ausias-armesto in #7315
* feat: Add binary signatures and multiple os package improvements by @ausias-armesto in #7297
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
* deps: Update newest versions (20250414) by @Teebor-Choka in #7024
* ci: Update renovate configuration by @Teebor-Choka in #7021
* Publishing benchmarks by @ausias-armesto in #7019
* audit: Fix issues by bumping relevant dependencies by @Teebor-Choka in #7016
* ci: Remove the remnant after physical vendoring inside the repo by @Teebor-Choka in #7008
* ci: Update the `cache-deps` naming convention by @Teebor-Choka in #7006
* ci: Fix cache dependency workflow by @Teebor-Choka in #7005
* ci: Don't persist credentials by @tolbrino in #7001
* ci: Fix flake lock action by @tolbrino in #7000
* Add Bencher pipeline by @ausias-armesto in #6998
* nix: Fix shell dependencies on macos by @tolbrino in #6995
* Backport python tests and expose cluster to local network by @esterlus in #6979
* Merge back changes from v2.2.3 (on top of libp2p_stream) by @tolbrino in #6977
* Add Auditing code by @ausias-armesto in #6972
* Changing folder for docker compose by @ausias-armesto in #6971
* Merge back changes from v2.2.3 by @tolbrino in #6970
* [StepSecurity] ci: Harden GitHub Actions by @tolbrino in #6950
* Introduce Return Path to the protocol by @NumberFour8 in #6932
* Split API endpoints by content type by @jeandemeusy in #6931
* Add support for libp2p_stream into the swarm object by @Teebor-Choka in #6928


