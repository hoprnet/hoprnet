# Changelog

<a name="1.90"></a>

## 1.90

### Changes

- Reduce eth_getBlockByNumber calls when indexing and sending transactions ([#3814](https://github.com/hoprnet/hoprnet/pull/3814))
- Marge hopr-stake contracs into monorepo and relevant deploy scripts
- Add additional CLI parameters `--heartbeatThreshold`, `--networkQualityThreshold` and `--onChainConfirmations`
- Allow configuration via environment variables instead of CLI parameters for all supported options
- Reenable e2e tests and enhance REST API ([#3836](https://github.com/hoprnet/hoprnet/pull/3836))
- Fix decoding error when API token contains certain characters
- Add Docker image running hardhat using the Hopr environment and smart contracts (useful for testing and development)
- Bump `libp2p@0.37` ([#3879](https://github.com/hoprnet/hoprnet/pull/3879))

<a name="1.89"></a>

## [1.89](https://github.com/hoprnet/hoprnet/compare/release/ouagadougou...hoprnet:master)

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

### Breaking changes

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
- Enhance TCP socket listening logic and cleanup keepAlice interval ([#3750](https://github.com/hoprnet/hoprnet/pull/3750))
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
