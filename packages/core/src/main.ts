import path from 'path'
import { mkdir } from 'fs/promises'

import { createLibp2p, type Libp2p } from 'libp2p'
import { LevelDatastore } from 'datastore-level'
import type { Multiaddr } from '@multiformats/multiaddr'
import { Mplex } from '@libp2p/mplex'
import { KadDHT } from '@libp2p/kad-dht'
import { Noise } from '@chainsafe/libp2p-noise'
import type { PeerId } from '@libp2p/interface-peer-id'
import type { Address, AddressSorter } from '@libp2p/interfaces/peer-store'
import { keysPBM } from '@libp2p/crypto/keys'

import {
  compareAddressesLocalMode,
  compareAddressesPublicMode,
  HoprConnect,
  type PublicNodesEmitter
} from '@hoprnet/hopr-connect'
import {
  Address as Packet_Address,
  ChainKeypair,
  debug,
  isAddressWithPeerId,
  LevelDb,
  OffchainKeypair,
  u8aConcat
} from '@hoprnet/hopr-utils'
import HoprCoreEthereum from '@hoprnet/hopr-core-ethereum'

import Hopr, { type HoprOptions } from './index.js'
import { getAddrs } from './identity.js'
import { createLibp2pMock } from './libp2p.mock.js'
import { getContractData, supportedNetworks } from './network.js'
import { MultiaddrConnection } from '@libp2p/interfaces/transport'
import { Address as Ethereum_Address, core_hopr_initialize_crate, Database } from '../lib/core_hopr.js'
import { peerIdFromKeys } from '@libp2p/peer-id'

core_hopr_initialize_crate()

const log = debug(`hopr-core:create-hopr`)
const error = debug(`hopr-core:error`)

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
  packetKeypair: OffchainKeypair,
  options: HoprOptions,
  initialNodes: { id: PeerId; multiaddrs: Multiaddr[] }[],
  publicNodes: PublicNodesEmitter,
  isAllowedToAccessNetwork: Hopr['isAllowedAccessToNetwork']
): Promise<Libp2p> {
  let libp2p: Libp2p

  // Hack until migrated to rs-libp2p: put the public key to the protobuf format expected by JS PeerId
  let protoBufPrefixedPubKey = keysPBM.PublicKey.encode({
    Type: keysPBM.KeyType.Ed25519,
    Data: packetKeypair.public().serialize()
  })
  let protoBufPrefixedPrivKey = keysPBM.PrivateKey.encode({
    Type: keysPBM.KeyType.Ed25519,
    Data: u8aConcat(packetKeypair.secret(), packetKeypair.public().serialize())
  })
  const peerId = await peerIdFromKeys(protoBufPrefixedPubKey, protoBufPrefixedPrivKey)

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
      log('Address sorting: prefer local addresses')
    } else {
      // Overwrite address sorter with identity function since
      // libp2p's own address sorter function is unable to handle
      // p2p addresses, e.g. /p2p/<RELAY>/p2p-circuit/p2p/<DESTINATION>
      addressSorter = (a: Address, b: Address) => compareAddressesPublicMode(a.multiaddr, b.multiaddr)
      log('Address sorting: start with most promising addresses')
    }

    // Store the peerstore on-disk under the main data path. Ensure store is
    // opened before passing it to libp2p.
    const datastorePath = path.join(options.dataPath, 'peerstore')
    await mkdir(datastorePath, { recursive: true })
    const datastore = new LevelDatastore(datastorePath, { createIfMissing: true })
    await datastore.open()

    log(`using peerstore at ${datastorePath}`)

    // Make libp2p aware of environments
    const protocolPrefix = `/hopr/${options.network.id}`

    // Collect supported environments and versions to be passed to HoprConnect
    // because hopr-connect doesn't have access to the protocol config file
    const supportedEnvironmentsInfo = supportedNetworks().map((env) => {
      return { id: env.id, versionRange: env.version_range }
    })

    log(`creating libp2p with peer id ${peerId.toString()}`)
    libp2p = await createLibp2p({
      peerId,
      addresses: { listen: getAddrs(peerId, options).map((x: Multiaddr) => x.toString()) },
      transports: [
        // @ts-ignore libp2p interface type clash
        new HoprConnect({
          config: {
            initialNodes,
            publicNodes,
            network: options.network.id,
            supportedNetworks: supportedEnvironmentsInfo,
            allowLocalConnections: options.allowLocalConnections,
            allowPrivateConnections: options.allowPrivateConnections,
            // Amount of nodes for which we are willing to act as a relay with 2GB memory limit
            maxRelayedConnections: 2_000,
            announce: options.announce,
            isAllowedToAccessNetwork,
            noRelay: options.noRelay
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
            __localModeStun: options.testing?.localModeStun
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
        maxParallelDials: options.maxParallelConnections,
        // default timeout of 30s appears to be too long
        dialTimeout: 3e3
      },
      connectionGater: {
        denyDialPeer: async (peer: PeerId) => !(await isAllowedToAccessNetwork(peer)),
        denyInboundEncryptedConnection: async (peer: PeerId, conn: MultiaddrConnection) => {
          const isAllowed = await isAllowedToAccessNetwork(peer)

          if (!isAllowed) {
            try {
              // Connection must be closed explicitly because not yet
              // part of any data structure
              await conn.close()
            } catch (err) {
              error(`Error while closing connection to non-registered node`)
            }
          }

          return !isAllowed
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
        // FIXME: libp2p automatically adds a leading `/`
        // protocolPrefix: protocolPrefix.startsWith('/') ? protocolPrefix.slice(1) : protocolPrefix
        protocolPrefix // for compatibility
      },
      fetch: {
        // FIXME: libp2p automatically adds a leading `/`
        // protocolPrefix: protocolPrefix.startsWith('/') ? protocolPrefix.slice(1) : protocolPrefix
        protocolPrefix // for compatibility
      },
      push: {
        // FIXME: libp2p automatically adds a leading `/`
        // protocolPrefix: protocolPrefix.startsWith('/') ? protocolPrefix.slice(1) : protocolPrefix
        protocolPrefix // for compatibility
      },
      identify: {
        // FIXME: libp2p automatically adds a leading `/`
        // protocolPrefix: protocolPrefix.startsWith('/') ? protocolPrefix.slice(1) : protocolPrefix
        protocolPrefix // for compatibility
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
  chainKeypair: ChainKeypair,
  packetKeypair: OffchainKeypair,
  options: HoprOptions,
  automaticChainCreation = true
): Promise<Hopr> {
  let levelDb = new LevelDb()

  try {
    const dbPath = path.join(options.dataPath, 'db')
    await levelDb.init(options.createDbIfNotExist, dbPath, options.forceCreateDB, options.network.id)

    // Dump entire database to a file if given by the env variable
    const dump_file = process.env.DB_DUMP ?? ''
    if (dump_file.length > 0) {
      await levelDb.dump(dump_file)
    }
  } catch (err: unknown) {
    log(`failed init db:`, err)
    throw err
  }

  let db = new Database(levelDb, Ethereum_Address.deserialize(chainKeypair.public().to_address().serialize()))

  // if safe address or module address is not provided, replace with values stored in the db
  let safeAddress = options.safeModule.safeAddress
  let moduleAddress = options.safeModule.moduleAddress
  log(`options.safeModule.safeAddress: ${safeAddress}`)
  log(`options.safeModule.moduleAddress: ${moduleAddress}`)
  if (!safeAddress) {
    safeAddress = await db.get_staking_safe_address()
    if (safeAddress) {
      safeAddress = Packet_Address.deserialize(safeAddress.serialize())
    }
  }
  if (!moduleAddress) {
    moduleAddress = await db.get_staking_module_address()
    if (moduleAddress) {
      moduleAddress = Packet_Address.deserialize(moduleAddress.serialize())
    }
  }
  if (!safeAddress || !moduleAddress) {
    log(`failed to provide safe or module address:`)
    throw new Error('Hopr Node must be initialized with safe and module address')
  }

  log(`using provider URL: ${options.network.chain.default_provider}`)

  // get contract data for the given environment id and pass it on to create chain wrapper
  const resolvedContractAddresses = getContractData(options.network.id)
  log(`[DEBUG] resolvedContractAddresses ${options.network.id} ${JSON.stringify(resolvedContractAddresses, null, 2)}`)

  await HoprCoreEthereum.createInstance(
    db,
    packetKeypair,
    chainKeypair,
    {
      chainId: options.network.chain.chain_id,
      network: options.network.id,
      maxFeePerGas: options.network.chain.max_fee_per_gas,
      maxPriorityFeePerGas: options.network.chain.max_priority_fee_per_gas,
      chain: options.network.chain.id,
      provider: options.network.chain.default_provider
    },
    {
      safeTransactionServiceProvider: options.safeModule.safeTransactionServiceProvider,
      safeAddress,
      moduleAddress
    },
    resolvedContractAddresses,
    automaticChainCreation
  )

  // // Initialize connection to the blockchain
  // await chain.initializeChainWrapper(resolvedContractAddresses)

  return new Hopr(chainKeypair, packetKeypair, db, options)
}
