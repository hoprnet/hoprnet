import EventEmitter from 'events'
import path from 'path'
import fs from 'fs'

import { Multiaddr, multiaddr, protocols } from '@multiformats/multiaddr'

import BN from 'bn.js'
import type { PeerId } from '@libp2p/interface-peer-id'

// @ts-ignore untyped library
import retimer from 'retimer'

import {
  AccountEntry,
  AcknowledgedTicket,
  Address,
  app_version,
  ApplicationData,
  Balance,
  BalanceType,
  ChainKeypair,
  ChannelDirection,
  ChannelEntry,
  ChannelStatus,
  close_channel,
  compareAddressesLocalMode,
  compareAddressesPublicMode,
  create_counter,
  create_gauge,
  create_histogram_with_buckets,
  create_multi_gauge,
  Database,
  debug,
  durations,
  fund_channel,
  getBackoffRetries,
  getBackoffRetryTimeout,
  HalfKeyChallenge,
  Hash,
  Health,
  HoprdConfig,
  HoprTools,
  isErrorOutOfFunds,
  isMultiaddrLocal,
  MIN_NATIVE_BALANCE,
  OffchainKeypair,
  OffchainPublicKey,
  open_channel,
  PacketInteractionConfig,
  Path,
  PeerOrigin,
  PeerStatus,
  PingConfig,
  redeem_all_tickets,
  redeem_tickets_in_channel,
  redeem_tickets_with_counterparty,
  retimer as intervalTimer,
  retryWithBackoffThenThrow,
  Snapshot,
  CoreApp,
  Ticket,
  WasmIndexerInteractions,
  WasmNetwork,
  WasmPing,
  WasmTxExecutor,
  withdraw
} from '@hoprnet/hopr-utils'

import { INTERMEDIATE_HOPS, MAX_HOPS, MAX_PARALLEL_PINGS, PACKET_SIZE, VERSION } from './constants.js'

import { findPath } from './path/index.js'

import HoprCoreEthereum, {
  type Indexer,
  NetworkRegistryNodeAllowedEventName,
  NetworkRegistryNodeNotAllowedEventName
} from '@hoprnet/hopr-core-ethereum'

import { type ResolvedNetwork, resolveNetwork } from './network.js'
import { utils as ethersUtils } from 'ethers/lib/ethers.js'
import { peerIdFromString } from '@libp2p/peer-id'

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

/// Maximum time to wait for a packet to be pushed to the interaction queue in milliseconds
const PACKET_QUEUE_TIMEOUT_MILLISECONDS = 15000n

const TICKET_AGGREGATION_TIMEOUT_MILLISECONDS = 10000n

type PeerStoreAddress = {
  id: PeerId
  multiaddrs: Multiaddr[]
}

export type NodeStatus = 'UNINITIALIZED' | 'INITIALIZING' | 'RUNNING' | 'DESTROYED'

export class Hopr extends EventEmitter {
  public status: NodeStatus = 'UNINITIALIZED'

  private stopPeriodicCheck: (() => void) | undefined
  private tools: HoprTools
  private networkPeers: WasmNetwork
  private pinger: WasmPing
  private index_updater: WasmIndexerInteractions
  private id: PeerId
  private main_loop: Promise<void>
  private hasAnnounced: boolean = false
  private hasNodeSafeRegistered: boolean = false

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
    private cfg: HoprdConfig
  ) {
    super()

    this.network = resolveNetwork(cfg.network, cfg.chain.provider)
    log(`Using network: ${this.network.id}`)
    this.indexer = HoprCoreEthereum.getInstance().indexer
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
    if (!this.cfg.chain.announce) {
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

    let ping_cfg = new PingConfig(
      MAX_PARALLEL_PINGS,
      BigInt(2000) // in millis
    )

    const onAcknowledgement = (ackChallenge: HalfKeyChallenge) => {
      // Can subscribe to both: per specific message or all message acknowledgments
      this.emit(`hopr:message-acknowledged:${ackChallenge.to_hex()}`)
      this.emit('hopr:message-acknowledged', ackChallenge.to_hex())
    }

    let packetCfg = new PacketInteractionConfig(this.packetKeypair, this.chainKeypair)
    packetCfg.check_unrealized_balance = this.cfg.chain.check_unrealized_balance

    const onReceivedMessage = (msg: ApplicationData) => {
      try {
        this.emit('hopr:message', msg)
      } catch (err) {
        log(`could not deserialize application data: ${err}`)
      }
    }

    log('Linking chain and packet keys')
    this.db.link_chain_and_packet_keys(
      this.chainKeypair.to_address(),
      this.packetKeypair.public(),
      Snapshot.make_default()
    )

    const tbfPath = path.join(this.cfg.db.data, 'tbf')
    let tagBloomFilter = new TagBloomFilter()
    try {
      let tbfData = new Uint8Array(fs.readFileSync(tbfPath))
      tagBloomFilter = TagBloomFilter.deserialize(tbfData)
    } catch (err) {
      error(`no tag bloom filter file found, using empty`)
    }

    let txExecutor = new WasmTxExecutor(
      connector.sendTicketRedeemTx.bind(connector),
      connector.openChannel.bind(connector),
      connector.fundChannel.bind(connector),
      connector.initializeClosure.bind(connector),
      connector.finalizeClosure.bind(connector),
      connector.withdraw.bind(connector)
    )

    log('Constructing the core application and tools')
    let coreApp = new CoreApp(
      new OffchainKeypair(this.packetKeypair.secret()),
      this.chainKeypair,
      this.db.clone(),
      this.cfg.network_options,
      this.cfg.heartbeat,
      ping_cfg,
      onAcknowledgement,
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
      txExecutor,
      this.getLocalMultiaddresses().map((x) => x.toString()),
      this.cfg.protocol.ack,
      this.cfg.protocol.heartbeat,
      this.cfg.protocol.msg,
      this.cfg.protocol.ticket_aggregation,
      this.cfg.strategy
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
        await this.announce(this.cfg.chain.announce)
        this.hasAnnounced = true
      } catch (err) {
        console.error('Could not announce directly self on-chain: ', err)
      }
    } else {
      log('NodeSafeRegistry entry already present, proceeding with Safe-Module announcement')
      try {
        await this.announce(this.cfg.chain.announce, true)
        this.hasAnnounced = true
      } catch (err) {
        console.error('Could not announce through Safe-Module self on-chain: ', err)
      }
    }
    // If the announcement fails we keep going to prevent the node from retrying
    // after restart. Functionality is limited and users must check the logs for
    // errors.

    // Possibly register node-safe pair to NodeSafeRegistry. Following that the
    // connector is set to use safe tx variants.
    try {
      log(`check node-safe registry`)
      await this.registerSafeByNode()
      this.hasNodeSafeRegistered = true
    } catch (err) {
      console.error('Could not register node with safe: ', err)
      // If the node safe registration fails we keep going to prevent the node from retrying
      // after restart. Functionality is limited and users must check the logs for
      // errors.
    }

    // subscribe so we can process channel close events
    connector.indexer.on('own-channel-updated', this.onOwnChannelUpdated.bind(this))

    // subscribe so we can process channel ticket redeemed events
    connector.indexer.on('ticket-redeemed', this.onTicketRedeemed.bind(this))

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

  public isReady(): boolean {
    return this.hasAnnounced && this.hasNodeSafeRegistered
  }

  private getLocalMultiaddresses(): Multiaddr[] {
    let mas: Multiaddr[] = []

    // at this point the values were parsed and validated already
    if (this.cfg.host.is_ipv4()) {
      mas.push(multiaddr(`/ip4/${this.cfg.host.address()}/tcp/${this.cfg.host.port}`))
    } else if (this.cfg.host.is_domain()) {
      mas.push(multiaddr(`/dns4/${this.cfg.host.address()}/tcp/${this.cfg.host.port}`))
    } else {
      new Error('Unknown format specified for host')
    }

    return mas
  }

  /*
   * Callback function used to react to on-chain channel update events.
   * Specifically we trigger the strategy on channel close handler.
   * @param channel object
   */
  private async onOwnChannelUpdated(channel: ChannelEntry): Promise<void> {
    try {
      await this.tools.channel_events().send_event(channel)
    } catch (e) {
      log(`failed to emit channel closure event`)
    }
  }

  /*
   * Callback function used to react to on-chain channel ticket redeem events.
   * Specifically we resolve the pending balance of the ticket.
   * @param channel object
   * @param ticket amount
   */
  private async onTicketRedeemed(channel: ChannelEntry, ticketAmount: Balance): Promise<void> {
    // We are only interested in channels where we are the source, since only
    // then we track the pending balance.
    if (channel.source.eq(this.getEthereumAddress())) {
      await this.db.resolve_pending(channel.destination, ticketAmount)
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
      // safeguard against empty multiaddrs, skip
      if (addr.toString() == '') {
        continue
      }
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

    return addrs.sort(this.cfg.test.prefer_local_addresses ? compareAddressesLocalMode : compareAddressesPublicMode)
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

    return addrs.sort(this.cfg.test.prefer_local_addresses ? compareAddressesLocalMode : compareAddressesPublicMode)
  }

  /**
   * Attempts to all tickets in the given channel.
   * @param channelId id of the channel
   */
  public async aggregateTickets(channelId: Hash) {
    const channel = await this.db.get_channel(channelId)

    // make additional assertions
    if (!channel) {
      throw new Error('Cannot aggregate tickets in non-existing channel')
    }
    if (!(channel.status in [ChannelStatus.Open, ChannelStatus.PendingToClose])) {
      throw new Error('Cannot aggregate tickets in channel when not in status OPEN or PENDING_TO_CLOSE')
    }

    let ticketCount = await this.db.get_acknowledged_tickets_count()
    if (ticketCount < 1) {
      throw new Error('No tickets found in channel')
    }

    await this.tools.aggregate_tickets(channel, TICKET_AGGREGATION_TIMEOUT_MILLISECONDS)
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

  // This now only used to update metrics
  public startPeriodicStrategyCheck() {
    const periodicCheck = async function (this: Hopr) {
      log('periodic check. Current status:', this.status)
      if (this.status != 'RUNNING') {
        return
      }
      const timer = retimer(() => {
        log('tick took longer than 10 secs')
      }, 10000)
      try {
        log('Triggering tick')
        await this.updateChannelMetrics()
      } catch (e) {
        log('error in periodic check', e)
      }
      log('Clearing out logging timeout.')
      timer.clear()
    }.bind(this)

    log(`Starting periodicCheck interval with 60000ms`)
    this.stopPeriodicCheck = intervalTimer(periodicCheck, () => 60000)
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

    return addrs.sort(this.cfg.test.prefer_local_addresses ? compareAddressesLocalMode : compareAddressesPublicMode)
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

      if (this.cfg.test.announce_local_addresses) {
        multiaddrs = multiaddrs.filter((ma) => isMultiaddrLocal(ma))
      } else if (this.cfg.test.prefer_local_addresses) {
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
      const dns4 = multiaddrs.find((ma) => ma.toString().startsWith('/dns4/'))

      // Prefer DNS addresses over IPv4 addresses, if any
      addrToAnnounce = dns4 ?? ip4

      // Submit P2P address if IPv4 or IPv6 address is not routable because link-locale, reserved or private address
      // except if testing locally, e.g. as part of an integration test
      if (addrToAnnounce && addrToAnnounce.toString().length > 0) {
        routableAddressAvailable = true
      }
    }

    // skip if no address to announce has been found
    if (!routableAddressAvailable) {
      log('Error: could not find an address to announce')
      return
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
        // we only log whether announcement is allowed but try either way
        // because an outdated Module contract might not give back that
        // information
        if (isRegisteredCorrectly) {
          log(
            'announcing via Safe-Module on-chain with announcement allowed=%s %s routable address %s',
            isAnnouncementAllowed,
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

  public async getBalance(): Promise<Balance> {
    verbose('Requesting hopr balance for node')
    // we do not keep the node's hopr balance in the db anymore, therefore use
    // the RPC API instead
    // FIXME: remove these functions entirely since the HOPR balance isn't used
    // anymore by the node
    return await HoprCoreEthereum.getInstance().getBalance(false)
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

  public async getSafeAllowance(): Promise<Balance> {
    verbose('Requesting hopr allowance from safe for hopr channels')
    return await this.db.get_staking_safe_allowance()
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
    if (!this.isReady) {
      log('openChannel: Node is not ready for on-chain operations')
    }
    let self_addr = this.getEthereumAddress()
    let amount = new Balance(amountToFund.toString(10), BalanceType.HOPR)
    let tx_sender = this.tools.get_tx_sender()
    try {
      let res = await open_channel(this.db, counterparty, self_addr, amount, tx_sender)
      return { channelId: res.channel_id, receipt: res.tx_hash.to_hex() }
    } catch (err) {
      log('failed to open channel', err)
      this.maybeEmitFundsEmptyEvent(err)
      throw new Error(`Failed to openChannel: ${err}`)
    }
  }

  /**
   * Fund a payment channel
   *
   * @param channelId the id of the channel
   * @param amount the amount to fund the channel
   */
  public async fundChannel(channelId: Hash, amount: BN): Promise<string> {
    if (!this.isReady) {
      log('fundChannel: Node is not ready for on-chain operations')
    }

    try {
      let newAmount = new Balance(amount.toString(10), BalanceType.HOPR)
      let tx_sender = this.tools.get_tx_sender()
      let res = await fund_channel(this.db, channelId, newAmount, tx_sender)
      return res.to_hex()
    } catch (err) {
      log('failed to fund channel', err)
      this.maybeEmitFundsEmptyEvent(err)
      throw new Error(`Failed to fundChannel: ${err}`)
    }
  }

  public async closeChannel(
    counterparty: Address,
    direction: ChannelDirection
  ): Promise<{ receipt: string; status: ChannelStatus }> {
    if (!this.isReady) {
      log('closeChannel: Node is not ready for on-chain operations')
    }

    try {
      let self_addr = this.getEthereumAddress()
      let tx_sender = this.tools.get_tx_sender()
      let res = await close_channel(this.db, counterparty, self_addr, direction, false, tx_sender)
      return { receipt: res.tx_hash.to_hex(), status: res.status }
    } catch (err) {
      log('failed to close channel', err)
      this.maybeEmitFundsEmptyEvent(err)
      throw new Error(`Failed to closeChannel: ${err}`)
    }
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
      neglectedValue: await this.db.get_neglected_tickets_value(),
      rejected: await this.db.get_rejected_tickets_count(),
      rejectedValue: await this.db.get_rejected_tickets_value()
    }
  }

  public async redeemAllTickets(onlyAggregated: boolean = false) {
    if (!this.isReady) {
      log('redeemAllTickets: Node is not ready for on-chain operations')
    }
    try {
      let tx_sender = this.tools.get_tx_sender()
      await redeem_all_tickets(this.db, onlyAggregated, tx_sender)
    } catch (err) {
      log(`error during all tickets redemption: ${err}`)
    }
  }

  public async redeemTicketsInChannel(channelId: Hash, onlyAggregated: boolean = false) {
    if (!this.isReady) {
      log('redeemTicketsInChannel: Node is not ready for on-chain operations')
    }
    try {
      log(`trying to redeem tickets in channel ${channelId.to_hex()}`)
      const channel = await this.db.get_channel(channelId)
      let tx_sender = this.tools.get_tx_sender()
      if (channel?.destination.eq(this.getEthereumAddress())) {
        await redeem_tickets_in_channel(this.db, channel, onlyAggregated, tx_sender)
      } else {
        log(`cannot redeem tickets in channel ${channelId.to_hex()}`)
      }
    } catch (err) {
      log(`error during tickets redemption in channel ${channelId.to_hex()}: ${err}`)
    }
  }

  public async redeemTicketsWithCounterparty(counterparty: Address, onlyAggregated: boolean = false) {
    if (!this.isReady) {
      log('redeemTicketsWithCounterparty: Node is not ready for on-chain operations')
    }

    try {
      let tx_sender = this.tools.get_tx_sender()
      await redeem_tickets_with_counterparty(this.db, counterparty, onlyAggregated, tx_sender)
    } catch (err) {
      log(`error during ticket redemption with counterparty ${counterparty.to_hex()}: ${err}`)
    }
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
    return this.chainKeypair.public().to_address().clone()
  }

  /**
   * Withdraw on-chain assets to a given address
   * @param currency either native currency or HOPR tokens
   * @param recipient the account where the assets should be transferred to
   * @param amount how many tokens to be transferred
   * @returns
   */
  public async withdraw(recipient: Address, amount: Balance): Promise<string> {
    if (!this.isReady) {
      log('withdraw: Node is not ready for on-chain operations')
    }
    try {
      let tx_sender = this.tools.get_tx_sender()
      let result = await withdraw(recipient, amount, tx_sender)
      return result.to_hex()
    } catch (err) {
      this.maybeEmitFundsEmptyEvent(err)
      throw new Error(`Failed to withdraw: ${err}`)
    }
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
              log(`waitforfunds: current balance is ${nativeBalance.to_formatted_string()}`)
              if (nativeBalance.gte(nativeBalance.of_same(MIN_NATIVE_BALANCE.toString(10)))) {
                resolve()
              } else {
                log('waitforfunds: still unfunded, trying again soon')
                reject()
              }
            } catch (e) {
              log('waitforfunds: error with native balance call, trying again soon')
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
export { findPath, PeerOrigin, PeerStatus, Health }
export { resolveNetwork, supportedNetworks, type ResolvedNetwork } from './network.js'
