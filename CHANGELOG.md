<a name="1.81"></a>

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
