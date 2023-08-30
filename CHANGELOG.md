# Changelog

<a name="2.00"></a>

## [2.00](https://github.com/hoprnet/hoprnet/compare/release/bratislava...hoprnet:master)

- Introduce the possibility of using YAML configuration file and revamp the configuration infrastructure to make the YAML a default base for all configuration types (env vars, cmd line args) ([#4796](https://github.com/hoprnet/hoprnet/pull/4796))
- Extend `hopli` so that it can derive peer ID from identity files. Add `initiate-node` to perform all the necessary on-chain operations before launching a HOPR node. Accept identity from path. ([#4894](https://github.com/hoprnet/hoprnet/pull/4894))
- Introduce new types & optimize cryptographic code ([#4974](https://github.com/hoprnet/hoprnet/pull/4974))
- Create DB functionality shim in Rust ([#4885](https://github.com/hoprnet/hoprnet/pull/4885))
- Migrate interface for using the DB in core packet processing ([#5025](https://github.com/hoprnet/hoprnet/pull/5025))
- Migrate all core types to Rust and remove all TypeScript types ([#5039](https://github.com/hoprnet/hoprnet/pull/5039))
- Migrate packet construction code and all related cryptography to Rust and remove the Typescript implementation ([#4834](https://github.com/hoprnet/hoprnet/pull/4834))
- Align terminologies used in the protocol ("network", "chain" and "environment") ([#4938](https://github.com/hoprnet/hoprnet/pull/4938))
- All cryptography related code has been migrated to Rust with TS implementations removed ([#5063](https://github.com/hoprnet/hoprnet/pull/5063))
- Migrate interfaces for using the DB in core-ethereum ([#5072](https://github.com/hoprnet/hoprnet/pull/5072))
- Upgrade OpenZeppelin dependency to 4.8.3 and Solidity to 0.8.19 ([#5094](https://github.com/hoprnet/hoprnet/pull/5094))
- Add support for logs in wasm environment Rust tests ([#5108](https://github.com/hoprnet/hoprnet/pull/5108))
- Migrate Packet and Acknowledgement interactions to Rust ([#5074](https://github.com/hoprnet/hoprnet/pull/5074))
- Change keystore to hold `chain_key` and `packet_key` ([#5175](https://github.com/hoprnet/hoprnet/pull/5175))
- Switch to Curve25519 to generate Sphinx keys, use Ed25519 PeerIDs for transport ([#5069](https://github.com/hoprnet/hoprnet/pull/5069))
- Remove Avado build support ([#5181](https://github.com/hoprnet/hoprnet/pull/5181))
- Upgrade to Node v18 as default runtime ([#5184](https://github.com/hoprnet/hoprnet/pull/5184))
- Use transitive Rust build features and fix various build issues ([#5187](https://github.com/hoprnet/hoprnet/pull/5187))
- Change `core-ethereum` to operate on Ethereum `Address`es rather than `PublicKey`s ([#5189](https://github.com/hoprnet/hoprnet/pull/5189))
- Change the ticket auto redeem and check unrealized balance cli arguments ([#5235](https://github.com/hoprnet/hoprnet/pull/5235))
- Fixed DB deadlocks and changed the DB backend & fixed channel and ticket issues in the DB ([#5229](https://github.com/hoprnet/hoprnet/pull/5229))
- Replace the js-libp2p connection mechanism with the rust-libp2p ([#5153](https://github.com/hoprnet/hoprnet/pull/5153))
- Add `application_tag` into the payload of HOPR protocol, prepare Message Inbox backend ([#5260](https://github.com/hoprnet/hoprnet/pull/5260))
- Rest API: Replace v2 with v3 which adds message inbox support and compatibility with new smart contracts ([#5297](https://github.com/hoprnet/hoprnet/pull/5297))
- Tickets 2.0: tickets use a VRF (verified random function) and no longer depend on external randomness ("iterated hash") ([#5338](https://github.com/hoprnet/hoprnet/pull/5338))
- Tickets 2.0: payment channel funding happens immediately as it no longer requires any on-chain commitments ([#5338](https://github.com/hoprnet/hoprnet/pull/5338))
- Prevent relayers from extracting multiple base amounts ([#5385](https://github.com/hoprnet/hoprnet/pull/5385))
- Added dynamic price per packet via the TicketPriceOracle smart contract ([#5372](https://github.com/hoprnet/hoprnet/pull/5372))

<a name="1.93"></a>

## [1.93](https://github.com/hoprnet/hoprnet/compare/release/riga...hoprnet:release/bratislava)

- Migrate the `Heartbeat`, `Ping` and `Network` functionality into Rust ([#4568](https://github.com/hoprnet/hoprnet/pull/4658))
- Rebrand `foundry-tool` to `hopli` and allow it to interact with identities and network-registry ([#4652](https://github.com/hoprnet/hoprnet/pull/4652))
- Migrate mixer code to Rust ([#4567](https://github.com/hoprnet/hoprnet/pull/4567))
- Fix initial network peer registration from the Peer Store ([#4741](https://github.com/hoprnet/hoprnet/pull/4741))
- `hopli` accepts floating number for the amount of tokens to be transferred/minted. Loosen requirement on the identity file. ([#4723](https://github.com/hoprnet/hoprnet/pull/4723))
- Use simple moving average for smoothing network peer numbers presented to the Promiscuous strategy, add more logging ([#4763](https://github.com/hoprnet/hoprnet/pull/4763))
- Added Rust implementations of all the cryptographic functionality used in the HOPR protocol ([#3842](https://github.com/hoprnet/hoprnet/pull/3842))
- Add reported HOPR protocol version information to `ping` and `peers` endpoints ([#4777](https://github.com/hoprnet/hoprnet/pull/4777))
- Extend the Grafana dashboards with the new mixer metrics (current mixer queue size & average mixer delay) ([#4768](https://github.com/hoprnet/hoprnet/pull/4768))
- Expose full payment channel graph in `/channels` API ([#4756](https://github.com/hoprnet/hoprnet/pull/4756))
- Fixed issue with too many channels being opened by the Promiscuous strategy, added option to enforce the maximum number of opened channels ([#4827](https://github.com/hoprnet/hoprnet/pull/4827))
- Added mitigation for connection leakage to prevent DHT and p2p connection failures ([#4870](https://github.com/hoprnet/hoprnet/pull/4870))
- Fixed incorrect recording of some Ping metrics ([#4867](https://github.com/hoprnet/hoprnet/pull/4867))
- Enforced 1 GB memory usage limit of HOPRd on Avado ([#4898](https://github.com/hoprnet/hoprnet/pull/4898))
- Added RelayState cleanup which prevents excessive connection leakage on public relay nodes ([#4912](https://github.com/hoprnet/hoprnet/pull/4912))
- Reduced maximum number of relay connections to 2000 ([#4912](https://github.com/hoprnet/hoprnet/pull/4912))
- Add pruning in the relay connections keep-alive mechanism ([#4916](https://github.com/hoprnet/hoprnet/pull/4916))
- Enforce closing and clean-up of libp2p2 connections ([#4957](https://github.com/hoprnet/hoprnet/pull/4957))
- Wipe libp2p's AddressManager cache when publishing new addresses to the DHT ([#4958](https://github.com/hoprnet/hoprnet/pull/4958))
- Add metrics & logs relevant for RPCh debugging ([#4995](https://github.com/hoprnet/hoprnet/pull/4995))
- Enhance debug logs in dialing logic to enhance debugging ([#5004](https://github.com/hoprnet/hoprnet/pull/5004))
- Fix address coercion issue in libp2p address handling ([#5020](https://github.com/hoprnet/hoprnet/pull/5020))
- Ensure public relay nodes don't create relay connections ([#5023](https://github.com/hoprnet/hoprnet/pull/5023))
- Add heartbeat and cleanup to API websocket connections ([#5023](https://github.com/hoprnet/hoprnet/pull/5023))
- Add send message over websocket support ([#4882](https://github.com/hoprnet/hoprnet/pull/4882))
- Increase Avado memory limit to 2GB ([#5051](https://github.com/hoprnet/hoprnet/pull/5051))

<a name="1.92"></a>

## [1.92](https://github.com/hoprnet/hoprnet/compare/release/bogota...hoprnet:release/riga)

- Removal of HOPR admin from `hoprd` ([#4420](https://github.com/hoprnet/hoprnet/pull/4420))
- Includes removal of CLI option `--admin`, `--adminHost` and `--adminPort`
- HOPR admin may now be used as a stand-alone component, see https://github.com/hoprnet/hopr-admin
- Add `--disableApiAuthentication` CLI option to allow using the API without authentication, default for Avado & Dappnode ([#4466](https://github.com/hoprnet/hoprnet/pull/4466))
- Grafana dashboards integration for all supported platforms ([#4472](https://github.com/hoprnet/hoprnet/pull/4472))
- Migrate environment checker code to Rust ([#4492](https://github.com/hoprnet/hoprnet/pull/4492))
- Migrate hoprd CLI to Rust ([#4491](https://github.com/hoprnet/hoprnet/pull/4491))
- Smart contract toolchain upgrade ([#4382](https://github.com/hoprnet/hoprnet/pull/4230))
- Switch staging environment to using Gnosis Chain instead of Goerli ([#4497](https://github.com/hoprnet/hoprnet/pull/4497))
- Introduced Promiscuous Strategy v1, strategies migrated to Rust ([#4506](https://github.com/hoprnet/hoprnet/pull/4506))
- Fix of ticket validation to be order-independent, thus not rejecting older tickets ([#4527](<(https://github.com/hoprnet/hoprnet/pull/4527)>)
- Migrate `heartbeat` ping network interaction to Rust using Rust `futures::stream::Stream` ([#4539](https://github.com/hoprnet/hoprnet/pull/4539))
- Added strategy related metrics, refactoring of the strategy code ([#4537](https://github.com/hoprnet/hoprnet/pull/4537))
- API: Allow user to specify number of intermediate hops when sending a message ([#4563](https://github.com/hoprnet/hoprnet/pull/4563))
- API: Add capability-based authorization ([#4560](https://github.com/hoprnet/hoprnet/pull/4560))
- Automatic ticket redemption disabled by default, added `--autoTicketRedemption` CLI option to enable it explicitly ([#4565](https://github.com/hoprnet/hoprnet/pull/4565))
- Return explict message for unsupported closing incoming channel ([#4551](https://github.com/hoprnet/hoprnet/pull/4551)) with a fix in admin UI ([hopr-admin/#3](https://github.com/hoprnet/hopr-admin/pull/3))
- Bring back `fund` command, to open outgoing and incoming channels with counterpart ([#4566](https://github.com/hoprnet/hoprnet/pull/4566))
- Improve Grafana dashboards & compose setup ([#4479](https://github.com/hoprnet/hoprnet/pull/4479))
- API: Prevent API privilege escalation ([#4625](https://github.com/hoprnet/hoprnet/pull/4625))
- Assign commitent at ticket redemption, so that tickets can be redeemed with a gap in ticket index ([#4643](https://github.com/hoprnet/hoprnet/pull/4643))
- Make maximum parallel connections configurable (#[4675](https://github.com/hoprnet/hoprnet/pull/4675))
- Reduce overall connection timeout from 10s to 3s (#[4680](https://github.com/hoprnet/hoprnet/pull/4680))
- Migrate mixer code to Rust ([#4567](https://github.com/hoprnet/hoprnet/pull/4567))
- Primitive & basic types re-created in Rust ([#4645](https://github.com/hoprnet/hoprnet/pull/4645)))
- Automatically switch to DHT `server`-mode if node announces public addresses to DHT ([#4685](https://github.com/hoprnet/hoprnet/pull/4685))
- Enhance address to sorting when dialing nodes ([#4684](https://github.com/hoprnet/hoprnet/pull/4684))
- Adjust NAT detection mechanism to correctly detect exposed ports on GCP ([#4692](https://github.com/hoprnet/hoprnet/pull/4692))

<a name="1.91"></a>

## [1.91](https://github.com/hoprnet/hoprnet/compare/release/valencia...hoprnet:release/bogota)

- Include HOPR Staking Season 5 smart contracts ([#4221](https://github.com/hoprnet/hoprnet/pull/4221))
- Various optimizations of Rust crates ([#4221](https://github.com/hoprnet/hoprnet/pull/4260))
- Add Metrics API for Prometheus, `metrics` API endpoint and collection of various metrics ([#4233](https://github.com/hoprnet/hoprnet/pull/4233))
- Improve pre-merge check to prevent PR from merging when the upstream deployment is in the failed state ([#4294](https://github.com/hoprnet/hoprnet/pull/4294))
- Add Health Status Indicator in the Admin UI ([#4197](https://github.com/hoprnet/hoprnet/pull/4197))
- Allow connectivity indicator to be GREEN on public nodes too ([#4314](https://github.com/hoprnet/hoprnet/pull/4314))
- Show correct counterparty in the `channels` command output ([#4370](https://github.com/hoprnet/hoprnet/pull/4370))
- Docker build pipeline refactor: use Alpine Linux + toolchain base image ([#4362](https://github.com/hoprnet/hoprnet/pull/4362))
- Improve error messages passed to the User ([#4375](https://github.com/hoprnet/hoprnet/pull/4375))
- Fix channel metrics, add channel balances metrics ([#4374](https://github.com/hoprnet/hoprnet/pull/4374))
- Fix ticket redemption ([#4382](https://github.com/hoprnet/hoprnet/pull/4382))
- Increase wait timeout for on-chain transactions to 60 seconds ([#4425](https://github.com/hoprnet/hoprnet/pull/4425))
- Fix bug in waiting logic for on-chain transactions ([#4425](https://github.com/hoprnet/hoprnet/pull/4425))
- Fixed incorrect acknowledged tickets handling in the DB
- Fix non-registered nodes can connect despite they are not allowed to so ([#4454](https://github.com/hoprnet/hoprnet/pull/4454))
- Fix STUN functionality, enhance it to check if host is exposed and a keep-alive mechanism to keep NAT port mapping ([#4401](https://github.com/hoprnet/hoprnet/pull/4401))
- Fix DAppnode / AVADO announcing internal container addresses ([#4467](https://github.com/hoprnet/hoprnet/pull/4467))
- Add `--disableApiAuthentication` CLI option to allow using the API without authentication, default for Avado & Dappnode ([#4466](https://github.com/hoprnet/hoprnet/pull/4466))

---

<a name="1.90"></a>

## [1.90](https://github.com/hoprnet/hoprnet/compare/release/paleochora...hoprnet:release/valencia)

- Improve Network Registry smart contract to allow 1-to-many node registration, add enable/disable make targets ([#4008](https://github.com/hoprnet/hoprnet/pull/4091))
- Replace `yarn` with `npx` in `pluto` Docker image to run `hoprd` to fix binary discoverability issue
- Add support for communication between different releases within the same environment
- Avado: limit Docker container memory to maximum 1GB
- Reduce memory copy operations by reusing underlying memory pages ([#4168](https://github.com/hoprnet/hoprnet/pull/4168))
- Fix public node resolution for connectivity indicator ([#4205](https://github.com/hoprnet/hoprnet/pull/4205))
- Remove charset complexity validation on API token ([#4210](https://github.com/hoprnet/hoprnet/pull/4210))
- Properly encode API token passed from the Admin UI ([#4210](https://github.com/hoprnet/hoprnet/pull/4210))
- Refactor timeouts for more throughput and increase usage of iterables ([#4238](https://github.com/hoprnet/hoprnet/pull/4238))
- Fix incoming channels being listed as outgoing and vice versa in API ([#4236](https://github.com/hoprnet/hoprnet/pull/4236))
- Refactor packet forward interaction for less locking ([#4232](https://github.com/hoprnet/hoprnet/pull/4243))
- Refactor mixer to migitate backpressure ([#4232](https://github.com/hoprnet/hoprnet/pull/4243))
- Filter addresses before adding them to libp2p's PeerStore ([#4246](https://github.com/hoprnet/hoprnet/pull/4246))
- Reuse existing connections to establish relayed connections over public relay ([#4245](https://github.com/hoprnet/hoprnet/pull/4245))
- Reuse existing connections to connections to entry nodes ([#4250](https://github.com/hoprnet/hoprnet/pull/4250))
- Remove recurring DHT ping queue cleanup and turn all public relay nodes into DHT servers ([#4247](https://github.com/hoprnet/hoprnet/pull/4247))
- Various enhancements regarding memory consumption and overall efficiency, spread over multiple PRs
- Remove obsolete stream compatibility layer ([#4276](https://github.com/hoprnet/hoprnet/pull/4276))
- Allow `info` command before node startup has finished ([#4273](https://github.com/hoprnet/hoprnet/pull/4273))
- Turn libp2p dual DHT into single DHT by forking DHT package in order to avoid a memory leak https://github.com/hoprnet/hoprnet/pull/4288
- Turn db operations into `zero-copy` operations ([#4293](https://github.com/hoprnet/hoprnet/pull/4293))
- Close existing connections once there is new one ([#4281](https://github.com/hoprnet/hoprnet/pull/4281))
- Properly remove closed connections from libp2p's `ConnectionManager`([#4281](https://github.com/hoprnet/hoprnet/pull/4281))
- Reimplement stream handling `class`es with `function`s in `connect` package for better performance ([#4285](https://github.com/hoprnet/hoprnet/pull/4285))
- Fix ticket redemption mechanism & acknowledged ticket fix in the DB ([#4437](https://github.com/hoprnet/hoprnet/pull/4437))
- Pluto: Fix initialization of channels once the cluster has started up ([#4436](https://github.com/hoprnet/hoprnet/pull/4436))

---

<a name="1.89"></a>

## [1.89](https://github.com/hoprnet/hoprnet/compare/release/ouagadougou...hoprnet:release/paleochora)

### Changes

- Reduce eth_getBlockByNumber calls when indexing and sending transactions ([#3814](https://github.com/hoprnet/hoprnet/pull/3814))
- Marge hopr-stake contracs into monorepo and relevant deploy scripts
- Add additional CLI parameters `--heartbeatThreshold`, `--networkQualityThreshold` and `--onChainConfirmations`
- Allow configuration via environment variables instead of CLI parameters for all supported options
- Reenable e2e tests and enhance REST API ([#3836](https://github.com/hoprnet/hoprnet/pull/3836))
- Fix decoding error when API token contains certain characters
- Bump `libp2p@0.37` ([#3879](https://github.com/hoprnet/hoprnet/pull/3879))
- Reimplement simulated NAT using libp2p's connection gater API ([#3879](https://github.com/hoprnet/hoprnet/pull/3879))
- Refactor module injection system of `connect` module to align with `libp2p@0.37` ([#3879](https://github.com/hoprnet/hoprnet/pull/3879))
- Automatically resend queuing transactions when provider is reset or a node receives sufficient native tokens
- Make environment variables for `hoprd` accessible in Avado package configuration ([#3885](https://github.com/hoprnet/hoprnet/pull/3885))
- Add Docker image (hopr-hardhat) running hardhat using the Hopr environment and smart contracts (useful for testing and development)
- Add Docker image (hopr-pluto) running a full hoprd test cluster for local dApp testing
- Add script that allow self register and self deregister on the Network Registry contract
- Add documentations around network registry
- Refactor and simplify stream handover functionality to be more robust ([#3898](https://github.com/hoprnet/hoprnet/pull/3898))
- Added possibility to specify custom RPC provider in Avado
- Add connectivity health indicator & NR eligibility status of the node to the `info` command ([#3921](https://github.com/hoprnet/hoprnet/pull/3921))
- Fix message encoding/decoding in HOPRd ([#3943](https://github.com/hoprnet/hoprnet/pull/3943))
- Properly display the reason of `ping` failure ([#3964](https://github.com/hoprnet/hoprnet/pull/3964))
- Removed deprecated `API V1`
- Removed deprecated `--rest`, `--restHost`, and `--restPort` HOPRd flags
- Removed deprecated `fund` command within `hopr-admin`
- Upgrades to `hopr-admin` ([#3647](https://github.com/hoprnet/hoprnet/pull/3647))
  - it now adheres to the [HOPR dApp standard](https://github.com/hoprnet/hopr-community/blob/main/DAPP_STANDARD.md)
  - uses `typescript`
  - command parsing has been overhauled to support more complex commands
  - improved user experience with more consistent messages
- Warn when manual path selection contains duplicate adjacent entries
- Correctly recognize Avado/Dappnode private subnet ([#4032](https://github.com/hoprnet/hoprnet/pull/4032))
- Redesign entry node code to recycle knowledge and automatically reconnect ([#3990]](https://github.com/hoprnet/hoprnet/pull/3990))
- Add `entryNodes` command to API and `hopr-admin` ([#4049](https://github.com/hoprnet/hoprnet/pull/4049))
- Refactor Docker build process to respect lockfiles generated by Yarn and Cargo ([#4060](https://github.com/hoprnet/hoprnet/pull/4060))
- Changed `release/paleochora` default environment to `monte_rosa` in preparation of next release
- Fix broken package link in Avado ([#4082](https://github.com/hoprnet/hoprnet/pull/4082))
- Automate contract verification on Gnosis chain and Goerli testnet.
- Add support for communication between different releases within the same environment

# Breaking changes

Bump `libp2p@0.37` which came with many bugfixes, plenty of internal API changes and different module injection system and made lots of workarounds obsolete.

- Use npm-shrinkwrap to publish correct lockfiles
- Add `--provider` flag for setting a custom blockchain RPC provider
- Improvements in our API v2 unit tests ([#3643](https://github.com/hoprnet/hoprnet/pull/3643))
- Improvements in our integration E2E tests ([#3643](https://github.com/hoprnet/hoprnet/pull/3643))
- API v2 `/api/v2/node/peers` now returns `multiaddr` for connected peers ([#3643](https://github.com/hoprnet/hoprnet/pull/3643))
- Add connectivity health indicator updates to the logs ([#3816](https://github.com/hoprnet/hoprnet/pull/3816))
- Introduce Rust WASM support into the build toolchain ([#3829](https://github.com/hoprnet/hoprnet/pull/3829))
- Optimize build pipeline and migrate to Makefile ([#3851](https://github.com/hoprnet/hoprnet/pull/3851))
- When sending an Ethereum transaction, also release nonce lock if transaction is considered a duplicate ([#3856](https://github.com/hoprnet/hoprnet/pull/3856))
- Within E2E tests, disable hardhat autmining after deployment is done ([#3851](https://github.com/hoprnet/hoprnet/pull/3857))

- Migration to ECMAscript module standard (ESM), drop support for CommonJS ([#3825](https://github.com/hoprnet/hoprnet/pull/3825))

---

<a name="1.88"></a>

## [1.88](https://github.com/hoprnet/hoprnet/compare/release/lisbon...hoprnet:release/ouagadougou) (2022-03-18)

### Changes

- New API v2 endpoint `/api/v2/node/stream/websockets` ([#3514](https://github.com/hoprnet/hoprnet/issues/3514))
- Do not attempt to reconnect to relays we already have a connection to ([#3411](https://github.com/hoprnet/hoprnet/issues/3411))
- New API v2 endpoint `/api/v2/node/peers` ([#3617](https://github.com/hoprnet/hoprnet/pull/3617))
- Bug fix endpoint `/api/v2/channels/{peerId}` ([#3627](https://github.com/hoprnet/hoprnet/issues/3627))
- Various bug fixes in `core`
- Performance improvements in `core`
- Enhanced database queries through range queries and batched operations ([#3648](https://github.com/hoprnet/hoprnet/pull/3648))
- Automatically cleanup stale connections to correctly handle reconnects ([#3688](https://github.com/hoprnet/hoprnet/pull/3688))
- Add `--provider` flag for setting a custom blockchain RPC provider
- Use a default address sorter for all address classes ([#3731](https://github.com/hoprnet/hoprnet/pull/3731))
- Enhance TCP socket listening logic and cleanup keepAlive interval ([#3750](https://github.com/hoprnet/hoprnet/pull/3750))
- Try to reconnect to entry nodes after connection has been dropped ([#3751](https://github.com/hoprnet/hoprnet/pull/3751))
- Unhandled rejection in relay requests ([#3779](https://github.com/hoprnet/hoprnet/pull/3779))
- Ping & DHT query timeout increased ([#3780](https://github.com/hoprnet/hoprnet/pull/3780))
- Dial refactoring and optimization ([#3780](https://github.com/hoprnet/hoprnet/pull/3780))
- onAbort unhandled promise rejection workaround fix ([#3780](https://github.com/hoprnet/hoprnet/pull/3780))
- Fix event listener leak and increase maximum number of event listeners to 20 ([#3790](https://github.com/hoprnet/hoprnet/pull/3790))

---

<a name="1.87"></a>

## [1.87](https://github.com/hoprnet/hoprnet/compare/release/athens...hoprnet:release/lisbon) (2022-02-10)

### Changes

- Expanded API v2, covering most of the legacy hopr-admin commands ([#3367](https://github.com/hoprnet/hoprnet/pull/3367))
- New API v2 endpoints allow fetching and redeeming tickets from specific channels ([#3367](https://github.com/hoprnet/hoprnet/pull/3367))
- Flags `--rest`, `--restHost`, and `--restPort` are being deprecated in favor of `--api`, `--apiHost`, and `--apiPort`
- Fixed automatic and manual ticket redemption ([#3395](https://github.com/hoprnet/hoprnet/pull/3395))
- In-order processing of blocks and in-order processing of on-chain events ([#3392](https://github.com/hoprnet/hoprnet/pull/3392))

---

<a name="1.86"></a>

## [1.86](https://github.com/hoprnet/hoprnet/compare/release/budapest...hoprnet:release/athens) (2022-01-26)

### Changes

- Fixed behavior when no network was found or invalid password is entered ([#3147](https://github.com/hoprnet/hoprnet/pull/3147))
- Added new API v2 endpoint `/messages/sign` to support message authentication ([#3243](https://github.com/hoprnet/hoprnet/pull/3243))
- Fixed NAT-to-NAT connection using entry nodes ([#3237](https://github.com/hoprnet/hoprnet/pull/3237))
- Removed recursive reset of periodic checks and replaced recursive util function by iterative counterparts ([#3237](https://github.com/hoprnet/hoprnet/pull/3237))
- Use UPNP to determine external IP address ([#3237](https://github.com/hoprnet/hoprnet/pull/3237))
- Use a relay tag to announce in the DHT that a node acts as a relayer for a specific node ([#3237](https://github.com/hoprnet/hoprnet/pull/3237))
- Added simulated NAT to E2E tests ([#3237](https://github.com/hoprnet/hoprnet/pull/3237))
- Automatic deployment of NAT nodes to GCloud ([#3165](https://github.com/hoprnet/hoprnet/issues/3165))
- Automatically extend TTL of relay tokens in the DHT ([#3304](https://github.com/hoprnet/hoprnet/issues/3304))
- Do not dial localhost unless the port is different from the ones we're listening on ([#3321](https://github.com/hoprnet/hoprnet/pull/3321))
- Add CLI parameter `--allowLocalNodeConnections` to explicitly allow connections to localhost ([#3349](https://github.com/hoprnet/hoprnet/pull/3349))
- Add CLI parameter `--allowPrivateNodeConnections` to explicitly allow connections to private nodes ([#3390](https://github.com/hoprnet/hoprnet/pull/3390))
- Normalize protocol version before checking the relay usability ([#3442](https://github.com/hoprnet/hoprnet/pull/3442))
- Fix connection parameters to prevent stalling of the Node.js process and update maximal number of relays ([#3471](https://github.com/hoprnet/hoprnet/pull/3471))
- Fix locking issues in various parts of the code ([#3515](https://github.com/hoprnet/hoprnet/pull/3515))
- Fix unhandled promise rejection in strategy code and infinite loop in ticket redemption logic ([#3515](https://github.com/hoprnet/hoprnet/pull/3515))
- Fixed locking issues in transaction processing ([#3568](https://github.com/hoprnet/hoprnet/pull/3568))
- Publish `hoprd` and `cover-traffic-daemon` NPM packages with lockfiles for `npm` and `yarn` ([#3646](https://github.com/hoprnet/hoprnet/pull/3646))
- Upgraded libp2p to v0.36.2 which includes multiple memory-usage improvements ([#3620](https://github.com/hoprnet/hoprnet/pull/3620))
- Added new CLI parameters `--heartbeatInterval` and `--heartbeatVariance` to configure heartbeat behaviour ([#3515](https://github.com/hoprnet/hoprnet/pull/3515))

---

<a name="1.85"></a>

## [1.85](https://github.com/hoprnet/hoprnet/compare/release/prague...hoprnet:release/budapest) (2021-12-17)

### Changes

- Rest API v2 ([#3093](https://github.com/hoprnet/hoprnet/pull/3093)), see [API specification](./packages/hoprd/rest-api-v2-spec.yaml)
- Update ping to use Blake2s instead of SHA256 for response computation (([#3080](https://github.com/hoprnet/hoprnet/pull/3080)))
- Fix broken AVADO build ([#3150](https://github.com/hoprnet/hoprnet/pull/3150))

### Bugfixes

- Fix nodes talking to nodes deployed in other environments ([#3127](https://github.com/hoprnet/hoprnet/pull/3127))
- Fix issues with STUN code ([#3124](https://github.com/hoprnet/hoprnet/pull/3124))
- Fixes various issues with indexer ([#3132](https://github.com/hoprnet/hoprnet/pull/3132), [#3129](https://github.com/hoprnet/hoprnet/pull/3129), [#3111](https://github.com/hoprnet/hoprnet/pull/3111), [#3043](https://github.com/hoprnet/hoprnet/pull/3043))
- Improve handling of provider errors ([#3116](https://github.com/hoprnet/hoprnet/pull/3116))
- Improved unit tests and e2e tests and mocks ([#3115](https://github.com/hoprnet/hoprnet/pull/3115), [#3118](https://github.com/hoprnet/hoprnet/pull/3118), [#3097](https://github.com/hoprnet/hoprnet/pull/3097), [#3020](https://github.com/hoprnet/hoprnet/pull/3020))

---

<a name="1.84"></a>

## [1.84](https://github.com/hoprnet/hoprnet/compare/release/tuttlingen...hoprnet:release/prague) (2021-12-03)

### Changes

- Add better handler for unhandled Promise rejections ([#3037](https://github.com/hoprnet/hoprnet/pull/3037))
- Multiple bug fixes preventing crashes.
- `randomInteger` function is now cryptographically safe
- ECDSA signatures now use a more compact representation (64 instead 65 bytes)
- Initial commitment seed is derived using node key and channel information

---

<a name="1.83"></a>

## [1.83](https://github.com/hoprnet/hoprnet/compare/release/freiburg...hoprnet:release/tuttlingen) (2021-11-15)

# Breaking changes

Due to the configuration changes (refer to #2778), transport packets are now properly encapsulated between environments/releases.
Thus, existing nodes won't be able to communicate to new nodes.

### Changes

- Added git hash in `hopr-admin` page ([#2869](https://github.com/hoprnet/hoprnet/pull/2869))
- Remove legacy bi-directional channels code ([#2765](https://github.com/hoprnet/hoprnet/pull/2765))
- Use environments as central configuration for hoprd releases ([#2778](https://github.com/hoprnet/hoprnet/pull/2778))

<a name="1.82"></a>

## [1.82](https://github.com/hoprnet/hoprnet/compare/release/limassol...hoprnet:release/freiburg) (2021-10-15)

# Breaking changes

None

### Changes

- improve ticket redemption ([#2711](https://github.com/hoprnet/hoprnet/pull/2711))
- bump HoprChannels solidity compiler to `0.8.9` ([#2697](https://github.com/hoprnet/hoprnet/pull/2697))
- more tech team processes ([#2686](https://github.com/hoprnet/hoprnet/pull/2686))
- transaction confirmation improvements ([#2715](https://github.com/hoprnet/hoprnet/pull/2715))
- various CI/CD fixes ([#2494](https://github.com/hoprnet/hoprnet/pull/2494))
- various CT fixes ([#2634](https://github.com/hoprnet/hoprnet/pull/2634), [#2680](https://github.com/hoprnet/hoprnet/pull/2680))
- refactor commitments ([#2671](https://github.com/hoprnet/hoprnet/pull/2671))

## [1.81](https://github.com/hoprnet/hoprnet/compare/release/constantine...hoprnet:release/limassol) (2021-10-04)

# Breaking changes

Nodes are required to `Announce` before being able to have an open channel.

### Changes

- improve CI (#2466, #2475, #2540)
- switch to renovate
- require `Announce` (#2473)
- CT improvements (#2474)
- various bug fixes (#2529, #2556, #2558, #2562)
- various yellow paper updates
- dependancy version updates

---

<a name="1.75"></a>

## [1.75](https://github.com/hoprnet/hoprnet/compare/release/moscow...hoprnet:release/constantine) (2021-07-30)

# Breaking changes

Deprecate Node 14 and require Node 16

### Changes

- Automatically populate and use list of potential low-level relay nodes (#2133)
- Bind release to specific networks and contract addresses (#2104)
- Align yellow paper with smart contract
- UX improvement, including reachability of frontend and showing incoming channels (#2124)
- Allow transaction aggregation (Multicall) (#2113)
- Stack updates:
  - Node.js@16
  - libp2p@0.32
  - hopr-connect@0.2.40
