import path from 'path'
import { mkdir } from 'fs/promises'

import { type Libp2p, createLibp2p } from 'libp2p'
import { LevelDatastore } from 'datastore-level'
import type { Multiaddr } from '@multiformats/multiaddr'
import { Mplex } from '@libp2p/mplex'
import { KadDHT } from '@libp2p/kad-dht'
import { Noise } from '@chainsafe/libp2p-noise'
import type { PeerId } from '@libp2p/interface-peer-id'
import { keysPBM } from '@libp2p/crypto/keys'
import type { AddressSorter, Address } from '@libp2p/interfaces/peer-store'

import { HoprConnect, compareAddressesLocalMode, type PublicNodesEmitter } from '@hoprnet/hopr-connect'
import { HoprDB, PublicKey, debug, isAddressWithPeerId } from '@hoprnet/hopr-utils'
import HoprCoreEthereum from '@hoprnet/hopr-core-ethereum'

import Hopr, { type HoprOptions } from './index.js'
import { getAddrs } from './identity.js'
import type AccessControl from './network/access-control.js'
import { createLibp2pMock } from './libp2p.mock.js'
import { NetworkPeersOrigin } from './network/network-peers.js'
import { supportedEnvironments } from './environment.js'

const log = debug(`hopr-core:create-hopr`)

/*
 * General function to create a libp2p instance to start sending
 * messages to other nodes..
 * @param peerId:PeerId - Identity used by the HOPR node
 * @param options:HoprOptions - Required options to create node
 * @param initialNodes:{ id: PeerId; multiaddrs: Multiaddr[] } - Array of PeerIds w/their multiaddrss
 * @param publicNodesEmitter:PublicNodesEmitter Event emitter for all public nodes.
 * @param isDenied given a peerId, checks whether we want to connect to that node
 * @returns {Hopr} - HOPR node
 */
export async function createLibp2pInstance(
  peerId: PeerId,
  options: HoprOptions,
  initialNodes: { id: PeerId; multiaddrs: Multiaddr[] }[],
  publicNodes: PublicNodesEmitter,
  reviewConnection: AccessControl['reviewConnection'],
  isAllowedToAccessNetwork: Hopr['isAllowedAccessToNetwork']
): Promise<Libp2p> {
  let libp2p: Libp2p
  if (options.testing?.useMockedLibp2p) {
    // Used for quick integration testing
    libp2p = createLibp2pMock(peerId, {
      network: options.testing.mockedNetwork,
      dht: options.testing.mockedDHT
    })
  } else {
    let addressSorter: AddressSorter

    if (options.testing?.preferLocalAddresses) {
      addressSorter = (a: Address, b: Address) => compareAddressesLocalMode(a.multiaddr, b.multiaddr)
      log('Preferring local addresses')
    } else {
      // Overwrite address sorter with identity function since
      // libp2p's own address sorter function is unable to handle
      // p2p addresses, e.g. /p2p/<RELAY>/p2p-circuit/p2p/<DESTINATION>
      addressSorter = (_addr) => 0
      log('Addresses are sorted by default')
    }

    // Store the peerstore on-disk under the main data path. Ensure store is
    // opened before passing it to libp2p.
    const datastorePath = path.join(options.dataPath, 'peerstore')
    await mkdir(datastorePath, { recursive: true })
    const datastore = new LevelDatastore(datastorePath, { createIfMissing: true })
    await datastore.open()

    log(`using peerstore at ${datastorePath}`)

    // Make libp2p aware of environments
    const protocolPrefix = `/hopr/${options.environment.id}`

    // Collect supported environments and versions to be passed to HoprConnect
    // because hopr-connect doesn't have access to the protocol config file
    const supportedEnvironmentsInfo = supportedEnvironments().map((env) => {
      return { id: env.id, versionRange: env.version_range }
    })

    libp2p = await createLibp2p({
      peerId,
      addresses: { listen: getAddrs(peerId, options).map((x: Multiaddr) => x.toString()) },
      transports: [
        // @ts-ignore libp2p interface type clash
        new HoprConnect({
          config: {
            initialNodes,
            publicNodes,
            environment: options.environment.id,
            supportedEnvironments: supportedEnvironmentsInfo,
            allowLocalConnections: options.allowLocalConnections,
            allowPrivateConnections: options.allowPrivateConnections,
            // Amount of nodes for which we are willing to act as a relay
            maxRelayedConnections: 50_000,
            isAllowedToAccessNetwork
          },
          testing: {
            // Treat local and private addresses as public addresses
            __useLocalAddresses: options.testing?.announceLocalAddresses,
            // Use local addresses to dial other nodes and reply to
            // STUN queries with local and private addresses
            __preferLocalAddresses: options.testing?.preferLocalAddresses,
            // Prevent nodes from dialing each other directly
            // but allow direct connection towards relays
            __noDirectConnections: options.testing?.noDirectConnections,
            // Do not upgrade to a direct WebRTC connection, even if it
            // is available. Used to test behavior of bidirectional NATs
            __noWebRTCUpgrade: options.testing?.noWebRTCUpgrade,
            // Prevent usage of UPNP to determine external IP address
            __noUPNP: options.testing?.noUPNP
          }
        })
      ],
      streamMuxers: [new Mplex()],
      connectionEncryption: [new Noise()],
      // @ts-ignore forked DHT
      dht: new KadDHT({
        // Protocol prefixes require a trailing slash
        // @TODO disabled for compatibility reasons
        // protocolPrefix: `/${protocolPrefix}`,
        protocolPrefix,
        // Make entry nodes Kad-DHT servers
        // A network requires at least on Kad-DHT server otherwise nodes
        // will flood each other forever with Kad-DHT ping attempts
        clientMode: !options.announce,
        // Limit size of ping queue by using smaller timeouts
        pingTimeout: 2e3,
        // Only for e2e testing, use DHT `lan` mode to accept connections
        // to nodes on the same machine
        lan: options.testing?.announceLocalAddresses ?? false
      }),
      connectionManager: {
        autoDial: true,
        // Use custom sorting to prevent from problems with libp2p
        // and HOPR's relay addresses
        addressSorter,
        // Don't try to dial a peer using multiple addresses in parallel
        maxDialsPerPeer: 1,
        // If we are a public node, assume that our system is able to handle
        // more connections
        maxParallelDials: options.announce ? 250 : 50,
        // default timeout of 30s appears to be too long
        dialTimeout: 10e3
      },
      connectionGater: {
        denyDialPeer: async (peer: PeerId) => {
          return !(await reviewConnection(peer, NetworkPeersOrigin.OUTGOING_CONNECTION))
        },
        denyInboundEncryptedConnection: async (peer: PeerId) => {
          return !(await reviewConnection(peer, NetworkPeersOrigin.INCOMING_CONNECTION))
        }
      },
      relay: {
        // Conflicts with HoprConnect's own mechanism
        enabled: false
      },
      nat: {
        // Conflicts with HoprConnect's own mechanism
        enabled: false
      },
      metrics: {
        // Not needed right now
        enabled: false
      },
      ping: {
        protocolPrefix
      },
      fetch: {
        protocolPrefix
      },
      push: {
        protocolPrefix
      },
      identify: {
        protocolPrefix
      },
      peerStore: {
        // Prevents peer-store from storing addresses twice, e.g.
        // /ip4/1.2.3.4/tcp/23/p2p/16Uiu2HAmQBZA4TzjKjU5fpCSprGuM2y8mpepNwMS6ZKFATiKg68h
        // /ip4/1.2.3.4/tcp/23
        // same for
        // /p2p/16Uiu2HAkzEnkW3xGJbvpXSXmvVR177LcR4Sw7z5S1ijuBcnbVFsV/p2p-circuit
        // /p2p/16Uiu2HAkzEnkW3xGJbvpXSXmvVR177LcR4Sw7z5S1ijuBcnbVFsV/p2p-circuit/p2p/16Uiu2HAmQBZA4TzjKjU5fpCSprGuM2y8mpepNwMS6ZKFATiKg68h
        addressFilter: async (_peerId: PeerId, multiaddr: Multiaddr) => !isAddressWithPeerId(multiaddr)
      },
      datastore
    })
  }

  return libp2p
}

/*
 * General function to create a HOPR node given an identity an
 * range of options.
 * @param peerId:PeerId - Identity used by the HOPR node
 * @param options:HoprOptions - Required options to create node
 * @returns {Hopr} - HOPR node
 */
export async function createHoprNode(
  peerId: PeerId,
  options: HoprOptions,
  automaticChainCreation = true
): Promise<Hopr> {
  const db = new HoprDB(PublicKey.fromPeerId(peerId))

  try {
    const dbPath = path.join(options.dataPath, 'db')
    await db.init(options.createDbIfNotExist, dbPath, options.forceCreateDB, options.environment.id)

    // Dump entire database to a file if given by the env variable
    const dump_file = process.env.DB_DUMP ?? ''
    if (dump_file.length > 0) {
      db.dumpDatabase(dump_file)
    }
  } catch (err: unknown) {
    log(`failed init db:`, err)
    throw err
  }

  log(`using provider URL: ${options.environment.network.default_provider}`)
  const chain = new HoprCoreEthereum(
    db,
    PublicKey.fromPeerId(peerId),
    keysPBM.PrivateKey.decode(peerId.privateKey as Uint8Array).Data,
    {
      chainId: options.environment.network.chain_id,
      environment: options.environment.id,
      maxFeePerGas: options.environment.network.max_fee_per_gas,
      maxPriorityFeePerGas: options.environment.network.max_priority_fee_per_gas,
      network: options.environment.network.id,
      provider: options.environment.network.default_provider
    },
    automaticChainCreation
  )

  // Initialize connection to the blockchain
  await chain.initializeChainWrapper()

  return new Hopr(peerId, db, chain, options)
}
