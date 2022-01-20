import { default as LibP2P, type Connection } from 'libp2p'
import { type AddressSorter, expandVars, HoprDB, localAddressesFirst, PublicKey } from '@hoprnet/hopr-utils'
import HoprCoreEthereum from '@hoprnet/hopr-core-ethereum'
import MPLEX from 'libp2p-mplex'
import KadDHT from 'libp2p-kad-dht'
import { NOISE } from '@chainsafe/libp2p-noise'
import type PeerId from 'peer-id'
import { debug } from '@hoprnet/hopr-utils'
import Hopr, { type HoprOptions, VERSION } from '.'
import { getAddrs } from './identity'
import HoprConnect, { type HoprConnectConfig, type PublicNodesEmitter } from '@hoprnet/hopr-connect'
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
    addressSorter = localAddressesFirst
    log('Preferring local addresses')
  } else {
    log('Addresses are sorted by default')
  }

  const libp2p = await LibP2P.create({
    peerId,
    addresses: { listen: getAddrs(peerId, options).map((x) => x.toString()) },
    // libp2p modules
    modules: {
      transport: [HoprConnect as any],
      streamMuxer: [MPLEX],
      connEncryption: [NOISE as any],
      dht: KadDHT
    },
    config: {
      protocolPrefix: `hopr/${options.environment.id}`,
      transport: {
        HoprConnect: {
          config: {
            initialNodes,
            publicNodes,
            environment: options.environment.id
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
        bootstrapPeers: initialNodes
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
      addressSorter,
      maxDialsPerPeer: 100
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
    await db.init(options.createDbIfNotExist, VERSION, options.dbPath, options.forceCreateDB, options.environment.id)
  } catch (err) {
    log(`failed init db: ${err.toString()}`)
    throw err
  }

  const provider = expandVars(options.environment.network.default_provider, process.env)
  log(`using provider URL: ${provider}`)
  const chain = new HoprCoreEthereum(
    db,
    PublicKey.fromPeerId(peerId),
    peerId.privKey.marshal(),
    {
      chainId: options.environment.network.chain_id,
      environment: options.environment.id,
      gasPrice: options.environment.network.gasPrice,
      network: options.environment.network.id,
      provider
    },
    automaticChainCreation
  )
  const node = new Hopr(peerId, db, chain, options)
  return node
}
