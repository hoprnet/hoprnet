import LibP2P from 'libp2p'
import { AddressSorter, expandVars, HoprDB, localAddressesFirst, PublicKey } from '@hoprnet/hopr-utils'
import HoprEthereum from '@hoprnet/hopr-core-ethereum'
import MPLEX from 'libp2p-mplex'
import KadDHT from 'libp2p-kad-dht'
import { NOISE } from '@chainsafe/libp2p-noise'
import PeerId from 'peer-id'
import { debug } from '@hoprnet/hopr-utils'
import Hopr, { HoprOptions, VERSION } from '.'
import { getAddrs } from './identity'
import HoprConnect, { HoprConnectOptions } from '@hoprnet/hopr-connect'
import { Multiaddr } from 'multiaddr'
import { PublicNodesEmitter } from '@hoprnet/hopr-connect/lib/types'

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
  publicNodesEmitter: PublicNodesEmitter
): Promise<LibP2P> {
  let addressSorter: AddressSorter
  if (options.preferLocalAddresses) {
    addressSorter = localAddressesFirst
    log('Preferring local addresses')
  } else {
    // Overwrite libp2p's default addressSorter to make
    // sure it doesn't fail on HOPR-flavored addresses
    addressSorter = (x) => x
    log('Addresses are sorted by default')
  }
  const libp2p = await LibP2P.create({
    peerId,
    addresses: { listen: getAddrs(peerId, options).map((x) => x.toString()) },
    // libp2p modules
    modules: {
      transport: [HoprConnect as any], // TODO re https://github.com/hoprnet/hopr-connect/issues/78
      streamMuxer: [MPLEX],
      connEncryption: [NOISE],
      dht: KadDHT
    },
    config: {
      // @ts-ignore
      protocolPrefix: 'hopr',
      transport: {
        HoprConnect: {
          initialNodes,
          publicNodes: publicNodesEmitter,
          // Tells hopr-connect to treat local and private addresses
          // as public addresses
          __useLocalAddresses: options.announceLocalAddresses
          // @dev Use these settings to simulate NAT behavior
          // __noDirectConnections: true,
          // __noWebRTCUpgrade: false
        } as HoprConnectOptions
      },
      dht: {
        enabled: true
      },
      relay: {
        enabled: false
      },
      peerDiscovery: {
        autoDial: false
      }
    },
    dialer: {
      addressSorter,
      maxDialsPerPeer: 100
    }
  })
  return libp2p
}

/*
 * General function to create a HOPR node given an identity an
 * range of options.
 * @param peerId:PeerId - Identity used by the HOPR node
 * @param options:HoprOptions - Required options to create node
 * @returns {Hopr} - HOPR node
 */
export async function createHoprNode(peerId: PeerId, options: HoprOptions, automaticChainCreation = true): Promise<Hopr> {
  const db = new HoprDB(
    PublicKey.fromPrivKey(peerId.privKey.marshal())
  )

  try {
    await db.init(options.createDbIfNotExist,
      VERSION,
      options.dbPath,
      options.forceCreateDB,
      options.environment.id,
    )
  } catch (err) {
    log(`failed init db: ${err.toString()}`)
    throw err
  }

  try {
    await db.verifyEnvironmentId(options.environment.id)
  } catch (err) {
    log(`failed to verify db: ${err.toString()}`)
    throw err
  }

  const provider = expandVars(options.environment.network.default_provider, process.env)
  log(`using provider URL: ${provider}`)
  const chain = new HoprEthereum(
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
