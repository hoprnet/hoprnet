What's changed

### 🚀 New Features

* feat(hopr-lib): add more state granularity by @Teebor-Choka in #7782
* chore(deps): update 20260129 by @Teebor-Choka in #7738
* chore: remove unmaintained `bincode` dependency by @NumberFour8 in #7712
* feat: add Safe deployment support to Connector by @NumberFour8 in #7710
* chore(deps): update 20251225 by @Teebor-Choka in #7703
* chore: bump blokli-client version to 0.14.0 by @NumberFour8 in #7686
* chore(deps): update 20251219 by @Teebor-Choka in #7673
* chore: update blokli-client to 0.11.0 by @NumberFour8 in #7671
* feat: use Blokli-returned key binding fee for new key bindings by @NumberFour8 in #7664
* feat: allow acknowledgement batching by @NumberFour8 in #7663
* feat(hopr-lib): novel 0-hop probing mechanism by @Teebor-Choka in #7657
* feat: remove channel ID from Ticket by @NumberFour8 in #7644
* chore(deps): update 20251124 by @Teebor-Choka in #7640
* feat(ci): Update actions/checkout to v6 by @tolbrino in #7632
* feat(ci): Add support for docker-compose healthchecks by @ausias-armesto in #7622
* feat: add v4 payload announcement by @QYuQianchen in #7618
* feat(ci): add support for detecting redundant cargo dependencies by @Teebor-Choka in #7615
* chore(deps): update 20251031 by @Teebor-Choka in #7596
* feat: add blokli client to hopr-lib by @NumberFour8 in #7592
* feat(hoprd): make the allocator selection configurable by feature by @tolbrino in #7591
* feat: introduce `NodeId` and use it in `DestinationRouting` by @NumberFour8 in #7585
* feat: enable acknowledgement batching by @NumberFour8 in #7576
* feat(hopr-lib): add external interfaces for swappable cover traffic engine by @Teebor-Choka in #7565
* chore(docs): remove deprecated merge-back section by @Teebor-Choka in #7540
* feat(chain-config): Add non-version-checked constructor by @tolbrino in #7527
* feat: add chain operations traits into the new hopr-traits crate by @NumberFour8 in #7495
* feat: node discover its NAT status by @jeandemeusy in #7451


### 🐞 Fixes

* fix(ci): update `pull_request_target` triggered issues upon merge by @Teebor-Choka in #7763
* fix(hopr-lib): fix issue related with the PEER_NOT_FOUND ping error by @Teebor-Choka in #7748
* fix: add check on key-binding before proceeding with startup by @NumberFour8 in #7709
* fix: change register_safe logic by @NumberFour8 in #7699
* fix: export BasicPayloadGenerator from by @NumberFour8 in #7667
* fix: improvements to docs and configs by @NumberFour8 in #7661
* fix(hopr-lib): Fix redeemTicket call selector by @tolbrino in #7630
* fix(deps): Downdgrade alloy to 1.0.42 by @tolbrino in #7629
* fix: add support for network health checking into readiness and healthiness by @Teebor-Choka in #7600


### 🧹 Refactor

* refactor: make hopli indenpendent of hopr chain object types by @NumberFour8 in #7590
* refactor: move packet processing traits from the DB to a standalone object by @NumberFour8 in #7575


### 🌟 Other

* Missing metrics by @jeandemeusy in #7720
* Panic after blokli query by @jeandemeusy in #7678
* `NamedTempFile::new()` resolves to a non-exiting path by @jeandemeusy in #7677
* `hoprd` binary argument `--provider` is semantically incorrect and should reflect the `blokli` server instance by @Teebor-Choka in #7659
* Replace the existing 0-hop probing with a more dynamic and stable mechanism by @Teebor-Choka in #7656
* Blokli v1 by @QYuQianchen in #7587
* Tb/202509 channel monitor by @tolbrino in #7524


