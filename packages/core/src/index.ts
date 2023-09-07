import EventEmitter from 'events'
import path from 'path'
import fs from 'fs'

import { Multiaddr, multiaddr, protocols } from '@multiformats/multiaddr'

import BN from 'bn.js'
import type { PeerId } from '@libp2p/interface-peer-id'

// @ts-ignore untyped library
import retimer from 'retimer'

import { compareAddressesLocalMode, compareAddressesPublicMode } from '@hoprnet/hopr-connect'

import {
  app_version,
  create_counter,
  create_gauge,
  create_histogram_with_buckets,
  create_multi_gauge,
  debug,
  durations,
  getBackoffRetries,
  getBackoffRetryTimeout,
  isErrorOutOfFunds,
  MIN_NATIVE_BALANCE,
  retimer as intervalTimer,
  retryWithBackoffThenThrow,
  Address,
  AccountEntry,
  AcknowledgedTicket,
  ChannelStatus,
  ChannelEntry,
  Ticket,
  Hash,
  HalfKeyChallenge,
  Balance,
  BalanceType,
  isMultiaddrLocal,
  OffchainKeypair,
  ChainKeypair,
  StrategyTickResult,
  Database,
  OffchainPublicKey,
  ApplicationData,
  PacketInteractionConfig,
  Path,
  PeerOrigin,
  PeerStatus,
  Health,
  Snapshot,
  HeartbeatConfig,
  CoreApp,
  get_peers_with_quality,
  HoprTools,
  WasmNetwork,
  WasmPing,
  WasmIndexerInteractions,
  PingConfig
} from '@hoprnet/hopr-utils'

import { INTERMEDIATE_HOPS, MAX_HOPS, PACKET_SIZE, VERSION, MAX_PARALLEL_PINGS } from './constants.js'

import { findPath } from './path/index.js'

import HoprCoreEthereum, {
  type Indexer,
  NetworkRegistryNodeNotAllowedEventName,
  NetworkRegistryNodeAllowedEventName
} from '@hoprnet/hopr-core-ethereum'

import {
  type ChannelStrategyInterface,
  isStrategy,
  OutgoingChannelStatus,
  SaneDefaults,
  Strategy,
  StrategyFactory
} from './channel-strategy.js'

import type { ResolvedNetwork } from './network.js'
import type { EventEmitter as Libp2pEmitter } from '@libp2p/interfaces/events'
import { utils as ethersUtils } from 'ethers/lib/ethers.js'
import { peerIdFromString } from '@libp2p/peer-id'

import { isIP } from 'node:net'
import { TagBloomFilter } from '@hoprnet/hoprd/lib/hoprd_hoprd.js'

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
const metric_sentMessageFailCount = create_counter(
  'core_counter_failed_send_messages',
  'Number of sent messages failures'
)
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

/// Maximum time to wait for a packet to be pushed to the interaction queue in milliseconds
const PACKET_QUEUE_TIMEOUT_MILLISECONDS = 15000n

interface NetOptions {
  ip: string
  port: number
}

type PeerStoreAddress = {
  id: PeerId
  multiaddrs: Multiaddr[]
}

export type HoprOptions = {
  network: ResolvedNetwork
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
  // disable NAT relay functionality
  noRelay?: boolean
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
  safeModule: {
    // Base URL to interact with safe transaction service
    safeTransactionServiceProvider?: string
    // Address of node's safe proxy instance
    safeAddress?: Address
    // Address of node's safe-module proxy instance
    moduleAddress?: Address
  }
}

export type NodeStatus = 'UNINITIALIZED' | 'INITIALIZING' | 'RUNNING' | 'DESTROYED'

export class Hopr extends EventEmitter {
  public status: NodeStatus = 'UNINITIALIZED'

  private stopPeriodicCheck: (() => void) | undefined
  private strategy: ChannelStrategyInterface
  private tools: HoprTools
  private networkPeers: WasmNetwork
  private pinger: WasmPing
  private index_updater: WasmIndexerInteractions
  private id: PeerId
  private main_loop: Promise<void>

  public network: ResolvedNetwork

  public indexer: Indexer

  /**
   * Create an uninitialized Hopr Node
   *
   * @constructor
   *
   * @param chainKeypair Chain key, determines node address
   * @param packetKeypair Packet key, determines peer ID
   * @param db used to persist protocol state
   * @param options
   * @param publicNodesEmitter used to pass information about newly announced nodes to transport module
   */
  public constructor(
    private chainKeypair: ChainKeypair,
    private packetKeypair: OffchainKeypair,
    public db: Database,
    private options: HoprOptions
  ) {
    super()

    this.network = options.network
    log(`using network: ${this.network.id}`)
    this.indexer = HoprCoreEthereum.getInstance().indexer // TODO temporary
    this.id = peerIdFromString(packetKeypair.to_peerid_str())
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
   */
  public async start(__initialNodes?: { id: PeerId; address: Address; multiaddrs: Multiaddr[] }[]) {
    this.status = 'INITIALIZING'
    log('Starting hopr node...')

    const connector = HoprCoreEthereum.getInstance()

    const balance = await connector.getNativeBalance(this.getEthereumAddress().to_string())

    verbose(
      `Ethereum account ${this.getEthereumAddress().to_hex()} has ${balance.to_formatted_string()}. Mininum balance is ${new Balance(
        MIN_NATIVE_BALANCE.toString(10),
        BalanceType.Native
      ).to_formatted_string()}`
    )

    if (!balance || balance.lte(balance.of_same(MIN_NATIVE_BALANCE.toString(10)))) {
      throw new Error('Cannot start node without a funded wallet')
    } else {
      log('Node has enough balance to start, initiating payment channels')
    }
    log('Node has enough to get started, continuing starting payment channels')

    verbose('Starting HoprEthereum, which will trigger the indexer')
    await connector.start()
    verbose('Waiting for indexer to find connected nodes.')

    // Add us as public node if announced
    if (!this.options.announce) {
      throw new Error('Announce option should be turned ON in Providence, only public nodes are supported')
    }

    // Fetch previous announcements from database
    const initialNodes: { id: PeerId; address: Address; multiaddrs: Multiaddr[] }[] =
      __initialNodes ?? (await this.getPublicNodes())

    // Fetch nodes in network registry from database
    let allowedNodes = initialNodes.filter(async (n) => {
      return await this.db.is_allowed_to_access_network(n.address)
    })

    log(
      'Using initial nodes: ',
      initialNodes.map((n) => n.id.toString())
    )

    // Fetch all nodes that announce themselves during startup
    const recentlyAnnouncedNodes: PeerStoreAddress[] = []
    const pushToRecentlyAnnouncedNodes = (peer: PeerStoreAddress) => recentlyAnnouncedNodes.push(peer)
    connector.indexer.on('peer', pushToRecentlyAnnouncedNodes)

    let heartbeat_cfg = new HeartbeatConfig(
      this.options?.heartbeatVariance,
      this.options?.heartbeatInterval,
      BigInt(this.options?.heartbeatThreshold)
    )

    let ping_cfg = new PingConfig(
      MAX_PARALLEL_PINGS,
      BigInt(2000) // in millis
    )

    const onAcknowledgement = (ackChallenge: HalfKeyChallenge) => {
      // Can subscribe to both: per specific message or all message acknowledgments
      this.emit(`hopr:message-acknowledged:${ackChallenge.to_hex()}`)
      this.emit('hopr:message-acknowledged', ackChallenge.to_hex())
    }

    const onAcknowledgedTicket = (ackTicket: AcknowledgedTicket) => {
      connector.emit('ticket:acknowledged', ackTicket)
    }

    let packetCfg = new PacketInteractionConfig(this.packetKeypair, this.chainKeypair)
    packetCfg.check_unrealized_balance = this.options.checkUnrealizedBalance ?? true

    const onReceivedMessage = (msg: Uint8Array) => {
      try {
        this.emit('hopr:message', ApplicationData.deserialize(msg))
      } catch (err) {
        log(`could not deserialize application data: ${err}`)
      }
    }

    log('Linking chain and packet keys')
    this.db.link_chain_and_packet_keys(this.chainKeypair.to_address(), this.packetKeypair.public(), Snapshot._default())

    const tbfPath = path.join(this.options.dataPath, 'tbf')
    let tagBloomFilter = new TagBloomFilter()
    try {
      let tbfData = new Uint8Array(fs.readFileSync(tbfPath))
      tagBloomFilter = TagBloomFilter.deserialize(tbfData)
    } catch (err) {
      error(`no tag bloom filter file found, using empty`)
    }

    log('Constructing the core application and tools')
    let coreApp = new CoreApp(
      new OffchainKeypair(this.packetKeypair.secret()),
      this.db.clone(),
      this.options.networkQualityThreshold,
      heartbeat_cfg,
      ping_cfg,
      onAcknowledgement,
      onAcknowledgedTicket,
      packetCfg,
      onReceivedMessage,
      tagBloomFilter,
      (tbfData: Uint8Array) => {
        try {
          fs.writeFileSync(tbfPath, tbfData)
        } catch (err) {
          error(`failed to save tag bloom filter data`)
        }
      },
      this.getLocalMultiaddresses().map((x) => x.toString())
    )

    this.tools = coreApp.tools()
    this.main_loop = coreApp.main_loop()

    this.pinger = this.tools.ping()
    this.index_updater = this.tools.index_updater()
    this.networkPeers = this.tools.network()

    connector.indexer.on('network-registry-eligibility-changed', async (address: Address, allowed: boolean) => {
      // If account is no longer eligible to register nodes, we might need to close existing connections,
      // otherwise there is nothing to do
      if (!allowed) {
        let pk: OffchainPublicKey
        try {
          pk = await connector.getPacketKeyOf(address)
        } catch (err) {
          // node has not announced itself, so we don't need to care
          return
        }

        await this.networkPeers.unregister(pk.to_peerid_str())
      }
    })

    connector.indexer.on(NetworkRegistryNodeAllowedEventName, this.onNetworkRegistryNodeAllowed.bind(this))
    connector.indexer.on(NetworkRegistryNodeNotAllowedEventName, this.onNetworkRegistryNodeNotAllowed.bind(this))
    connector.indexer.on('peer', this.onPeerAnnouncement.bind(this))

    // Add all entry nodes that were announced during startup or were already
    // known in the database.
    connector.indexer.off('peer', pushToRecentlyAnnouncedNodes)
    for (const announcedNode of initialNodes) {
      await this.onPeerAnnouncement(announcedNode)
    }
    for (const announcedNode of recentlyAnnouncedNodes) {
      await this.onPeerAnnouncement({ ...announcedNode, address: undefined })
    }
    // Populate p2p allowed peers on startup
    for (const allowedNode of allowedNodes) {
      await this.onNetworkRegistryNodeAllowed(allowedNode.address)
    }

    // If this is the first time the node starts up it has not registered a safe
    // yet, therefore it may announce directly. Trying that to get the first
    // announcement through. Otherwise, it may announce with the safe variant.
    if (await connector.isNodeSafeNotRegistered()) {
      log('No NodeSafeRegistry entry yet, proceeding with direct announcement')
      try {
        await this.announce(this.options.announce)
      } catch (err) {
        console.error('Could not announce directly self on-chain: ', err)
        process.exit(1)
      }
    } else {
      log('NodeSafeRegistry entry already present, proceeding with Safe-Module announcement')
      try {
        await this.announce(this.options.announce, true)
      } catch (err) {
        console.error('Could not announce through Safe-Module self on-chain: ', err)
        process.exit(1)
      }
    }

    // Possibly register node-safe pair to NodeSafeRegistry. Following that the
    // connector is set to use safe tx variants.
    try {
      log(`check node-safe registry`)
      await this.registerSafeByNode()
    } catch (err) {
      console.error('Could not register node with safe: ', err)
      process.exit(1)
    }

    // subscribe so we can process channel close events
    connector.indexer.on('own-channel-updated', this.onOwnChannelUpdated.bind(this))

    this.setChannelStrategy(this.options.strategy || StrategyFactory.getStrategy('passive'))

    log('announcing done, strategy interval')
    this.startPeriodicStrategyCheck()

    this.status = 'RUNNING'

    // TODO: change e2e tests to watch system status? Why check protocol version?
    // Log information
    // Debug log used in e2e integration tests, please don't change
    log('# STARTED NODE')
    log('ID', this.getId().toString())
    log('Protocol version', VERSION)
  }

  public async startProcessing() {
    await Promise.all([this.main_loop])
    log(`all interactions finished execution`)
  }

  private getLocalMultiaddresses(): Multiaddr[] {
    let mas: Multiaddr[] = []

    if (this.options.hosts.ip4 == undefined) {
      throw new Error('IP address of the host must be specified')
    }

    if (isIP(this.options.hosts.ip4.ip) == 0) {
      throw new Error('IP address of the host is not a valid IPv4 or IPv6 address')
    }

    if (this.options.hosts.ip4.ip == '0.0.0.0') {
      throw new Error('IP address of the host must be a specific IPv4 or IPv6 address')
    } else {
      mas.push(multiaddr(`/ip4/${this.options.hosts.ip4.ip}/tcp/${this.options.hosts.ip4.port}`))
    }

    return mas
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

  private async onNetworkRegistryNodeAllowed(node: Address): Promise<void> {
    const packetKey = await this.db.get_packet_key(node)
    if (packetKey) {
      const peerId = packetKey.to_peerid_str()
      this.index_updater.node_allowed_to_access_network(peerId)
    } else {
      log(`Skipping network registry allow event for ${node.to_string()} because the key binding isn't available yet`)
    }
  }

  private async onNetworkRegistryNodeNotAllowed(node: Address): Promise<void> {
    const packetKey = await this.db.get_packet_key(node)
    if (packetKey) {
      const peerId = packetKey.to_peerid_str()
      this.index_updater.node_not_allowed_to_access_network(peerId)
    } else {
      log(
        `Skipping network registry disallow event for ${node.to_string()} because the key binding isn't available yet`
      )
    }
  }

  /**
   * Called whenever a peer is announced
   * @param peer newly announced peer
   */
  private async onPeerAnnouncement(peer: { id: PeerId; address: Address; multiaddrs: Multiaddr[] }): Promise<void> {
    if (peer.id.equals(this.id)) {
      // Ignore announcements from ourself
      log(`Skipping announcements for ${peer}`)
      return
    }

    log(`Processing multiaddresses for peer ${peer.id.toString()}: ${peer.multiaddrs}`)
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

    log(`Registering announced peer '${peer.id.toString()} with multiaddresses: ${addrsToAdd}'`)
    this.index_updater.announce(
      peer.id.toString(),
      addrsToAdd.map((ma) => ma.toString())
    )

    // check whether the node is already registered in the network registry,
    // notify p2p layer if so
    if (peer.address && (await this.db.is_allowed_to_access_network(peer.address))) {
      this.index_updater.node_allowed_to_access_network(peer.id.toString())
    }
  }

  private async strategyOpenChannel(status: OutgoingChannelStatus) {
    try {
      const destinationAddress = Address.from_string(status.address)
      const pk = await HoprCoreEthereum.getInstance().getPacketKeyOf(Address.from_string(status.address))
      const stake = new BN(status.stake_str)

      const pId = peerIdFromString(pk.to_peerid_str())
      if (await this.isAllowedAccessToNetwork(pId)) {
        await this.networkPeers.register(pId.toString(), PeerOrigin.StrategyNewChannel)

        const hash = await this.openChannel(destinationAddress, stake)
        verbose('- opened channel', status.address, hash)
        this.emit('hopr:channel:opened', status)
      } else {
        error(`Protocol error: strategy wants to open channel to non-registered peer ${status.address}`)
      }
    } catch (e) {
      error(`strategy could not open channel to ${status.address}`, e)
    }
  }

  private async strategyCloseChannel(destination: string) {
    try {
      await this.closeChannel(Address.from_string(destination), 'outgoing')
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
      let outChannels = await this.db.get_channels_from(selfAddr.clone())
      for (let i = 0; i < outChannels.len(); i++) {
        let channel = outChannels.at(i) // TODO: why this sometimes give undefined ?
        if (channel && channel.status == ChannelStatus.Open) {
          metric_channelBalances.set(
            [channel.source.to_hex(), 'out'],
            +ethersUtils.formatEther(channel.balance.to_string())
          )
          outgoingChannels++
        }
      }

      let incomingChannels = 0
      let inChannels = await this.db.get_channels_to(selfAddr.clone())
      for (let i = 0; i < inChannels.len(); i++) {
        let channel = outChannels.at(i) // TODO: why this sometimes give undefined ?
        if (channel && channel.status == ChannelStatus.Open) {
          metric_channelBalances.set(
            [channel.source.to_hex(), 'in'],
            +ethersUtils.formatEther(channel.balance.to_string())
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
          const pk = await HoprCoreEthereum.getInstance().getPacketKeyOf(
            Address.from_string(channel.destination.to_string())
          )

          if (await this.isAllowedAccessToNetwork(peerIdFromString(pk.to_peerid_str()))) {
            await this.networkPeers.register(pk.to_peerid_str(), PeerOrigin.StrategyExistingChannel)
          } else {
            error(`Protocol error: Strategy is monitoring non-registered peer ${channel.destination.to_hex()}`)
          }
        })
      )

      // Perform the strategy tick
      tickResult = this.strategy.tick(
        new BN((await this.getBalance()).to_string()),
        await get_peers_with_quality(this.networkPeers, this.db),
        outgoingChannels.map((c) => {
          return {
            address: c.destination.to_string(),
            stake_str: c.balance.to_string(),
            status: c.status
          }
        })
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
    return app_version()
  }

  /**
   * Retrieves the current connectivity health indicator.
   */
  public async getConnectivityHealth(): Promise<Health> {
    return (await this.networkPeers.health()).unwrap()
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
    verbose('Stopping checking timeout')
    this.stopPeriodicCheck?.()

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
   * List the addresses on which the node is listening
   */
  public async getListeningAddresses(): Promise<Multiaddr[]> {
    if (this.status !== 'RUNNING') {
      // Not listening to any address unless `hopr` is running
      return []
    }
    let addrs: Multiaddr[] = (await this.networkPeers.get_peer_multiaddresses(this.id.toString())).map((mas) =>
      multiaddr(mas)
    )

    return addrs.sort(
      this.options.testing?.preferLocalAddresses ? compareAddressesLocalMode : compareAddressesPublicMode
    )
  }

  /**
   * Gets the observed addresses of a given peer.
   * @param peer peer to query for
   */
  public async getObservedAddresses(peer: PeerId): Promise<Multiaddr[]> {
    debug('Getting address for peer ' + peer)
    let addrs: Multiaddr[] = (await this.networkPeers.get_peer_multiaddresses(peer.toString())).map((mas) =>
      multiaddr(mas)
    )

    return addrs.sort(
      this.options.testing?.preferLocalAddresses ? compareAddressesLocalMode : compareAddressesPublicMode
    )
  }

  /**
   * @param msg message to send
   * @param destination PeerId of the destination
   * @param intermediatePath optional set path manually
   * @param hops optional number of required intermediate nodes
   * @param applicationTag optional tag identifying the sending application
   * @returns hex representation of ack challenge
   */
  public async sendMessage(
    msg: Uint8Array,
    destination: PeerId,
    intermediatePath?: OffchainPublicKey[],
    hops?: number,
    applicationTag?: number
  ): Promise<string> {
    if (this.status != 'RUNNING') {
      metric_sentMessageFailCount.increment()
      throw new Error('Cannot send message until the node is running')
    }

    if (msg.length > PACKET_SIZE) {
      metric_sentMessageFailCount.increment()
      throw Error(`Message does not fit into one packet. Please split message into chunks of ${PACKET_SIZE} bytes`)
    }

    let path: Path
    if (intermediatePath != undefined) {
      // Validate the manually specified intermediate path
      let withDestination = [...intermediatePath.map((pk) => pk.to_peerid_str()), destination.toString()]
      try {
        path = await Path.validated(withDestination, this.chainKeypair.to_address(), true, this.db)
      } catch (e) {
        metric_sentMessageFailCount.increment()
        throw e
      }
    } else {
      let chain_key = await this.peerIdToChainKey(destination)
      if (chain_key) {
        intermediatePath = await this.getIntermediateNodes(chain_key, hops)

        if (intermediatePath == null || !intermediatePath.length) {
          metric_sentMessageFailCount.increment()
          throw Error(`Failed to find automatic path`)
        }

        let withDestination = [...intermediatePath.map((pk) => pk.to_peerid_str()), destination.toString()]
        path = new Path(withDestination)
      } else {
        throw Error(`Failed to obtain chain key for peer id ${destination}`)
      }
    }

    metric_pathLength.observe(path.length())

    return (
      await this.tools.send_message(new ApplicationData(applicationTag, msg), path, PACKET_QUEUE_TIMEOUT_MILLISECONDS)
    ).to_hex()
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
    if (!(await this.networkPeers.contains(dest))) {
      await this.networkPeers.register(dest, PeerOrigin.ManualPing)
    }

    await this.pinger.ping(destination.toString())

    let peer_info = await this.networkPeers.get_peer_info(destination.toString())
    if (peer_info !== undefined && peer_info.last_seen >= 0) {
      return { latency: Number(peer_info.last_seen) - start }
    } else {
      return { info: 'failure', latency: -1 }
    }
  }

  /**
   * @returns a list connected peerIds
   */
  public async getConnectedPeers(): Promise<Iterable<PeerId>> {
    if (!this.networkPeers) {
      return []
    }

    const entries = await this.networkPeers.all()
    return (function* () {
      for (const entry of entries) {
        yield peerIdFromString(entry)
      }
    })()
  }

  /**
   * Takes a look into the indexer.
   * @returns a list of account entries
   */
  public async *getAccountsAnnouncedOnChain(): AsyncGenerator<AccountEntry, void, void> {
    yield* this.indexer.getAccountsAnnouncedOnChain()
  }

  /**
   * @param peerId of the node we want to get the connection info for
   * @returns various information about the connection
   */
  public async getConnectionInfo(peerId: PeerId): Promise<PeerStatus | undefined> {
    return await this.networkPeers.get_peer_info(peerId.toString())
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
   * List of addresses that is announced to other nodes
   * @dev returned list can change at runtime
   * @param peer peer to query for, default self
   * @param _timeout [optional] custom timeout for DHT query
   */
  public async getAddressesAnnouncedToDHT(peer: PeerId = this.getId(), _timeout = 5e3): Promise<Multiaddr[]> {
    let addrs: Multiaddr[] = (await this.networkPeers.get_peer_multiaddresses(peer.toString())).map((mas) =>
      multiaddr(mas)
    )

    return addrs.sort(
      this.options.testing?.preferLocalAddresses ? compareAddressesLocalMode : compareAddressesPublicMode
    )
  }

  /*
   * Register node with safe in HoprNodeSaferegistry if needed
   * @dev Promise resolves before own announcement appears in the indexer
   * @returns a Promise that resolves once announce transaction has been published
   */
  private async registerSafeByNode(): Promise<void> {
    const connector = HoprCoreEthereum.getInstance()

    try {
      log('registering node safe on-chain... ')
      const registryTxHash = await connector.registerSafeByNode()
      log('registering node safe on-chain done in tx %s', registryTxHash)
    } catch (err) {
      log('registering node safe on-chain failed')
      this.maybeEmitFundsEmptyEvent(err)
      throw new Error(`Failed to register node safe: ${err}`)
    }
  }

  /**
   * Announces address of node on-chain to be reachable by other nodes.
   * @dev Promise resolves before own announcement appears in the indexer
   * @param announceRoutableAddress publish routable address if true
   * @param useSafe use Safe-variant of call if true
   * @returns a Promise that resolves once announce transaction has been published
   */
  private async announce(announceRoutableAddress = false, useSafe = false): Promise<void> {
    let routableAddressAvailable = false

    // Address that we will announce soon
    let addrToAnnounce: Multiaddr
    const connector = HoprCoreEthereum.getInstance()

    if (announceRoutableAddress) {
      let multiaddrs = this.getLocalMultiaddresses()

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

    // Check if there was a previous announcement from us
    const ownAccount = await connector.getAccount(this.getEthereumAddress())

    // Do not announce if our last is equal to what we intend to announce
    log('known own multiaddr from previous announcement %s', ownAccount?.get_multiaddr_str())
    if (ownAccount?.get_multiaddr_str() === addrToAnnounce.toString()) {
      log(`intended address has already been announced, nothing to do`)
      return
    }

    // only announce when:
    // (1) directly, safe is not used
    // (2) safe is used, correctly set, and target has been configured with ALLOW_ALL
    try {
      if (!useSafe) {
        log(
          'announcing directy on-chain %s routable address %s',
          announceRoutableAddress && routableAddressAvailable ? 'with' : 'without',
          addrToAnnounce.toString()
        )
        const announceTxHash = await connector.announce(addrToAnnounce)
        log('announcing address %s done in tx %s', addrToAnnounce.toString(), announceTxHash)
      } else {
        const isRegisteredCorrectly = await connector.isNodeSafeRegisteredCorrectly()
        const isAnnouncementAllowed = await connector.isSafeAnnouncementAllowed()
        if (isRegisteredCorrectly && isAnnouncementAllowed) {
          log(
            'announcing via Safe-Module on-chain %s routable address %s',
            announceRoutableAddress && routableAddressAvailable ? 'with' : 'without',
            addrToAnnounce.toString()
          )
          const announceTxHash = await connector.announce(addrToAnnounce, true)
          log('announcing address %s done in tx %s', addrToAnnounce.toString(), announceTxHash)
        } else {
          // FIXME: implement path through the Safe as delegate
          error('Cannot announce new multiaddress because Safe-Module configuration does not allow it')
        }
      }
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
    verbose('Requesting hopr balance for node')
    return await HoprCoreEthereum.getInstance().getBalance(true)
  }

  public async getNativeBalance(): Promise<Balance> {
    verbose('Requesting native balance for node')
    return await HoprCoreEthereum.getInstance().getNativeBalance(this.getEthereumAddress().to_string(), true)
  }

  public async getSafeBalance(): Promise<Balance> {
    verbose('Requesting hopr balance for safe')
    return await HoprCoreEthereum.getInstance().getSafeBalance()
  }

  public async getSafeNativeBalance(): Promise<Balance> {
    verbose('Requesting native balance from safe')
    return await HoprCoreEthereum.getInstance().getNativeBalance(this.smartContractInfo().safeAddress, true)
  }

  public smartContractInfo(): {
    chain: string
    hoprTokenAddress: string
    hoprChannelsAddress: string
    hoprNetworkRegistryAddress: string
    hoprNodeSafeRegistryAddress: string
    hoprTicketPriceOracleAddress: string
    moduleAddress: string
    safeAddress: string
    noticePeriodChannelClosure: number
  } {
    return HoprCoreEthereum.getInstance().smartContractInfo()
  }

  /**
   * Open a payment channel
   *
   * @param counterparty the counterparty's address
   * @param amountToFund the amount to fund in HOPR(wei)
   */
  public async openChannel(
    counterparty: Address,
    amountToFund: BN
  ): Promise<{
    channelId: Hash
    receipt: string
  }> {
    if (this.getEthereumAddress().eq(counterparty)) {
      throw Error('Cannot open channel to self!')
    }

    const myAvailableTokens = await HoprCoreEthereum.getInstance().getSafeBalance()

    // validate 'amountToFund'
    if (amountToFund.lten(0)) {
      throw Error(`Invalid 'amountToFund' provided: ${amountToFund.toString(10)}`)
    } else if (amountToFund.gt(new BN(myAvailableTokens.to_string()))) {
      throw Error(
        `You don't have enough tokens: ${amountToFund.toString(10)}<${myAvailableTokens.to_string()} at safe address ${
          this.smartContractInfo().safeAddress
        }`
      )
    }

    try {
      return HoprCoreEthereum.getInstance().openChannel(
        counterparty,
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
  public async fundChannel(counterparty: Address, myFund: BN, counterpartyFund: BN): Promise<string> {
    const connector = HoprCoreEthereum.getInstance()
    const myBalance = await connector.getSafeBalance()
    const totalFund = myFund.add(counterpartyFund)

    // validate 'amountToFund'
    if (totalFund.lten(0)) {
      throw Error(`Invalid 'totalFund' provided: ${totalFund.toString(10)}`)
    } else if (totalFund.gt(new BN(myBalance.to_string()))) {
      throw Error(
        `You don't have enough tokens: ${totalFund.toString(10)}<${myBalance.to_string()} at safe address ${
          this.smartContractInfo().safeAddress
        }`
      )
    }

    try {
      return connector.fundChannel(
        counterparty,
        new Balance(myFund.toString(10), BalanceType.HOPR),
        new Balance(counterpartyFund.toString(10), BalanceType.HOPR)
      )
    } catch (err) {
      this.maybeEmitFundsEmptyEvent(err)
      throw new Error(`Failed to fundChannel: ${err}`)
    }
  }

  public async closeChannel(
    counterparty: Address,
    direction: 'incoming' | 'outgoing'
  ): Promise<{ receipt: string; status: ChannelStatus }> {
    const connector = HoprCoreEthereum.getInstance()
    const channel = ChannelEntry.deserialize(
      (direction === 'outgoing'
        ? await this.db.get_channel_x(this.getEthereumAddress(), counterparty)
        : await this.db.get_channel_x(counterparty, this.getEthereumAddress())
      ).serialize()
    )

    if (channel === undefined) {
      log(`The requested channel for counterparty ${counterparty.toString()} does not exist`)
      throw new Error('Requested channel does not exist')
    }

    log(`asking to close channel: ${channel.to_string()}`)

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
      if (channel.status === ChannelStatus.Open) {
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
    let list = await this.db.get_acknowledged_tickets()
    let ret: Ticket[] = []
    for (let i = 0; i < list.len(); i++) {
      ret.push(list.at(i).ticket)
    }
    return ret
  }

  public async getTickets(channelId: Hash): Promise<Ticket[]> {
    log(`looking for tickets in channel ${channelId.to_hex()}`)
    const channel = await this.db.get_channel(channelId)
    // return no tickets if channel does not exist or is not an incoming channel
    if (!channel || !channel.destination.eq(this.getEthereumAddress())) {
      return []
    }

    const ackedTickets = await this.db.get_acknowledged_tickets(channel)

    let result = []
    let current: AcknowledgedTicket | undefined

    while (true) {
      current = ackedTickets.next()

      if (current == undefined) {
        break
      } else {
        result.push(current.ticket)
      }
    }

    return result
  }

  public async getTicketStatistics() {
    const acked_tickets = await this.db.get_acknowledged_tickets()
    const pending = await this.db.get_pending_tickets_count()
    const losing = await this.db.get_losing_tickets_count()

    let totalValue = Balance.zero(BalanceType.HOPR)
    for (let i = 0; i < acked_tickets.len(); i++) {
      totalValue = totalValue.add(acked_tickets.at(i).ticket.amount)
    }

    return {
      pending,
      losing,
      winProportion: acked_tickets.len() / (acked_tickets.len() + losing) || 0,
      unredeemed: acked_tickets.len(),
      unredeemedValue: totalValue.clone(),
      redeemed: await this.db.get_redeemed_tickets_count(),
      redeemedValue: await this.db.get_redeemed_tickets_value(),
      neglected: await this.db.get_neglected_tickets_count(),
      rejected: await this.db.get_rejected_tickets_count(),
      rejectedValue: await this.db.get_rejected_tickets_value()
    }
  }

  public async redeemAllTickets() {
    await HoprCoreEthereum.getInstance().redeemAllTickets()
  }

  public async redeemTicketsInChannel(channelId: Hash) {
    log(`trying to redeem tickets in channel ${channelId.to_hex()}`)
    const channel = await this.db.get_channel(channelId)
    if (channel) {
      if (channel.destination.eq(this.getEthereumAddress())) {
        await HoprCoreEthereum.getInstance().redeemTicketsInChannel(channel)
        return
      }
      log(`cannot redeem tickets in outgoing channel ${channelId.to_hex()}`)
    }
    log(`cannot redeem tickets in unknown channel ${channelId.to_hex()}`)
  }

  /**
   * Get the channel entry between source and destination node.
   * @param src PeerId
   * @param dest PeerId
   * @returns the channel entry of those two nodes
   */
  public async getChannel(src: Address, dest: Address): Promise<ChannelEntry> {
    return await this.db.get_channel_x(src, dest)
  }

  public async getAllChannels(): Promise<ChannelEntry[]> {
    let list = await this.db.get_channels()
    let ret: ChannelEntry[] = []
    for (let i = 0; i < list.len(); i++) {
      ret.push(ChannelEntry.deserialize(list.at(i).serialize()))
    }
    return ret
  }

  public async getChannelsFrom(addr: Address): Promise<ChannelEntry[]> {
    let list = await this.db.get_channels_from(addr)
    let ret: ChannelEntry[] = []
    for (let i = 0; i < list.len(); i++) {
      ret.push(ChannelEntry.deserialize(list.at(i).serialize()))
    }
    return ret
  }

  public async getChannelsTo(addr: Address): Promise<ChannelEntry[]> {
    let list = await this.db.get_channels_to(addr)
    let ret: ChannelEntry[] = []
    for (let i = 0; i < list.len(); i++) {
      ret.push(ChannelEntry.deserialize(list.at(i).serialize()))
    }
    return ret
  }

  public async getPublicNodes(): Promise<{ id: PeerId; address: Address; multiaddrs: Multiaddr[] }[]> {
    const result: { id: PeerId; address: Address; multiaddrs: Multiaddr[] }[] = []
    let publicAccounts = await this.db.get_public_node_accounts()

    while (publicAccounts.len() > 0) {
      let account = publicAccounts.next()
      if (account) {
        let packetKey = await this.db.get_packet_key(account.chain_addr)
        if (packetKey) {
          result.push({
            id: peerIdFromString(packetKey.to_peerid_str()),
            address: account.chain_addr,
            multiaddrs: [new Multiaddr(account.get_multiaddr_str())]
          })
        } else {
          log(`could not retrieve packet key for address ${account.chain_addr.to_string()}`)
        }
      }
    }
    return result
  }

  public getEthereumAddress(): Address {
    return this.chainKeypair.public().to_address()
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

  public async peerIdToChainKey(id: PeerId): Promise<Address> {
    let pk = OffchainPublicKey.from_peerid_str(id.toString())
    return await this.db.get_chain_key(pk)
  }

  /**
   * @param id the peer id of the account we want to check if it's allowed access to the network
   * @returns true if allowed access
   */
  public async isAllowedAccessToNetwork(id: PeerId): Promise<boolean> {
    // Don't wait for key binding and local linking if identity is the local node
    if (this.id.equals(id)) {
      return await this.db.is_allowed_to_access_network(this.getEthereumAddress())
    }
    let chain_key: Address = await this.peerIdToChainKey(id)
    // Only check if we found a chain key, otherwise peer is not allowed
    if (chain_key) {
      return await this.db.is_allowed_to_access_network(chain_key)
    }
    return false
  }

  /**
   * Takes a destination, and optionally the desired number of hops,
   * and samples randomly intermediate nodes
   * that will relay that message before it reaches its destination.
   *
   * @param destination ethereum address of the destination node
   * @param hops optional number of required intermediate nodes (must be an integer 1,2,...MAX_HOPS inclusive)
   */
  private async getIntermediateNodes(destination: Address, hops?: number): Promise<OffchainPublicKey[]> {
    if (!hops) {
      hops = INTERMEDIATE_HOPS
    } else if (![...Array(MAX_HOPS).keys()].map((i) => i + 1).includes(hops)) {
      throw new Error(`the number of intermediate nodes must be an integer between 1 and ${MAX_HOPS} inclusive`)
    }
    const path = await findPath(
      this.getEthereumAddress(),
      destination,
      hops,
      async (address: Address) => {
        try {
          const pk = await HoprCoreEthereum.getInstance().getPacketKeyOf(address)
          return await this.networkPeers.quality_of(pk.to_peerid_str())
        } catch (e) {
          log(`error while looking up the packet key of ${address}`)
          return 0
        }
      },
      HoprCoreEthereum.getInstance().getOpenChannelsFrom.bind(HoprCoreEthereum.getInstance())
    )

    return await Promise.all(path.map((x) => HoprCoreEthereum.getInstance().getPacketKeyOf(x)))
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
              const nativeBalance = await HoprCoreEthereum.getInstance().getNativeBalance(
                this.getEthereumAddress().to_string()
              )
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

export { PEER_METADATA_PROTOCOL_VERSION } from './constants.js'
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
  type ChannelStrategyInterface
}
export { resolveNetwork, supportedNetworks, type ResolvedNetwork } from './network.js'
export { sampleOptions } from './index.mock.js'
