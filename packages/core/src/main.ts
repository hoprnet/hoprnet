import path from 'path'
import { mkdir } from 'fs/promises'

import { default as LibP2P, type Connection } from 'libp2p'
import { LevelDatastore } from 'datastore-level'
import { type AddressSorter, HoprDB, PublicKey, debug } from '@hoprnet/hopr-utils'
import HoprCoreEthereum from '@hoprnet/hopr-core-ethereum'
import Mplex from 'libp2p-mplex'
import KadDHT from 'libp2p-kad-dht'
import { NOISE } from '@chainsafe/libp2p-noise'
import type PeerId from 'peer-id'
import Hopr, { type HoprOptions } from '.'
import { getAddrs } from './identity'
import HoprConnect, {
  type HoprConnectConfig,
  type PublicNodesEmitter,
  compareAddressesLocalMode,
  compareAddressesPublicMode
} from '@hoprnet/hopr-connect'
import type { Multiaddr } from 'multiaddr'

const log = debug(`hopr-core:create-hopr`)

/*
 * General function to create a libp2p instance to start sending
 * messages to other nodes..
 * @param peerId:PeerId - Identity used by the HOPR node
 * @param options:HoprOptions - Required options to create node
 * @param initialNodes:{ id: PeerId; multiaddrs: Multiaddr[] } - Array of PeerIds w/their multiaddrss
 * @param publicNodesEmitter:PublicNodesEmitter Event emitter for all public nodes.
 * @returns {Hopr} - HOPR node
 */
export async function createLibp2pInstance(
  peerId: PeerId,
  options: HoprOptions,
  initialNodes: { id: PeerId; multiaddrs: Multiaddr[] }[],
  publicNodes: PublicNodesEmitter
): Promise<LibP2P> {
  let addressSorter: AddressSorter

  if (options.testing?.preferLocalAddresses) {
    // @TODO use address.isCertified to treat signed peer records differently
    // than observed addresses
    addressSorter = (addresses) => addresses.sort((a, b) => compareAddressesLocalMode(a.multiaddr, b.multiaddr))
    log('Preferring local addresses')
  } else {
    // Address sorter **must** be overwritten since libp2p
    // cannot handle Hopr's circuit addresses
    // @TODO use address.isCertified to treat signed peer records differently
    // than observed addresses
    addressSorter = (addresses) => addresses.sort((a, b) => compareAddressesPublicMode(a.multiaddr, b.multiaddr))
    log('Addresses are sorted by default')
  }

  // Store the peerstore on-disk under the main data path. Ensure store is
  // opened before passing it to libp2p.
  const datastorePath = path.join(options.dataPath, 'peerstore')
  await mkdir(datastorePath, { recursive: true })
  const datastore = new LevelDatastore(datastorePath, { createIfMissing: true })
  await datastore.open()

  log(`using peerstore at ${datastorePath}`)

  const libp2p = await LibP2P.create({
    peerId,
    addresses: { listen: getAddrs(peerId, options).map((x) => x.toString()) },
    // libp2p modules
    modules: {
      transport: [HoprConnect as any],
      streamMuxer: [Mplex],
      connEncryption: [NOISE],
      dht: KadDHT
    },
    // Currently disabled due to problems with serialization and deserialization
    // Configure peerstore to be persisted using LevelDB, also requires config
    // persistence to be set.
    datastore,
    peerStore: {
      persistence: true
    },
    config: {
      protocolPrefix: `hopr/${options.environment.id}`,
      transport: {
        HoprConnect: {
          config: {
            initialNodes,
            publicNodes,
            environment: options.environment.id,
            allowLocalConnections: options.allowLocalConnections,
            allowPrivateConnections: options.allowPrivateConnections,
            // Amount of nodes for which we are willing to act as a relay
            maxRelayedConnections: 50_000
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
        } as HoprConnectConfig
      },
      dht: {
        enabled: true,
        // Feed DHT with all previously announced nodes
        // @ts-ignore
        bootstrapPeers: initialNodes,
        // Answer requests from other peers
        clientMode: false
      },
      relay: {
        // Conflicts with HoprConnect's own mechanism
        enabled: false
      },
      nat: {
        // Conflicts with HoprConnect's own mechanism
        enabled: false
      }
    },
    dialer: {
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
    }
  })

  // Isolate DHTs
  const DHT_WAN_PREFIX = libp2p._dht._wan._protocol
  const DHT_LAN_PREFIX = libp2p._dht._lan._protocol

  if (DHT_WAN_PREFIX !== '/ipfs/kad/1.0.0' || DHT_LAN_PREFIX !== '/ipfs/lan/kad/1.0.0') {
    throw Error(`Libp2p DHT implementation has changed. Cannot set DHT environments`)
  }

  const HOPR_DHT_WAN_PROTOCOL = `/hopr/${options.environment.id}/kad/1.0.0`
  libp2p._dht._wan._protocol = HOPR_DHT_WAN_PROTOCOL
  libp2p._dht._wan._network._protocol = HOPR_DHT_WAN_PROTOCOL
  libp2p._dht._wan._topologyListener._protocol = HOPR_DHT_WAN_PROTOCOL

  const HOPR_DHT_LAN_PROTOCOL = `/hopr/${options.environment.id}/lan/kad/1.0.0`
  libp2p._dht._lan._protocol = HOPR_DHT_LAN_PROTOCOL
  libp2p._dht._lan._network._protocol = HOPR_DHT_LAN_PROTOCOL
  libp2p._dht._lan._topologyListener._protocol = HOPR_DHT_LAN_PROTOCOL

  const onConnection = libp2p.upgrader.onConnection

  // @TODO implement whitelisting support
  libp2p.upgrader.onConnection = (conn: Connection) => {
    // if (isWhitelisted()) {
    onConnection(conn)
    // }
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
  const db = new HoprDB(PublicKey.fromPrivKey(peerId.privKey.marshal()))

  try {
    const dbPath = path.join(options.dataPath, 'db')
    await db.init(options.createDbIfNotExist, dbPath, options.forceCreateDB, options.environment.id)
  } catch (err: unknown) {
    log(`failed init db:`, err)
    throw err
  }

  log(`using provider URL: ${options.environment.network.default_provider}`)
  const chain = new HoprCoreEthereum(
    db,
    PublicKey.fromPeerId(peerId),
    peerId.privKey.marshal(),
    {
      chainId: options.environment.network.chain_id,
      environment: options.environment.id,
      gasPrice: options.environment.network.gas_price,
      network: options.environment.network.id,
      provider: options.environment.network.default_provider
    },
    automaticChainCreation
  )

  // Initialize connection to the blockchain
  await chain.initializeChainWrapper()

  return new Hopr(peerId, db, chain, options)
}
