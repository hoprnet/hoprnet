import EventEmitter from 'events'

import { Multiaddr, protocols } from '@multiformats/multiaddr'

import BN from 'bn.js'
import { keysPBM } from '@libp2p/crypto/keys'
import { createHash } from 'crypto'
import secp256k1 from 'secp256k1'
import type { Libp2p as Libp2pType } from 'libp2p'
import type { Connection } from '@libp2p/interface-connection'
import type { Peer } from '@libp2p/interface-peer-store'
import type { PeerId } from '@libp2p/interface-peer-id'
import type { Components } from '@libp2p/interfaces/components'
import {
  compareAddressesLocalMode,
  compareAddressesPublicMode,
  maToClass,
  AddressClass,
  type HoprConnectConfig,
  PeerConnectionType
} from '@hoprnet/hopr-connect'

// @ts-ignore untyped library
import retimer from 'retimer'

import {
  convertPubKeyFromPeerId,
  create_counter,
  create_gauge,
  create_histogram_with_buckets,
  create_multi_gauge,
  createCircuitAddress,
  createRelayerKey,
  debug,
  type DialOpts,
  durations,
  getBackoffRetries,
  getBackoffRetryTimeout,
  isErrorOutOfFunds,
  isMultiaddrLocal,
  isSecp256k1PeerId,
  type LibP2PHandlerFunction,
  libp2pSendMessage,
  MIN_NATIVE_BALANCE,
  registerMetricsCollector,
  retimer as intervalTimer,
  retryWithBackoffThenThrow,
  iterableToArray
} from '@hoprnet/hopr-utils'

import {
  Address,
  AcknowledgedTicket,
  ChannelStatus,
  ChannelEntry,
  PublicKey,
  Ticket,
  Hash,
  HalfKeyChallenge,
  Balance,
  BalanceType
} from '@hoprnet/hopr-utils'

import { type HoprDB } from '@hoprnet/hopr-utils'

import { FULL_VERSION, INTERMEDIATE_HOPS, MAX_HOPS, PACKET_SIZE, VERSION, MAX_PARALLEL_PINGS } from './constants.js'

import {
  Network,
  PeerStatus,
  PeerOrigin,
  Health,
  health_to_string,
  HeartbeatConfig,
  core_network_set_panic_hook,
  core_network_gather_metrics
} from '../lib/core_network.js'
core_network_set_panic_hook()
registerMetricsCollector(core_network_gather_metrics)

import Heartbeat from './network/heartbeat.js'

import { findPath } from './path/index.js'

import HoprCoreEthereum, { type Indexer } from '@hoprnet/hopr-core-ethereum'

import {
  type ChannelStrategyInterface,
  isStrategy,
  OutgoingChannelStatus,
  SaneDefaults,
  Strategy,
  StrategyFactory,
  StrategyTickResult
} from './channel-strategy.js'

import { AcknowledgementInteraction } from './interactions/packet/acknowledgement.js'
import { PacketForwardInteraction } from './interactions/packet/forward.js'

import { Packet, PacketHelper } from './messages/index.js'
import type { ResolvedEnvironment } from './environment.js'
import { createLibp2pInstance } from './main.js'
import type { EventEmitter as Libp2pEmitter } from '@libp2p/interfaces/events'
import { utils as ethersUtils } from 'ethers/lib/ethers.js'
import { peerIdFromString } from '@libp2p/peer-id'

const CODE_P2P = protocols('p2p').code

const DEBUG_PREFIX = `hopr-core`
const log = debug(DEBUG_PREFIX)
const verbose = debug(DEBUG_PREFIX.concat(`:verbose`))
const error = debug(DEBUG_PREFIX.concat(`:error`))

// Metrics
const metric_outChannelCount = create_gauge('core_gauge_num_outgoing_channels', 'Number of outgoing channels')
const metric_inChannelCount = create_gauge('core_gauge_num_incoming_channels', 'Number of incoming channels')
const metric_channelBalances = create_multi_gauge(
  'core_mgauge_channel_balances',
  'Balances on channels with counterparties',
  ['counterparty', 'direction']
)
const metric_sentMessageCount = create_counter('core_counter_sent_messages', 'Number of sent messages')
const metric_pathLength = create_histogram_with_buckets(
  'core_histogram_path_length',
  'Distribution of number of hops of sent messages',
  new Float64Array([0, 1, 2, 3, 4])
)
const metric_strategyTicks = create_counter('core_counter_strategy_ticks', 'Number of strategy decisions (ticks)')
const metric_strategyLastOpened = create_gauge(
  'core_gauge_strategy_last_opened_channels',
  'Number of opened channels in the last strategy tick'
)
const metric_strategyLastClosed = create_gauge(
  'core_gauge_strategy_last_closed_channels',
  'Number of closed channels in the last strategy tick'
)
const metric_strategyMaxChannels = create_gauge(
  'core_gauge_strategy_max_auto_channels',
  'Maximum number of channels the current strategy can open'
)

// Using libp2p components directly because it allows us
// to bypass the API layer
type Libp2p = Libp2pType & {
  components: Components
}
interface NetOptions {
  ip: string
  port: number
}

type PeerStoreAddress = {
  id: PeerId
  multiaddrs: Multiaddr[]
}

export type HoprOptions = {
  environment: ResolvedEnvironment
  announce: boolean
  dataPath: string
  createDbIfNotExist: boolean
  forceCreateDB: boolean
  allowLocalConnections: boolean
  allowPrivateConnections: boolean
  password: string
  connector?: HoprCoreEthereum
  strategy: ChannelStrategyInterface
  hosts: {
    ip4?: NetOptions
    ip6?: NetOptions
  }
  heartbeatInterval?: number
  heartbeatThreshold?: number
  heartbeatVariance?: number
  networkQualityThreshold?: number
  onChainConfirmations?: number
  checkUnrealizedBalance?: boolean
  maxParallelConnections?: number
  testing?: {
    // when true, assume that the node is running in an isolated network and does
    // not need any connection to nodes outside of the subnet
    // default: false
    announceLocalAddresses?: boolean
    // when true, assume a testnet with multiple nodes running on the same machine
    // or in the same private IPv4 network
    // default: false
    preferLocalAddresses?: boolean
    // when true, intentionally fail on direct connections
    // to test NAT behavior
    // default: false
    noDirectConnections?: boolean
    // local-mode STUN, used for unit testing and e2e testing
    // default: false
    localModeStun?: boolean
    // when true, even if a direct WebRTC connection is possible,
    // don't do the upgrade to it to test bidirectional NAT
    // default: false
    noWebRTCUpgrade?: boolean
    // Use mocked libp2p instance instead of real one
    useMockedLibp2p?: boolean
    // When using mocked libp2p instance, use existing mocked
    // DHT to simulate decentralized networks
    mockedDHT?: Map<string, string[]>
    // When using mocked libp2p instances
    mockedNetwork?: Libp2pEmitter<any>
  }
}

export type NodeStatus = 'UNINITIALIZED' | 'INITIALIZING' | 'RUNNING' | 'DESTROYED'

export type Subscribe = ((
  protocols: string | string[],
  handler: LibP2PHandlerFunction<Promise<Uint8Array>>,
  includeReply: true,
  errHandler: (err: any) => void
) => void) &
  ((
    protocol: string | string[],
    handler: LibP2PHandlerFunction<Promise<void> | void>,
    includeReply: false,
    errHandler: (err: any) => void
  ) => void)

export type SendMessage = ((
  dest: PeerId,
  protocols: string | string[],
  msg: Uint8Array,
  includeReply: true,
  opts?: DialOpts
) => Promise<Uint8Array[]>) &
  ((dest: PeerId, protocols: string | string[], msg: Uint8Array, includeReply: false, opts?: DialOpts) => Promise<void>)

class Hopr extends EventEmitter {
  public status: NodeStatus = 'UNINITIALIZED'

  private stopPeriodicCheck: (() => void) | undefined
  private strategy: ChannelStrategyInterface
  private networkPeers: Network
  private heartbeat: Heartbeat
  private forward: PacketForwardInteraction
  private acknowledgements: AcknowledgementInteraction
  private libp2pComponents: Components
  private stopLibp2p: Libp2p['stop']
  private pubKey: PublicKey
  private knownPublicNodesCache = new Set()

  public environment: ResolvedEnvironment

  public indexer: Indexer

  /**
   * Create an uninitialized Hopr Node
   *
   * @constructor
   *
   * @param id PeerId to use, determines node address
   * @param db used to persist protocol state
   * @param connector an instance of the blockchain wrapper
   * @param options
   * @param publicNodesEmitter used to pass information about newly announced nodes to transport module
   */
  public constructor(
    private id: PeerId,
    public db: HoprDB,
    private options: HoprOptions,
    private publicNodesEmitter = new (EventEmitter as new () => HoprConnectConfig['config']['publicNodes'])()
  ) {
    super()

    if (!id.privateKey || !isSecp256k1PeerId(id)) {
      throw new Error('Hopr Node must be initialized with an id with a secp256k1 private key')
    }
    this.environment = options.environment
    log(`using environment: ${this.environment.id}`)
    this.indexer = HoprCoreEthereum.getInstance().indexer // TODO temporary
    this.pubKey = PublicKey.from_peerid_str(id.toString())
  }

  /**
   * Start node
   *
   * The node has a fairly complex lifecycle. This method should do all setup
   * required for a node to be functioning.
   *
   * If the node is not funded, it will throw.
   *
   * - Create a link to the ethereum blockchain
   *   - Finish indexing previous blocks [SLOW]
   *   - Find publicly accessible relays
   *
   * - Start LibP2P and work out our network configuration.
   *   - Pass the list of relays from the indexer
   *
   * - Wait for wallet to be funded with ETH [requires user interaction]
   *
   * - Announce address, pubkey, and multiaddr on chain.
   *
   * - Start heartbeat, automatic strategies, etc..
   *
   * @param __testingLibp2p use simulated libp2p instance for testing
   */
  public async start(__initialNodes?: { id: PeerId; multiaddrs: Multiaddr[] }[], __testingLibp2p?: Libp2p) {
    this.status = 'INITIALIZING'
    log('Starting hopr node...')

    const connector = HoprCoreEthereum.getInstance()

    const balance = await connector.getNativeBalance(false)

    verbose(
      `Ethereum account ${this.getEthereumAddress().to_hex()} has ${balance.to_formatted_string()}. Mininum balance is ${new Balance(
        MIN_NATIVE_BALANCE.toString(10),
        BalanceType.Native
      ).to_formatted_string()}`
    )

    if (!balance || balance.lte(balance.of_same(MIN_NATIVE_BALANCE.toString(10)))) {
      throw new Error('Cannot start node without a funded wallet')
    }
    log('Node has enough to get started, continuing starting payment channels')
    verbose('Starting HoprEthereum, which will trigger the indexer')
    await connector.start()
    verbose('Started HoprEthereum. Waiting for indexer to find connected nodes.')

    // Add us as public node if announced
    if (this.options.announce) {
      this.knownPublicNodesCache.add(this.id.toString())
    }

    // Fetch previous announcements from database
    const initialNodes = __initialNodes ?? (await connector.waitForPublicNodes())

    // Add all initial public nodes to public nodes cache
    initialNodes.forEach((initialNode) => this.knownPublicNodesCache.add(initialNode.id.toString()))

    // Fetch all nodes that will announces themselves during startup
    const recentlyAnnouncedNodes: PeerStoreAddress[] = []
    const pushToRecentlyAnnouncedNodes = (peer: PeerStoreAddress) => recentlyAnnouncedNodes.push(peer)
    connector.indexer.on('peer', pushToRecentlyAnnouncedNodes)

    // Initialize libp2p object and pass configuration
    const libp2p = (await createLibp2pInstance(
      this.id,
      this.options,
      initialNodes,
      this.publicNodesEmitter,
      this.isAllowedAccessToNetwork.bind(this)
    )) as Libp2p

    // Needed to stop libp2p instance
    this.stopLibp2p = libp2p.stop.bind(libp2p)

    this.libp2pComponents = libp2p.components

    const sendMessage = ((
      dest: PeerId,
      protocols: string | string[],
      msg: Uint8Array,
      includeReply: boolean,
      opts: DialOpts
    ) => libp2pSendMessage(this.libp2pComponents, dest, protocols, msg, includeReply, opts)) as SendMessage // Typescript limitation

    // Attach network health measurement functionality

    this.networkPeers = Network.build(
      this.id.toString(),
      this.options.networkQualityThreshold,
      (peer: string) => {
        let p = peerIdFromString(peer)
        this.libp2pComponents.getPeerStore().delete(p)
        this.publicNodesEmitter.emit('removePublicNode', p)
      },
      (oldHealth: Health, newHealth: Health) => this.emit('hopr:network-health-changed', oldHealth, newHealth),
      (peerId: string) => {
        let peer = peerIdFromString(peerId)
        if (this.knownPublicNodesCache.has(peerId)) return true

        // If we have a direct connection to this peer ID, declare it a public node
        if (
          libp2p.connectionManager
            .getConnections(peer)
            .flatMap((c) => c.tags ?? [])
            .includes(PeerConnectionType.DIRECT)
        ) {
          this.knownPublicNodesCache.add(peerId)
          return true
        }

        return false
      },
      (peer: string) => this.closeConnectionsTo(peerIdFromString(peer))
    )

    // initialize with all the peers identified in the peer store
    const peers: Peer[] = await this.libp2pComponents.getPeerStore().all()
    peers
      .map((peer) => peer.id.toString())
      .forEach((peerId) => {
        this.networkPeers.register(peerId, PeerOrigin.Initialization)
        log(`peer store: loaded peer ${peerId}`)
      })

    // react when network registry is enabled / disabled
    connector.indexer.on('network-registry-status-changed', async (enabled: boolean) => {
      // If Network Registry got enabled, we might need to close existing connections,
      // otherwise there is nothing to do
      if (enabled) {
        for (const connection of this.libp2pComponents.getConnectionManager().getConnections()) {
          if (!(await this.isAllowedAccessToNetwork(connection.remotePeer))) {
            this.networkPeers.unregister(connection.remotePeer.toString())
            try {
              await connection.close()
            } catch (err) {
              error(`error while closing existing connection to ${connection.remotePeer.toString()}`)
            }
          }
        }
      }
    })

    // react when an account's eligibility has changed
    connector.indexer.on(
      'network-registry-eligibility-changed',
      async (_account: Address, nodes: PublicKey[], eligible: boolean) => {
        // If account is no longer eligible to register nodes, we might need to close existing connections,
        // otherwise there is nothing to do
        if (!eligible) {
          for (const node of nodes) {
            this.networkPeers.unregister(node.to_peerid_str())

            for (const conn of this.libp2pComponents
              .getConnectionManager()
              .getConnections(peerIdFromString(node.to_peerid_str()))) {
              try {
                await conn.close()
              } catch (err) {
                error(`error while closing existing connection to ${conn.remotePeer.toString()}`)
              }
            }
          }
        }
      }
    )

    let heartbeat_config = HeartbeatConfig.build(
      MAX_PARALLEL_PINGS,
      this.options?.heartbeatVariance,
      this.options?.heartbeatInterval,
      BigInt(this.options?.heartbeatThreshold)
    )

    this.heartbeat = new Heartbeat(
      this.networkPeers,
      this.libp2pComponents,
      sendMessage,
      this.environment.id,
      heartbeat_config
    )

    this.libp2pComponents.getConnectionManager().addEventListener('peer:connect', (event: CustomEvent<Connection>) => {
      this.networkPeers.register(event.detail.remotePeer.toString(), PeerOrigin.IncomingConnection)
    })

    this.acknowledgements = new AcknowledgementInteraction(
      sendMessage,
      this.libp2pComponents,
      this.getId(),
      this.db,
      (ackChallenge: HalfKeyChallenge) => {
        // Can subscribe to both: per specific message or all message acknowledgments
        this.emit(`hopr:message-acknowledged:${ackChallenge.to_hex()}`)
        this.emit('hopr:message-acknowledged', ackChallenge.to_hex())
      },
      (ack: AcknowledgedTicket) => connector.emit('ticket:acknowledged', ack),
      this.environment
    )

    const onMessage = (msg: Uint8Array) => this.emit('hopr:message', msg)
    this.forward = new PacketForwardInteraction(
      this.libp2pComponents,
      sendMessage,
      this.getId(),
      onMessage,
      this.db,
      this.environment,
      this.acknowledgements,
      this.options
    )

    // Attach socket listener and check availability of entry nodes
    await libp2p.start()

    // Register protocols
    await this.acknowledgements.start()
    await this.forward.start()

    log('libp2p started')

    connector.indexer.on('peer', this.onPeerAnnouncement.bind(this))

    // Add all entry nodes that were announced during startup
    connector.indexer.off('peer', pushToRecentlyAnnouncedNodes)
    for (const announcedNode of recentlyAnnouncedNodes) {
      await this.onPeerAnnouncement(announcedNode)
    }

    connector.indexer.on('channel-waiting-for-commitment', this.onChannelWaitingForCommitment.bind(this))

    try {
      await this.announce(this.options.announce)
    } catch (err) {
      console.error(`Could not announce self on-chain`)
      console.error(`Observed error:`, err)
      process.exit(1)
    }
    // subscribe so we can process channel close events
    connector.indexer.on('own-channel-updated', this.onOwnChannelUpdated.bind(this))

    this.setChannelStrategy(this.options.strategy || StrategyFactory.getStrategy('passive'))

    log('announcing done, starting heartbeat & strategy interval')
    await this.heartbeat.start()
    this.startPeriodicStrategyCheck()

    this.status = 'RUNNING'

    // Log information
    // Debug log used in e2e integration tests, please don't change
    log('# STARTED NODE')
    log('ID', this.getId().toString())
    log('Protocol version', VERSION)

    log(this.printAvailableAddresses())
    log(this.printAvailableProtocols())

    // Enable DHT server-mode if announcing publicly routable addresses to the DHT
    await this.maybeEnableDhtServerMode()

    await this.maybeLogProfilingToGCloud()
  }

  /**
   * Pretty-print available addresses
   */
  private printAvailableAddresses() {
    const addrs = this.libp2pComponents.getAddressManager().getAddresses()

    if (addrs == undefined || addrs.length == 0) {
      return 'Attention: no Multiaddr registered for listening. Node might not be able to communicate.'
    }
    let out = 'Available under the following addresses:'

    for (const addr of addrs) {
      out += ` - ${addr.toString()}\n`
    }

    return out
  }

  /**
   * Pretty-print available addresses
   */
  private printAvailableProtocols() {
    const protos = this.libp2pComponents.getRegistrar().getProtocols()

    if (protos == undefined || protos.length == 0) {
      return 'Attention: no protocols registered for listening. Node might not be able communicate.'
    }
    let out = 'Listening to following protocols:'

    for (const protocol of this.libp2pComponents.getRegistrar().getProtocols()) {
      out += ` - ${protocol}\n`
    }

    return out
  }

  /**
   * Checks if we are announcing public addresses to the DHT.
   * If so, switch DHT to `server`-mode such that the node will
   * reply to DHT queries of other nodes
   */
  private async maybeEnableDhtServerMode() {
    let dht: Components['dht']
    try {
      dht = this.libp2pComponents.getDHT()
    } catch (err) {
      error(`Cannot switch DHT to server mode:`, err)
      return
    }

    if (dht.getMode() === 'server') {
      // Nothing to do
      return
    }

    let announcedAddresses = this.libp2pComponents.getTransportManager().getAddrs()

    for (const addr of announcedAddresses) {
      if ([AddressClass.Public, AddressClass.Public6].includes(maToClass(addr))) {
        await dht.setMode('server')
        break
      }
    }
  }

  private async maybeLogProfilingToGCloud() {
    if (process.env.GCLOUD) {
      try {
        var name = 'hopr_node_' + this.getId().toString().slice(-5).toLowerCase()
        ;(await import('@google-cloud/profiler'))
          .start({
            projectId: 'hoprassociation',
            serviceContext: {
              service: name,
              version: FULL_VERSION
            }
          })
          .catch((e: any) => console.log(e))
      } catch (e) {
        console.log(e)
      }
    }
  }

  private async onChannelWaitingForCommitment(c: ChannelEntry): Promise<void> {
    if (this.strategy.shouldCommitToChannel(c)) {
      log(`Found channel ${c.get_id().to_hex()} to us with unset commitment. Setting commitment`)
      try {
        await retryWithBackoffThenThrow(() => HoprCoreEthereum.getInstance().commitToChannel(c))
      } catch (err) {
        // @TODO what to do here? E.g. delete channel from db?
        error(
          `Couldn't set commitment in channel to ${c.destination.to_peerid_str()} (channelId ${c.get_id().to_hex()})`
        )
      }
    }
  }

  /*
   * Callback function used to react to on-chain channel update events.
   * Specifically we trigger the strategy on channel close handler.
   * @param channel object
   */
  private async onOwnChannelUpdated(channel: ChannelEntry): Promise<void> {
    if (channel.status === ChannelStatus.PendingToClose) {
      await this.strategy.onChannelWillClose(channel)
    }
  }

  /**
   * If error provided is considered an out of funds error
   * - it will emit that the node is out of funds
   * @param error error thrown by an ethereum transaction
   */
  private maybeEmitFundsEmptyEvent(error: any): void {
    const isOutOfFunds = isErrorOutOfFunds(error)
    if (!isOutOfFunds) return

    const address = this.getEthereumAddress().to_hex()
    log('unfunded node', address)

    if (isOutOfFunds === 'NATIVE') {
      this.emit('hopr:warning:unfundedNative', address)
    } else if (isOutOfFunds === 'HOPR') {
      this.emit('hopr:warning:unfunded', address)
    }
  }

  /**
   * Called whenever a peer is announced
   * @param peer newly announced peer
   */
  private async onPeerAnnouncement(peer: { id: PeerId; multiaddrs: Multiaddr[] }): Promise<void> {
    if (peer.id.equals(this.id)) {
      // Ignore announcements from ourself
      return
    }

    const addrsToAdd: Multiaddr[] = []
    for (const addr of peer.multiaddrs) {
      const tuples = addr.tuples()

      if (tuples.length <= 1 && tuples[0][0] == CODE_P2P) {
        // No routable address
        continue
      }

      // Remove /p2p/<PEER_ID> from Multiaddr to prevent from duplicates
      // in peer store
      addrsToAdd.push(addr.decapsulateCode(CODE_P2P))
    }

    const pubKey = convertPubKeyFromPeerId(peer.id)
    try {
      await this.libp2pComponents.getPeerStore().keyBook.set(peer.id, pubKey.bytes)
    } catch (err) {
      log(`Failed to update key peer-store with new peer ${peer.id.toString()} info`, err)
    }

    if (addrsToAdd.length > 0) {
      this.publicNodesEmitter.emit('addPublicNode', { id: peer.id, multiaddrs: addrsToAdd })

      try {
        await this.libp2pComponents.getPeerStore().addressBook.add(peer.id, addrsToAdd)
      } catch (err) {
        log(`Failed to update address peer-store with new peer ${peer.id.toString()} info`, err)
      }
    }

    this.knownPublicNodesCache.add(peer.id.toString())
  }

  private async strategyOpenChannel(status: OutgoingChannelStatus) {
    try {
      const destination = peerIdFromString(status.peer_id)
      const stake = new BN(status.stake_str)

      if (await this.isAllowedAccessToNetwork(destination)) {
        this.networkPeers.register(destination.toString(), PeerOrigin.StrategyNewChannel)

        const hash = await this.openChannel(destination, stake)
        verbose('- opened channel', destination, hash)
        this.emit('hopr:channel:opened', status)
      } else {
        error(`Protocol error: strategy wants to open channel to non-registered peer ${destination.toString()}`)
      }
    } catch (e) {
      error(`strategy could not open channel to ${status.peer_id}`, e)
    }
  }

  private async strategyCloseChannel(destination: string) {
    try {
      await this.closeChannel(peerIdFromString(destination), 'outgoing')
      verbose(`closed channel to ${destination.toString()}`)
      this.emit('hopr:channel:closed', destination)
    } catch (e) {
      error(`strategy could not close channel ${destination}`)
    }
  }

  private async updateChannelMetrics() {
    const selfAddr = this.getEthereumAddress()

    try {
      let outgoingChannels = 0
      for await (const channel of this.db.getChannelsFromIterable(selfAddr)) {
        if (channel.status == ChannelStatus.Open) {
          metric_channelBalances.set(
            [channel.source.to_address().to_hex(), 'out'],
            +ethersUtils.formatEther(channel.balance.to_string())
          )
          outgoingChannels++
        }
      }

      let incomingChannels = 0
      for await (const channel of this.db.getChannelsToIterable(selfAddr)) {
        if (channel.status == ChannelStatus.Open) {
          metric_channelBalances.set(
            [channel.source.to_address().to_hex(), 'in'],
            +ethersUtils.formatEther(channel.balance.toString())
          )
          incomingChannels++
        }
      }

      metric_inChannelCount.set(incomingChannels)
      metric_outChannelCount.set(outgoingChannels)
    } catch (e) {
      error(`error: failed to update channel metrics`, e)
    }
  }

  // On the strategy interval, poll the strategy to see what channel changes
  // need to be made.
  private async tickChannelStrategy() {
    verbose('strategy tick', this.status, this.strategy.name)
    if (this.status != 'RUNNING') {
      throw new Error('node is not RUNNING')
    }

    let tickResult: StrategyTickResult
    try {
      // Retrieve all outgoing channels
      const outgoingChannels = await this.getChannelsFrom(this.getEthereumAddress())
      verbose(`strategy tracks ${outgoingChannels.length} outgoing channels`)

      // Check if all peer ids are still registered
      await Promise.all(
        outgoingChannels.map(async (channel) => {
          if (await this.isAllowedAccessToNetwork(peerIdFromString(channel.destination.to_peerid_str()))) {
            this.networkPeers.register(channel.destination.to_peerid_str(), PeerOrigin.StrategyExistingChannel)
          } else {
            error(`Protocol error: Strategy is monitoring non-registered peer ${channel.destination.to_hex(true)}`)
          }
        })
      )

      // Perform the strategy tick
      tickResult = this.strategy.tick(
        new BN((await this.getBalance()).to_string()),
        this.networkPeers.all().values(),
        outgoingChannels.map((c) => {
          return {
            peer_id: c.destination.to_peerid_str(),
            stake_str: c.balance.to_string(),
            status: c.status
          }
        }),
        (peer_id_str: string) => this.networkPeers.quality_of(peer_id_str)
      )
      metric_strategyTicks.increment()
      metric_strategyMaxChannels.set(tickResult.max_auto_channels)
    } catch (e) {
      error(`failed to do a strategy tick`, e)
      throw new Error('error while performing strategy tick')
    }

    let allClosedChannels = tickResult.to_close()
    verbose(`strategy wants to close ${allClosedChannels.length} channels`)
    metric_strategyLastClosed.set(allClosedChannels.length)

    let allOpenedChannels: OutgoingChannelStatus[] = tickResult.to_open()
    verbose(`strategy wants to open ${allOpenedChannels.length} new channels`)
    metric_strategyLastOpened.set(allOpenedChannels.length)

    try {
      await Promise.all(allClosedChannels.map(this.strategyCloseChannel.bind(this)))
      await Promise.all(allOpenedChannels.map(this.strategyOpenChannel.bind(this)))
    } catch (e) {
      error(`error when strategy was trying to open or close channels`, e)
    }
  }

  /**
   * Returns the version of hopr-core.
   */
  public getVersion() {
    return FULL_VERSION
  }

  /**
   * Recalculates and retrieves the current connectivity health indicator.
   */
  public getConnectivityHealth() {
    return this.networkPeers.health()
  }

  /**
   * Shuts down the node and saves keys and peerBook in the database
   */
  // @TODO make modules Startable
  public async stop(): Promise<void> {
    if (this.status == 'DESTROYED') {
      throw Error(`Hopr instance already destroyed.`)
    }
    this.status = 'DESTROYED'
    this.forward?.stop()
    this.acknowledgements?.stop()
    verbose('Stopping checking timeout')
    this.stopPeriodicCheck?.()
    verbose('Stopping heartbeat & indexer')
    this.heartbeat?.stop()

    verbose('Stopping database')
    await this.db?.close()
    log(`Database closed.`)

    if (this.stopLibp2p) {
      verbose('Stopping libp2p')
      await this.stopLibp2p()
      log(`Libp2p closed.`)
    }

    // Give the operating system some extra time to close the sockets
    await new Promise((resolve) => setTimeout(resolve, 100))
  }

  /**
   * Gets the peer ID of this HOPR node.
   */
  public getId(): PeerId {
    return this.id
  }

  /**
   * List of addresses that is announced to other nodes
   * @dev returned list can change at runtime
   * @param peer peer to query for, default self
   * @param _timeout [optional] custom timeout for DHT query
   */
  public async getAddressesAnnouncedToDHT(peer: PeerId = this.getId(), _timeout = 5e3): Promise<Multiaddr[]> {
    let addrs: Multiaddr[]

    if (peer.equals(this.getId())) {
      addrs = this.libp2pComponents.getAddressManager().getAddresses()
    } else {
      addrs = await this.getObservedAddresses(peer)

      try {
        // @TODO add abort controller
        for await (const relayer of this.libp2pComponents.getContentRouting().findProviders(createRelayerKey(peer))) {
          const relayAddress = createCircuitAddress(relayer.id)
          if (addrs.findIndex((ma) => ma.equals(relayAddress)) < 0) {
            addrs.push(relayAddress)
          }
        }
      } catch (err) {
        log(`Could not find any relayer key for ${peer.toString()}`)
      }
    }

    return addrs.sort(
      this.options.testing?.preferLocalAddresses ? compareAddressesLocalMode : compareAddressesPublicMode
    )
  }

  /**
   * List the addresses on which the node is listening
   */
  public getListeningAddresses(): Multiaddr[] {
    if (this.status !== 'RUNNING') {
      // Not listening to any address unless `hopr` is running
      return []
    }
    // @TODO find a better way to do this
    // @ts-ignore undocumented method
    return this.libp2pComponents.getAddressManager().getListenAddrs()
  }

  /**
   * Gets the observed addresses of a given peer.
   * @param peer peer to query for
   */
  public async getObservedAddresses(peer: PeerId): Promise<Multiaddr[]> {
    const addresses = await this.libp2pComponents.getPeerStore().addressBook.get(peer)
    return addresses.map((addr) => addr.multiaddr)
  }

  /**
   * Validates the manual intermediate path by checking if it does not contain
   * channels that are not opened.
   * Throws an error if some channel is not opened.
   * @param intermediatePath
   */
  private async validateIntermediatePath(intermediatePath: PublicKey[]) {
    // checking if path makes sense
    for (let i = 0; i < intermediatePath.length; i++) {
      let ticketIssuer: PublicKey
      let ticketReceiver: PublicKey

      if (i == 0) {
        ticketIssuer = PublicKey.from_peerid_str(this.getId().toString())
        ticketReceiver = intermediatePath[0]
      } else {
        ticketIssuer = intermediatePath[i - 1]
        ticketReceiver = intermediatePath[i]
      }

      if (ticketIssuer.eq(ticketReceiver)) log(`WARNING: duplicated adjacent path entries.`)

      let channel: ChannelEntry
      try {
        channel = await this.db.getChannelX(ticketIssuer, ticketReceiver)
      } catch (err) {
        throw Error(
          `Channel from ${ticketIssuer.to_address().to_hex()} to ${ticketReceiver.to_address().to_hex()} not found`
        )
      }

      if (channel.status !== ChannelStatus.Open) {
        throw Error(`Channel ${channel.get_id().to_hex()} is not open`)
      }
    }
  }

  /**
   * @param msg message to send
   * @param destination PeerId of the destination
   * @param intermediatePath optional set path manually
   * @param hops optional number of required intermediate nodes
   */
  public async sendMessage(msg: Uint8Array, destination: PeerId, intermediatePath?: PublicKey[], hops?: number) {
    if (this.status != 'RUNNING') {
      throw new Error('Cannot send message until the node is running')
    }

    if (msg.length > PACKET_SIZE) {
      throw Error(`Message does not fit into one packet. Please split message into chunks of ${PACKET_SIZE} bytes`)
    }

    if (intermediatePath != undefined) {
      // Validate the manually specified intermediate path
      await this.validateIntermediatePath(intermediatePath)
    } else {
      intermediatePath = await this.getIntermediateNodes(PublicKey.from_peerid_str(destination.toString()), hops)

      if (intermediatePath == null || !intermediatePath.length) {
        throw Error(`Failed to find automatic path`)
      }
    }

    const path: PublicKey[] = [].concat(intermediatePath, [PublicKey.from_peerid_str(destination.toString())])
    metric_pathLength.observe(path.length)

    let packet: Packet
    try {
      packet = await PacketHelper.create(
        msg,
        path.map((x) => peerIdFromString(x.to_peerid_str())),
        this.getId(),
        this.db
      )
    } catch (err) {
      log(`Could not create packet ${err}`)
      throw Error(`Error while creating packet.`)
    }

    await PacketHelper.storePendingAcknowledgement(packet, this.db)

    try {
      await this.forward.interact(peerIdFromString(path[0].to_peerid_str()), packet)
    } catch (err) {
      log(`Could not send packet ${err}`)
      throw Error(`Failed to send packet.`)
    }

    metric_sentMessageCount.increment()
    return packet.ack_challenge().to_hex()
  }

  /**
   * Ping a node.
   * @param destination PeerId of the node
   * @returns latency
   */
  public async ping(destination: PeerId): Promise<{ info?: string; latency: number }> {
    let start = Date.now()

    if (!(await this.isAllowedAccessToNetwork(destination))) {
      throw Error(`Connection to node is not allowed`)
    }

    let dest = destination.toString()
    if (!this.networkPeers.contains(dest)) {
      this.networkPeers.register(dest, PeerOrigin.ManualPing)
    }

    await this.heartbeat.pingNode(destination)

    let peer_info = this.networkPeers.get_peer_info(destination.toString())
    if (peer_info !== undefined && peer_info.last_seen >= 0) {
      return { latency: Number(peer_info.last_seen) - start }
    } else {
      return { info: 'failure', latency: -1 }
    }
  }

  /**
   * @returns a list connected peerIds
   */
  public getConnectedPeers(): Iterable<PeerId> {
    if (!this.networkPeers) {
      return []
    }

    const entries = this.networkPeers.all()
    return (function* () {
      for (const entry of entries) {
        yield peerIdFromString(entry)
      }
    })()
  }

  /**
   * Takes a look into the indexer.
   * @returns a list of announced multi addresses
   */
  public async *getAddressesAnnouncedOnChain() {
    yield* this.indexer.getAddressesAnnouncedOnChain()
  }

  /**
   * @param peerId of the node we want to get the connection info for
   * @returns various information about the connection
   */
  public getConnectionInfo(peerId: PeerId): PeerStatus | undefined {
    return this.networkPeers.get_peer_info(peerId.toString())
  }

  /**
   * Closes all open connections to a peer. Used to temporarily or permanently
   * disconnect from a peer.
   * Similar to `libp2p.hangUp` but catching all errors.
   * @param peer PeerId of the peer from whom we want to disconnect
   */
  private closeConnectionsTo(peer: PeerId): void {
    const connections = this.libp2pComponents.getConnectionManager().getConnections(peer)

    for (const conn of connections) {
      // Don't block event loop
      ;(async function () {
        try {
          await conn.close()
        } catch (err: any) {
          error(`Error while intentionally closing connection to ${peer.toString()}`, err)
        }
      })()
    }
  }

  /**
   * @deprecated Used by API v1
   * @returns a string describing the connection status between
   * us and various nodes
   */
  public async connectionReport(): Promise<string> {
    if (!this.networkPeers) {
      return 'Node has not started yet'
    }
    const connected = this.networkPeers.debug_output()

    let announced: string[] = []
    for await (const announcement of HoprCoreEthereum.getInstance().indexer.getAddressesAnnouncedOnChain()) {
      announced.push(announcement.toString())
    }

    return `${connected}
    \n${announced.length} peers have announced themselves on chain:
    \n${announced.join('\n')}`
  }

  public subscribeOnConnector(event: string, callback: () => void): void {
    HoprCoreEthereum.getInstance().on(event, callback)
  }
  public emitOnConnector(event: string): void {
    HoprCoreEthereum.getInstance().emit(event)
  }

  public startPeriodicStrategyCheck() {
    const periodicCheck = async function (this: Hopr) {
      log('periodic check. Current status:', this.status)
      if (this.status != 'RUNNING') {
        return
      }
      const timer = retimer(() => {
        log('strategy tick took longer than 10 secs')
      }, 10000)
      try {
        log('Triggering tick channel strategy')
        await this.tickChannelStrategy()
        await this.updateChannelMetrics()
      } catch (e) {
        log('error in periodic check', e)
      }
      log('Clearing out logging timeout.')
      timer.clear()
      log(`Setting up timeout for ${this.strategy.tickInterval}ms`)
    }.bind(this)

    log(`Starting periodicCheck interval with ${this.strategy.tickInterval}ms`)

    this.stopPeriodicCheck = intervalTimer(periodicCheck, () => this.strategy.tickInterval)
  }

  /**
   * Announces address of node on-chain to be reachable by other nodes.
   * @dev Promise resolves before own announcement appears in the indexer
   * @param announceRoutableAddress publish routable address if true
   * @returns a Promise that resolves once announce transaction has been published
   */
  private async announce(announceRoutableAddress = false): Promise<void> {
    let routableAddressAvailable = false

    // Address that we will announce soon
    let addrToAnnounce: Multiaddr
    const connector = HoprCoreEthereum.getInstance()

    if (announceRoutableAddress) {
      let multiaddrs = await this.getAddressesAnnouncedToDHT()

      if (this.options.testing?.announceLocalAddresses) {
        multiaddrs = multiaddrs.filter((ma) => isMultiaddrLocal(ma))
      } else if (this.options.testing?.preferLocalAddresses) {
        // If we need local addresses, sort them first according to their class
        multiaddrs.sort(compareAddressesLocalMode)
      } else {
        // If we don't need local addresses, just throw them away
        multiaddrs = multiaddrs.filter((ma) => !isMultiaddrLocal(ma))
      }

      log(`available multiaddresses for on-chain announcement:`)
      for (const ma of multiaddrs) {
        log(` - ${ma.toString()}`)
      }

      const ip4 = multiaddrs.find((ma) => ma.toString().startsWith('/ip4/'))
      const ip6 = multiaddrs.find((ma) => ma.toString().startsWith('/ip6/'))

      // Prefer IPv4 addresses over IPv6 addresses, if any
      addrToAnnounce = ip4 ?? ip6

      // Submit P2P address if IPv4 or IPv6 address is not routable because link-locale, reserved or private address
      // except if testing locally, e.g. as part of an integration test
      if (addrToAnnounce == undefined) {
        addrToAnnounce = new Multiaddr('/p2p/' + this.getId().toString())
      } else {
        routableAddressAvailable = true
      }
    } else {
      addrToAnnounce = new Multiaddr('/p2p/' + this.getId().toString())
    }

    // Check if there was a previous annoucement from us
    const ownAccount = await connector.getAccount(this.getEthereumAddress())

    // Do not announce if our last is equal to what we intend to announce
    if (ownAccount?.get_multiaddress_str() === addrToAnnounce.toString()) {
      log(`intended address has already been announced, nothing to do`)
      return
    }

    try {
      log(
        'announcing on-chain %s routable address',
        announceRoutableAddress && routableAddressAvailable ? 'with' : 'without'
      )
      const announceTxHash = await connector.announce(addrToAnnounce)
      log('announcing address %s done in tx %s', addrToAnnounce.toString(), announceTxHash)
    } catch (err) {
      log('announcing address %s failed', addrToAnnounce.toString())
      this.maybeEmitFundsEmptyEvent(err)
      throw new Error(`Failed to announce address ${addrToAnnounce.toString()}: ${err}`)
    }
  }

  public setChannelStrategy(strategy: ChannelStrategyInterface): void {
    log('setting channel strategy from', this.strategy?.name, 'to', strategy.name)
    this.strategy = strategy

    HoprCoreEthereum.getInstance().on('ticket:acknowledged', async (ack: AcknowledgedTicket) => {
      try {
        await this.strategy.onAckedTicket(ack)
      } catch (err) {
        error(`Strategy error while handling acknowledged ticket`, err)
      }
    })
  }

  public getChannelStrategy(): ChannelStrategyInterface {
    return this.strategy
  }

  public async getBalance(): Promise<Balance> {
    return await HoprCoreEthereum.getInstance().getBalance(true)
  }

  public async getNativeBalance(): Promise<Balance> {
    verbose('Requesting native balance from node.')
    return await HoprCoreEthereum.getInstance().getNativeBalance(true)
  }

  public smartContractInfo(): {
    network: string
    hoprTokenAddress: string
    hoprChannelsAddress: string
    hoprNetworkRegistryAddress: string
    channelClosureSecs: number
  } {
    return HoprCoreEthereum.getInstance().smartContractInfo()
  }

  /**
   * Open a payment channel
   *
   * @param counterparty the counterparty's peerId
   * @param amountToFund the amount to fund in HOPR(wei)
   */
  public async openChannel(
    counterparty: PeerId,
    amountToFund: BN
  ): Promise<{
    channelId: Hash
    receipt: string
  }> {
    if (this.id.equals(counterparty)) {
      throw Error('Cannot open channel to self!')
    }

    const counterpartyPubKey = PublicKey.from_peerid_str(counterparty.toString())
    const myAvailableTokens = await HoprCoreEthereum.getInstance().getBalance(true)

    // validate 'amountToFund'
    if (amountToFund.lten(0)) {
      throw Error(`Invalid 'amountToFund' provided: ${amountToFund.toString(10)}`)
    } else if (amountToFund.gt(new BN(myAvailableTokens.to_string()))) {
      throw Error(
        `You don't have enough tokens: ${amountToFund.toString(
          10
        )}<${myAvailableTokens.to_string()} at address ${this.pubKey.to_address().to_hex()}`
      )
    }

    try {
      return HoprCoreEthereum.getInstance().openChannel(
        counterpartyPubKey,
        new Balance(amountToFund.toString(10), BalanceType.HOPR)
      )
    } catch (err) {
      this.maybeEmitFundsEmptyEvent(err)
      throw new Error(`Failed to openChannel: ${err}`)
    }
  }

  /**
   * Fund a payment channel
   *
   * @param counterparty the counter party's peerId
   * @param myFund the amount to fund the channel in my favor HOPR(wei)
   * @param counterpartyFund the amount to fund the channel in counterparty's favor HOPR(wei)
   */
  public async fundChannel(counterparty: PeerId, myFund: BN, counterpartyFund: BN): Promise<string> {
    const connector = HoprCoreEthereum.getInstance()
    const counterpartyPubKey = PublicKey.from_peerid_str(counterparty.toString())
    const myBalance = await connector.getBalance(false)
    const totalFund = myFund.add(counterpartyFund)

    // validate 'amountToFund'
    if (totalFund.lten(0)) {
      throw Error(`Invalid 'totalFund' provided: ${totalFund.toString(10)}`)
    } else if (totalFund.gt(new BN(myBalance.to_string()))) {
      throw Error(
        `You don't have enough tokens: ${totalFund.toString(10)}<${myBalance.to_string()} at address ${this.pubKey
          .to_address()
          .to_hex()}`
      )
    }

    try {
      return connector.fundChannel(
        counterpartyPubKey,
        new Balance(myFund.toString(10), BalanceType.HOPR),
        new Balance(counterpartyFund.toString(10), BalanceType.HOPR)
      )
    } catch (err) {
      this.maybeEmitFundsEmptyEvent(err)
      throw new Error(`Failed to fundChannel: ${err}`)
    }
  }

  public async closeChannel(
    counterparty: PeerId,
    direction: 'incoming' | 'outgoing'
  ): Promise<{ receipt: string; status: ChannelStatus }> {
    const connector = HoprCoreEthereum.getInstance()
    const counterpartyPubKey = PublicKey.from_peerid_str(counterparty.toString())
    const channel =
      direction === 'outgoing'
        ? await this.db.getChannelX(this.pubKey, counterpartyPubKey)
        : await this.db.getChannelX(counterpartyPubKey, this.pubKey)

    // TODO: should we wait for confirmation?
    if (channel.status === ChannelStatus.Closed) {
      throw new Error('Channel is already closed')
    }

    if (channel.status === ChannelStatus.Open) {
      await this.strategy.onChannelWillClose(channel)
    }

    // TODO: should remove this blocker when https://github.com/hoprnet/hoprnet/issues/4194 gets addressed
    if (direction === 'incoming') {
      log(
        `Incoming channel: ignoring closing channel ${channel
          .get_id()
          .to_hex()} because current HoprChannels contract does not support closing incoming channels.`
      )
      throw new Error('Incoming channel: Closing incoming channels currently is not supported.')
    }

    let txHash: string
    try {
      if (channel.status === ChannelStatus.Open || channel.status == ChannelStatus.WaitingForCommitment) {
        log('initiating closure of channel', channel.get_id().to_hex())
        txHash = await connector.initializeClosure(channel.source, channel.destination)
      } else {
        // verify that we passed the closure waiting period to prevent failing
        // on-chain transactions

        if (channel.closure_time_passed()) {
          txHash = await connector.finalizeClosure(channel.source, channel.destination)
        } else {
          log(
            `ignoring finalizing closure of channel ${channel
              .get_id()
              .to_hex()} because closure window is still active. Need to wait ${channel
              .remaining_closure_time()
              .toString(10)} seconds.`
          )
        }
      }
    } catch (err) {
      log('failed to close channel', err)
      this.maybeEmitFundsEmptyEvent(err)
      throw new Error(`Failed to closeChannel: ${err}`)
    }

    return { receipt: txHash, status: channel.status }
  }

  public async getAllTickets(): Promise<Ticket[]> {
    return this.db.getAcknowledgedTickets().then((list) => list.map((t) => t.ticket))
  }

  public async getTickets(peerId: PeerId): Promise<Ticket[]> {
    const selfPubKey = PublicKey.from_peerid_str(this.getId().toString())
    const counterpartyPubKey = PublicKey.from_peerid_str(peerId.toString())
    const channel = await this.db.getChannelX(counterpartyPubKey, selfPubKey)
    return this.db
      .getAcknowledgedTickets({
        channel
      })
      .then((list) => list.map((t) => t.ticket))
  }

  public async getTicketStatistics() {
    const ack = await this.db.getAcknowledgedTickets()
    const pending = await this.db.getPendingTicketCount()
    const losing = await this.db.getLosingTicketCount()
    const totalValue = (ackTickets: AcknowledgedTicket[]): Balance =>
      ackTickets.map((a) => a.ticket.amount).reduce((x, y) => x.add(y), Balance.zero(BalanceType.HOPR))

    return {
      pending,
      losing,
      winProportion: ack.length / (ack.length + losing) || 0,
      unredeemed: ack.length,
      unredeemedValue: totalValue(ack),
      redeemed: await this.db.getRedeemedTicketsCount(),
      redeemedValue: await this.db.getRedeemedTicketsValue(),
      neglected: await this.db.getNeglectedTicketsCount(),
      rejected: await this.db.getRejectedTicketsCount(),
      rejectedValue: await this.db.getRejectedTicketsValue()
    }
  }

  public async redeemAllTickets() {
    await HoprCoreEthereum.getInstance().redeemAllTickets()
  }

  public async redeemTicketsInChannel(peerId: PeerId) {
    const selfPubKey = PublicKey.from_peerid_str(this.getId().toString())
    const counterpartyPubKey = PublicKey.from_peerid_str(peerId.toString())
    const channel = await this.db.getChannelX(counterpartyPubKey, selfPubKey)
    await HoprCoreEthereum.getInstance().redeemTicketsInChannel(channel)
  }

  /**
   * Get the channel entry between source and destination node.
   * @param src PeerId
   * @param dest PeerId
   * @returns the channel entry of those two nodes
   */
  public async getChannel(src: PeerId, dest: PeerId): Promise<ChannelEntry> {
    return await this.db.getChannelX(
      PublicKey.from_peerid_str(src.toString()),
      PublicKey.from_peerid_str(dest.toString())
    )
  }

  public async getAllChannels(): Promise<ChannelEntry[]> {
    return iterableToArray(this.db.getChannelsIterable())
  }

  public async getChannelsFrom(addr: Address): Promise<ChannelEntry[]> {
    return await this.db.getChannelsFrom(addr)
  }

  public async getChannelsTo(addr: Address): Promise<ChannelEntry[]> {
    return await this.db.getChannelsTo(addr)
  }

  public async getPublicKeyOf(addr: Address): Promise<PublicKey> {
    return await HoprCoreEthereum.getInstance().getPublicKeyOf(addr)
  }

  public async getEntryNodes(): Promise<{ id: PeerId; multiaddrs: Multiaddr[] }[]> {
    return HoprCoreEthereum.getInstance().waitForPublicNodes()
  }

  // @TODO remove this
  // NB: The prefix "HOPR Signed Message: " is added as a security precaution.
  // Without it, the node could be convinced to sign a message like an Ethereum
  // transaction draining it's connected wallet funds, since they share the key.
  public signMessage(message: Uint8Array): Uint8Array {
    const taggedMessage = Uint8Array.from([...new TextEncoder().encode('HOPR Signed Message: '), ...message])

    const signature = secp256k1.ecdsaSign(
      createHash('sha256').update(taggedMessage).digest(),
      keysPBM.PrivateKey.decode(this.id.privateKey).Data
    )

    return signature.signature
  }

  public getEthereumAddress(): Address {
    return HoprCoreEthereum.getInstance().getPublicKey().to_address()
  }

  /**
   * Withdraw on-chain assets to a given address
   * @param currency either native currency or HOPR tokens
   * @param recipient the account where the assets should be transferred to
   * @param amount how many tokens to be transferred
   * @returns
   */
  public async withdraw(currency: 'NATIVE' | 'HOPR', recipient: string, amount: string): Promise<string> {
    let result: string
    try {
      result = await HoprCoreEthereum.getInstance().withdraw(currency, recipient, amount)
    } catch (err) {
      this.maybeEmitFundsEmptyEvent(err)
      throw new Error(`Failed to withdraw: ${err}`)
    }

    return result
  }

  /**
   * @param id the peer id of the account we want to check if it's allowed access to the network
   * @returns true if allowed access
   */
  public async isAllowedAccessToNetwork(id: PeerId): Promise<boolean> {
    return HoprCoreEthereum.getInstance().isAllowedAccessToNetwork(PublicKey.from_peerid_str(id.toString()))
  }

  /**
   * Takes a destination, and optionally the desired number of hops,
   * and samples randomly intermediate nodes
   * that will relay that message before it reaches its destination.
   *
   * @param destination instance of peerInfo that contains the peerId of the destination
   * @param hops optional number of required intermediate nodes (must be an integer 1,2,...MAX_HOPS inclusive)
   */
  private async getIntermediateNodes(destination: PublicKey, hops?: number): Promise<PublicKey[]> {
    if (!hops) {
      hops = INTERMEDIATE_HOPS
    } else if (![...Array(MAX_HOPS).keys()].map((i) => i + 1).includes(hops)) {
      throw new Error(`the number of intermediate nodes must be an integer between 1 and ${MAX_HOPS} inclusive`)
    }
    return await findPath(
      PublicKey.from_peerid_str(this.getId().toString()),
      destination,
      hops,
      (p: PublicKey) => this.networkPeers.quality_of(p.to_peerid_str()),
      HoprCoreEthereum.getInstance().getOpenChannelsFrom.bind(HoprCoreEthereum.getInstance())
    )
  }

  /**
   * This is a utility method to wait until the node is funded.
   * A backoff is implemented, if node has not been funded and
   * MAX_DELAY is reached, this function will reject.
   */
  public async waitForFunds(): Promise<void> {
    const minDelay = durations.seconds(1)
    const maxDelay = durations.seconds(200)
    const delayMultiple = 1.05
    try {
      await retryWithBackoffThenThrow(
        () =>
          new Promise<void>(async (resolve, reject) => {
            try {
              // call connector directly and don't use cache, since this is
              // most likely outdated during node startup
              const nativeBalance = await HoprCoreEthereum.getInstance().getNativeBalance(false)
              if (nativeBalance.gte(nativeBalance.of_same(MIN_NATIVE_BALANCE.toString(10)))) {
                resolve()
              } else {
                log('still unfunded, trying again soon')
                reject()
              }
            } catch (e) {
              log('error with native balance call, trying again soon')
              reject()
            }
          }),
        {
          minDelay,
          maxDelay,
          delayMultiple
        }
      )
    } catch {
      log(
        `unfunded for more than ${getBackoffRetryTimeout(
          minDelay,
          maxDelay,
          delayMultiple
        )} seconds and ${getBackoffRetries(minDelay, maxDelay, delayMultiple)} retries, shutting down`
      )
      // Close DB etc.
      await this.stop()
      await HoprCoreEthereum.getInstance().stop()
      process.exit(1)
    }
  }

  // Utility method to wait until the node is running successfully
  public async waitForRunning(): Promise<void> {
    if (this.status == 'RUNNING') {
      return Promise.resolve()
    }
    return new Promise((resolve) => this.once('running', resolve))
  }
}

export default Hopr
export * from './constants.js'
export { createHoprNode } from './main.js'
export {
  Strategy,
  StrategyFactory,
  StrategyTickResult,
  isStrategy,
  SaneDefaults,
  findPath,
  PeerOrigin,
  PeerStatus,
  Health,
  health_to_string,
  type ChannelStrategyInterface
}
export { resolveEnvironment, supportedEnvironments, type ResolvedEnvironment } from './environment.js'
export { CORE_CONSTANTS as CONSTANTS } from '../lib/core_misc.js'
export { sampleOptions } from './index.mock.js'

export * from './types.js'
