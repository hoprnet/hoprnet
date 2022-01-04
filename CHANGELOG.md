<a name="1.85"></a>

## Next

### Changes

- Update ping to use Blake2s instead of SHA256 for response computation (([#3080](https://github.com/hoprnet/hoprnet/pull/3080)))
- Fix broken AVADO build ([#3150](https://github.com/hoprnet/hoprnet/pull/3150))

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
