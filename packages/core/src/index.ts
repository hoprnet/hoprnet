import { setImmediate } from 'timers/promises'
import { type default as LibP2P, type Connection } from 'libp2p'

import type { HoprConnectConfig } from '@hoprnet/hopr-connect'

import { PACKET_SIZE, INTERMEDIATE_HOPS, VERSION, FULL_VERSION } from './constants'

import NetworkPeers from './network/network-peers'
import Heartbeat, { type HeartbeatPingResult } from './network/heartbeat'
import { findPath } from './path'

import { protocols, Multiaddr } from 'multiaddr'
import chalk from 'chalk'

import type PeerId from 'peer-id'
import {
  PublicKey,
  Balance,
  NativeBalance,
  HoprDB,
  libp2pSubscribe,
  libp2pSendMessage,
  isSecp256k1PeerId,
  ChannelStatus,
  MIN_NATIVE_BALANCE,
  u8aConcat,
  isMultiaddrLocal,
  retryWithBackoff,
  durations,
  isErrorOutOfFunds,
  debug,
  retimer,
  type LibP2PHandlerFunction,
  type AcknowledgedTicket,
  type ChannelEntry,
  type Address,
  type DialOpts,
  type Hash,
  type HalfKeyChallenge,
  multiaddressCompareByClassFunction,
  createRelayerKey
} from '@hoprnet/hopr-utils'
import { type default as HoprCoreEthereum, type Indexer } from '@hoprnet/hopr-core-ethereum'
import type BN from 'bn.js'

import EventEmitter from 'events'
import {
  ChannelStrategy,
  PassiveStrategy,
  PromiscuousStrategy,
  SaneDefaults,
  type ChannelsToOpen,
  type ChannelsToClose
} from './channel-strategy'

import { subscribeToAcknowledgements } from './interactions/packet/acknowledgement'
import { PacketForwardInteraction } from './interactions/packet/forward'

import { Packet } from './messages'
import type { ResolvedEnvironment } from './environment'
import { createLibp2pInstance } from './main'

const DEBUG_PREFIX = `hopr-core`
const log = debug(DEBUG_PREFIX)
const verbose = debug(DEBUG_PREFIX.concat(`:verbose`))
const error = debug(DEBUG_PREFIX.concat(`:error`))

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
  announce?: boolean
  dbPath?: string
  createDbIfNotExist?: boolean
  forceCreateDB?: boolean
  allowLocalConnections?: boolean
  allowPrivateConnections?: boolean
  password?: string
  connector?: HoprCoreEthereum
  strategy?: ChannelStrategy
  hosts?: {
    ip4?: NetOptions
    ip6?: NetOptions
  }
  heartbeatInterval?: number
  heartbeatVariance?: number
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
    // when true, even if a direct WebRTC connection is possible,
    // don't do the upgrade to it to test bidirectional NAT
    // default: false
    noWebRTCUpgrade?: boolean
    // when true, disable usage of UPNP to automatically detect
    // external IP
    // default: false
    noUPNP?: boolean
  }
}

export type NodeStatus = 'UNINITIALIZED' | 'INITIALIZING' | 'RUNNING' | 'DESTROYED'

export type Subscribe = ((
  protocol: string,
  handler: LibP2PHandlerFunction<Promise<void> | void>,
  includeReply: false,
  errHandler: (err: any) => void
) => void) &
  ((
    protocol: string,
    handler: LibP2PHandlerFunction<Promise<Uint8Array>>,
    includeReply: true,
    errHandler: (err: any) => void
  ) => void)

export type SendMessage = ((
  dest: PeerId,
  protocol: string,
  msg: Uint8Array,
  includeReply: false,
  opts: DialOpts
) => Promise<void>) &
  ((dest: PeerId, protocol: string, msg: Uint8Array, includeReply: true, opts: DialOpts) => Promise<Uint8Array[]>)

class Hopr extends EventEmitter {
  public status: NodeStatus = 'UNINITIALIZED'

  private stopPeriodicCheck: (() => void) | undefined
  private strategy: ChannelStrategy
  private networkPeers: NetworkPeers
  private heartbeat: Heartbeat
  private forward: PacketForwardInteraction
  private libp2p: LibP2P
  private pubKey: PublicKey

  public environment: ResolvedEnvironment

  public indexer: Indexer

  /**
   * Create an uninitialized Hopr Node
   *
   * @constructor
   *
   * @param options
   * @param provider
   */
  public constructor(
    private id: PeerId,
    private db: HoprDB,
    private connector: HoprCoreEthereum,
    private options: HoprOptions,
    private publicNodesEmitter: HoprConnectConfig['config']['publicNodes'] = new EventEmitter()
  ) {
    super()

    if (!id.privKey || !isSecp256k1PeerId(id)) {
      throw new Error('Hopr Node must be initialized with an id with a secp256k1 private key')
    }
    this.environment = options.environment
    log(`using environment: ${this.environment.id}`)
    this.indexer = this.connector.indexer // TODO temporary
    this.pubKey = PublicKey.fromPeerId(id)
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
   * @param options
   */
  public async start() {
    this.status = 'INITIALIZING'
    log('Starting hopr node...')

    const balance = await this.connector.getNativeBalance(false)

    verbose(
      `Ethereum account ${this.getEthereumAddress().toHex()} has ${balance.toFormattedString()}. Mininum balance is ${new NativeBalance(
        MIN_NATIVE_BALANCE
      ).toFormattedString()}`
    )

    if (!balance || balance.toBN().lte(MIN_NATIVE_BALANCE)) {
      throw new Error('Cannot start node without a funded wallet')
    }
    log('Node has enough to get started, continuing starting payment channels')
    verbose('Starting HoprEthereum, which will trigger the indexer')
    await this.connector.start()
    verbose('Started HoprEthereum. Waiting for indexer to find connected nodes.')

    // Fetch previous announcements from database
    const initialNodes = await this.connector.waitForPublicNodes()

    // Fetch all nodes that will announces themselves during startup
    const recentlyAnnouncedNodes: PeerStoreAddress[] = []
    const pushToRecentlyAnnouncedNodes = (peer: PeerStoreAddress) => recentlyAnnouncedNodes.push(peer)
    this.connector.on('peer', pushToRecentlyAnnouncedNodes)

    // Initialize libp2p object and pass configuration
    this.libp2p = await createLibp2pInstance(this.id, this.options, initialNodes, this.publicNodesEmitter)

    // Subscribe to p2p events from libp2p. Wraps our instance of libp2p.
    const subscribe = ((
      protocol: string,
      handler: LibP2PHandlerFunction<Promise<void | Uint8Array>>,
      includeReply: boolean,
      errHandler: (err: any) => Promise<void>
    ) => libp2pSubscribe(this.libp2p, protocol, handler, errHandler, includeReply)) as Subscribe

    const sendMessage = ((dest: PeerId, protocol: string, msg: Uint8Array, includeReply: boolean, opts: DialOpts) =>
      libp2pSendMessage(this.libp2p, dest, protocol, msg, includeReply, opts)) as SendMessage
    const hangup = this.libp2p.hangUp.bind(this.libp2p)

    // Attach network health measurement functionality
    this.networkPeers = new NetworkPeers(
      Array.from(this.libp2p.peerStore.peers.values()).map((x) => x.id),
      [this.id],
      (peer: PeerId) => this.publicNodesEmitter.emit('removePublicNode', peer)
    )
    this.heartbeat = new Heartbeat(this.networkPeers, subscribe, sendMessage, hangup, this.environment.id, this.options)

    this.libp2p.connectionManager.on('peer:connect', (conn: Connection) => {
      this.networkPeers.register(conn.remotePeer)
    })

    const protocolMsg = `/hopr/${this.environment.id}/msg`
    const protocolAck = `/hopr/${this.environment.id}/ack`

    // Attach mixnet functionality
    subscribeToAcknowledgements(
      subscribe,
      this.db,
      this.getId(),
      (ackChallenge: HalfKeyChallenge) => {
        this.emit(`message-acknowledged:${ackChallenge.toHex()}`)
      },
      (ack: AcknowledgedTicket) => this.connector.emit('ticket:win', ack),
      // TODO: automatically reinitialize commitments
      () => {},
      protocolAck
    )
    const onMessage = (msg: Uint8Array) => this.emit('hopr:message', msg)
    this.forward = new PacketForwardInteraction(
      subscribe,
      sendMessage,
      this.getId(),
      onMessage,
      this.db,
      protocolMsg,
      protocolAck
    )

    // Attach socket listener and check availability of entry nodes
    await this.libp2p.start()
    log('libp2p started')

    this.connector.indexer.on('peer', this.onPeerAnnouncement.bind(this))

    // Add all entry nodes that were announced during startup
    this.connector.indexer.off('peer', pushToRecentlyAnnouncedNodes)
    recentlyAnnouncedNodes.forEach(this.onPeerAnnouncement.bind(this))

    this.connector.indexer.on('channel-waiting-for-commitment', this.onChannelWaitingForCommitment.bind(this))

    try {
      await this.announce(this.options.announce)
    } catch (err) {
      console.error(`Could not announce self on-chain`)
      console.error(`Observed error:`, err)
      process.exit(1)
    }
    // subscribe so we can process channel close events
    this.connector.indexer.on('own-channel-updated', this.onOwnChannelUpdated.bind(this))

    this.setChannelStrategy(this.options.strategy || new PassiveStrategy())

    log('announcing done, starting heartbeat & strategy interval')
    this.heartbeat.start()
    this.startPeriodicStrategyCheck()

    this.status = 'RUNNING'

    // Log information
    // Debug log used in e2e integration tests, please don't change
    log('# STARTED NODE')
    log('ID', this.getId().toB58String())
    log('Protocol version', VERSION)
    if (this.libp2p.multiaddrs !== undefined) {
      log(`Available under the following addresses:`)
      for (const ma of this.libp2p.multiaddrs) {
        log(` - ${ma.toString()}`)
      }
    } else {
      log(`No multiaddrs has been registered.`)
    }
    this.maybeLogProfilingToGCloud()
  }

  private maybeLogProfilingToGCloud() {
    if (process.env.GCLOUD) {
      try {
        var name = 'hopr_node_' + this.getId().toB58String().slice(-5).toLowerCase()
        require('@google-cloud/profiler')
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
      log(`Found channel ${c.getId().toHex()} to us with unset commitment. Setting commitment`)
      retryWithBackoff(() => this.connector.commitToChannel(c))
    }
  }

  /*
   * Callback function used to react to on-chain channel update events.
   * Specifically we trigger the strategy on channel close handler.
   * @param channel object
   */
  private async onOwnChannelUpdated(channel: ChannelEntry): Promise<void> {
    if (channel.status === ChannelStatus.PendingToClose) {
      await this.strategy.onChannelWillClose(channel, this.connector)
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

    const address = this.getEthereumAddress().toHex()
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
  private onPeerAnnouncement(peer: { id: PeerId; multiaddrs: Multiaddr[] }): void {
    if (peer.id.equals(this.id)) {
      // Ignore announcements from ourself
      return
    }

    // Total hack
    // function cannot throw because it has a catch all
    this.addPeerToDHT(peer.id)

    const dialables = peer.multiaddrs.filter((ma: Multiaddr) => {
      const tuples = ma.tuples()
      return tuples.length > 1 && tuples[0][0] != protocols('p2p').code
    })

    // @ts-ignore wrong type
    this.libp2p.peerStore.keyBook.set(peer.id)

    if (dialables.length > 0) {
      this.publicNodesEmitter.emit('addPublicNode', { id: peer.id, multiaddrs: dialables })

      this.libp2p.peerStore.addressBook.add(peer.id, dialables)
    }
  }

  /**
   * Total hack.
   * Libp2p seems to miss a channel that passes discovered peers
   * to the DHT routing table.
   * @param peer peer to add to DHT routing table
   */
  private async addPeerToDHT(peer: PeerId): Promise<void> {
    try {
      await this.libp2p._dht._wan._routingTable.add(peer)
      await this.libp2p._dht._lan._routingTable.add(peer)

      await this.libp2p._dht._wan._routingTableRefresh.start()
      await this.libp2p._dht._lan._routingTableRefresh.start()

      await this.libp2p._dht._wan.refreshRoutingTable()
      await this.libp2p._dht._lan.refreshRoutingTable()
    } catch (err) {
      // Catch and log all DHT errors, entirely unclear how to handle them
      log(`Failed while populating the DHT routing table`, err)
    }
  }

  // On the strategy interval, poll the strategy to see what channel changes
  // need to be made.
  private async tickChannelStrategy() {
    verbose('strategy tick', this.status, this.strategy.name)
    if (this.status != 'RUNNING') {
      throw new Error('node is not RUNNING')
    }

    const currentChannels: ChannelEntry[] | undefined = await this.getAllChannels()
    verbose('Channels obtained', currentChannels.map((entry) => entry.toString()).join(`\n`))

    if (currentChannels === undefined) {
      throw new Error('invalid channels retrieved from database')
    }

    for (const channel of currentChannels) {
      this.networkPeers.register(channel.destination.toPeerId()) // Make sure current channels are 'interesting'
    }

    let balance: Balance
    try {
      balance = await this.getBalance()
    } catch (e) {
      throw new Error('failed to getBalance, aborting tick')
    }
    const [nextChannelDestinations, closeChannelDestinations]: [ChannelsToOpen[], ChannelsToClose[]] =
      await this.strategy.tick(
        balance.toBN(),
        currentChannels,
        this.networkPeers,
        this.connector.getRandomOpenChannel.bind(this.connector)
      )

    const closeChannelDestinationsCount = closeChannelDestinations.length
    verbose(`strategy wants to close ${closeChannelDestinationsCount} channels`)

    for (const channel of currentChannels) {
      if (channel.status == ChannelStatus.PendingToClose && !closeChannelDestinations.includes(channel.destination)) {
        // attempt to finalize closure
        closeChannelDestinations.push(channel.destination)
      }
    }

    verbose(
      `strategy wants to finalize closure of ${
        closeChannelDestinations.length - closeChannelDestinationsCount
      } channels`
    )

    for (let i = 0; i < closeChannelDestinations.length; i++) {
      const destination = closeChannelDestinations[i]
      verbose(`closing channel to ${destination.toB58String()}`)
      try {
        await this.closeChannel(destination.toPeerId())
        verbose(`closed channel to ${destination.toString()}`)
        this.emit('hopr:channel:closed', destination.toPeerId())
      } catch (e) {
        log(`error when strategy trying to close channel to ${destination.toString()}`, e)
      }

      if (i + 1 < closeChannelDestinations.length) {
        // Give other tasks CPU time to happen
        // Push next loop iteration to end of next event loop iteration
        await setImmediate()
      }
    }

    verbose(`strategy wants to open ${nextChannelDestinations.length} new channels`)

    for (let i = 0; i < nextChannelDestinations.length; i++) {
      const channel = nextChannelDestinations[i]
      this.networkPeers.register(channel[0].toPeerId())
      try {
        // Opening channels can fail if we can't establish a connection.
        const hash = await this.openChannel(channel[0].toPeerId(), channel[1])
        verbose('- opened', channel, hash)
        this.emit('hopr:channel:opened', channel)
      } catch (e) {
        log(`error when strategy trying to open channel to ${channel[0].toString()}`, e)
      }

      if (i + 1 < nextChannelDestinations.length) {
        // Give other tasks CPU time to happen
        // Push next loop iteration to end of next event loop iteration
        await setImmediate()
      }
    }
  }

  private async getAllChannels(): Promise<ChannelEntry[]> {
    return this.db.getChannelsFrom(PublicKey.fromPeerId(this.getId()).toAddress())
  }

  /**
   * Returns the version of hopr-core.
   */
  public getVersion() {
    return FULL_VERSION
  }

  /**
   * Shuts down the node and saves keys and peerBook in the database
   */
  public async stop(): Promise<void> {
    this.status = 'DESTROYED'
    verbose('Stopping checking timeout')
    this.stopPeriodicCheck?.()
    verbose('Stopping heartbeat & indexer')
    await Promise.all([this.heartbeat.stop(), this.connector.stop()])
    verbose('Stopping database & libp2p')
    await Promise.all([
      this.db?.close().then(() => log(`Database closed.`)),
      this.libp2p.stop().then(() => log(`Libp2p closed.`))
    ])

    // Give the operating system some extra time to close the sockets
    await new Promise((resolve) => setTimeout(resolve, 100))
  }

  public getId(): PeerId {
    return this.id
  }

  /**
   * List of addresses that is announced to other nodes
   * @dev returned list can change at runtime
   * @param peer peer to query for, default self
   * @param timeout [optional] custom timeout for DHT query
   */
  public async getAnnouncedAddresses(peer: PeerId = this.getId(), timeout = 5e3): Promise<Multiaddr[]> {
    if (peer.equals(this.getId())) {
      return this.libp2p.multiaddrs
    }

    const knownAddresses = this.libp2p.peerStore.get(peer)?.addresses?.map((addr) => addr.multiaddr) ?? []

    try {
      for await (const relayer of this.libp2p.contentRouting.findProviders(await createRelayerKey(peer), {
        timeout
      })) {
        const relayAddress = new Multiaddr(`/p2p/${relayer.id.toB58String()}/p2p-circuit/p2p/${peer.toB58String()}`)
        if (knownAddresses.findIndex((ma) => ma.equals(relayAddress)) < 0) {
          knownAddresses.push(relayAddress)
        }
      }
    } catch (err) {
      log(`Could not find any relayer key for ${peer.toB58String()}`)
    }

    return knownAddresses
  }

  /**
   * List the addresses on which the node is listening
   */
  public getListeningAddresses(): Multiaddr[] {
    return this.libp2p.addressManager.getListenAddrs()
  }

  /**
   * Gets the observed addresses of a given peer.
   * @param peer peer to query for
   */
  public getObservedAddresses(peer: PeerId): Multiaddr[] {
    return (this.libp2p.peerStore.get(peer)?.addresses ?? []).map((addr) => addr.multiaddr)
  }

  /**
   * @param msg message to send
   * @param destination PeerId of the destination
   * @param intermediateNodes optional set path manually
   */
  public async sendMessage(msg: Uint8Array, destination: PeerId, intermediatePath?: PublicKey[]): Promise<void> {
    const promises: Promise<void>[] = []

    if (this.status != 'RUNNING') {
      throw new Error('Cannot send message until the node is running')
    }

    if (intermediatePath != undefined) {
      // checking if path makes sense
      for (let i = 0; i < intermediatePath.length; i++) {
        let ticketIssuer: PublicKey
        let ticketReceiver: PublicKey

        if (i == 0) {
          ticketIssuer = PublicKey.fromPeerId(this.getId())
          ticketReceiver = intermediatePath[0]
        } else {
          ticketIssuer = intermediatePath[i - 1]
          ticketReceiver = intermediatePath[i]
        }

        const channel = await this.db.getChannelX(ticketIssuer, ticketReceiver)

        if (channel.status !== ChannelStatus.Open) {
          throw Error(`Channel ${channel.getId().toHex()} is not open`)
        }
      }
    }

    for (let n = 0; n < msg.length / PACKET_SIZE; n++) {
      promises.push(
        new Promise<void>(async (resolve, reject) => {
          if (intermediatePath == undefined) {
            try {
              intermediatePath = await this.getIntermediateNodes(PublicKey.fromPeerId(destination))
            } catch (e) {
              reject(e)
              return
            }
            if (!intermediatePath || !intermediatePath.length) {
              reject(new Error('bad path'))
            }
          }

          const path: PublicKey[] = [].concat(intermediatePath, [PublicKey.fromPeerId(destination)])
          let packet: Packet
          try {
            packet = await Packet.create(
              msg.slice(n * PACKET_SIZE, Math.min(msg.length, (n + 1) * PACKET_SIZE)),
              path.map((x) => x.toPeerId()),
              this.getId(),
              this.db
            )
          } catch (err) {
            return reject(err)
          }

          await packet.storePendingAcknowledgement(this.db)

          this.once('message-acknowledged:' + packet.ackChallenge.toHex(), () => {
            resolve()
          })

          try {
            await this.forward.interact(path[0].toPeerId(), packet)
          } catch (err) {
            return reject(err)
          }
        })
      )
    }

    try {
      await Promise.all(promises)
    } catch (err) {
      log(`Could not send message. Error was: ${chalk.red(err.message)}`)
      throw err
    }
  }

  /**
   * Ping a node.
   * @param destination PeerId of the node
   * @returns latency
   */
  public async ping(destination: PeerId): Promise<{ info?: string; latency: number }> {
    let start = Date.now()

    let pingResult: HeartbeatPingResult
    try {
      pingResult = await this.heartbeat.pingNode(destination)
    } catch (err) {
      log(`Could not ping ${destination.toB58String()}.`, err)
      return { latency: -1, info: 'error' }
    }

    if (pingResult.lastSeen >= 0) {
      if (this.networkPeers.has(destination)) {
        this.networkPeers.updateRecord(pingResult)
      } else {
        this.networkPeers.register(destination)
      }
      return { latency: pingResult.lastSeen - start }
    } else {
      return { info: 'failure', latency: -1 }
    }
  }

  public getConnectedPeers(): PeerId[] {
    if (!this.networkPeers) {
      return []
    }
    return this.networkPeers.all()
  }

  public async connectionReport(): Promise<string> {
    if (!this.networkPeers) {
      return 'Node has not started yet'
    }
    const connected = this.networkPeers.debugLog()
    const announced = await this.connector.indexer.getAnnouncedAddresses()
    return `${connected}
    \n${announced.length} peers have announced themselves on chain:
    \n${announced.map((ma: Multiaddr) => ma.toString()).join('\n')}`
  }

  public subscribeOnConnector(event: string, callback: () => void): void {
    this.connector.on(event, callback)
  }
  public emitOnConnector(event: string): void {
    this.connector.emit(event)
  }

  public startPeriodicStrategyCheck() {
    const periodicCheck = async function (this: Hopr) {
      log('periodic check. Current status:', this.status)
      if (this.status != 'RUNNING') {
        return
      }
      const logTimeout = setTimeout(() => {
        log('strategy tick took longer than 10 secs')
      }, 10000)
      try {
        log('Triggering tick channel strategy')
        await this.tickChannelStrategy()
      } catch (e) {
        log('error in periodic check', e)
      }
      log('Clearing out logging timeout.')
      clearTimeout(logTimeout)
      log(`Setting up timeout for ${this.strategy.tickInterval}ms`)
    }.bind(this)

    log(`Starting periodicCheck interval with ${this.strategy.tickInterval}ms`)

    this.stopPeriodicCheck = retimer(periodicCheck, () => this.strategy.tickInterval)
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

    if (announceRoutableAddress) {
      let multiaddrs = await this.getAnnouncedAddresses()

      if (this.options.testing?.announceLocalAddresses) {
        multiaddrs = multiaddrs.filter((ma) => isMultiaddrLocal(ma))
      } else if (this.options.testing?.preferLocalAddresses) {
        // If we need local addresses, sort them first according to their class
        multiaddrs.sort(multiaddressCompareByClassFunction)
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
        addrToAnnounce = new Multiaddr('/p2p/' + this.getId().toB58String())
      } else {
        routableAddressAvailable = true
      }
    } else {
      addrToAnnounce = new Multiaddr('/p2p/' + this.getId().toB58String())
    }

    // Check if there was a previous annoucement from us
    const ownAccount = await this.connector.getAccount(this.getEthereumAddress())

    // Do not announce if our last is equal to what we intend to announce
    if (ownAccount?.multiAddr?.equals(addrToAnnounce)) {
      log(`intended address has already been announced, nothing to do`)
      return
    }

    try {
      log(
        `announcing address ${addrToAnnounce.toString()} ${
          announceRoutableAddress && routableAddressAvailable ? 'with' : 'without'
        } routing`
      )
      const announceTxHash = await this.connector.announce(addrToAnnounce)
      log(`announcing address ${addrToAnnounce.toString()} done in tx ${announceTxHash}`)
    } catch (err) {
      log(`announcing address ${addrToAnnounce.toString()} failed`)
      this.maybeEmitFundsEmptyEvent(err)
      throw new Error(`Failed to announce address ${addrToAnnounce.toString()}: ${err}`)
    }
  }

  public setChannelStrategy(strategy: ChannelStrategy): void {
    log('setting channel strategy from', this.strategy?.name, 'to', strategy.name)
    this.strategy = strategy

    this.connector.on('ticket:win', async (ack) => {
      try {
        await this.strategy.onWinningTicket(ack, this.connector)
      } catch (err) {
        error(`Strategy error while handling winning ticket`, err)
      }
    })
  }

  public getChannelStrategy(): string {
    return this.strategy.name
  }

  public async getBalance(): Promise<Balance> {
    return await this.connector.getBalance(true)
  }

  public async getNativeBalance(): Promise<NativeBalance> {
    verbose('Requesting native balance from node.')
    return await this.connector.getNativeBalance(true)
  }

  public smartContractInfo(): {
    network: string
    hoprTokenAddress: string
    hoprChannelsAddress: string
    channelClosureSecs: number
  } {
    return this.connector.smartContractInfo()
  }

  /**
   * Open a payment channel
   *
   * @param counterparty the counter party's peerId
   * @param amountToFund the amount to fund in HOPR(wei)
   */
  public async openChannel(
    counterparty: PeerId,
    amountToFund: BN
  ): Promise<{
    channelId: Hash
  }> {
    const counterpartyPubKey = PublicKey.fromPeerId(counterparty)
    const myAvailableTokens = await this.connector.getBalance(true)

    // validate 'amountToFund'
    if (amountToFund.lten(0)) {
      throw Error(`Invalid 'amountToFund' provided: ${amountToFund.toString(10)}`)
    } else if (amountToFund.gt(myAvailableTokens.toBN())) {
      throw Error(
        `You don't have enough tokens: ${amountToFund.toString(10)}<${myAvailableTokens
          .toBN()
          .toString(10)} at address ${this.pubKey.toAddress().toHex()}`
      )
    }

    try {
      return {
        channelId: await this.connector.openChannel(counterpartyPubKey, new Balance(amountToFund))
      }
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
  public async fundChannel(counterparty: PeerId, myFund: BN, counterpartyFund: BN): Promise<void> {
    const counterpartyPubKey = PublicKey.fromPeerId(counterparty)
    const myBalance = await this.connector.getBalance(false)
    const totalFund = myFund.add(counterpartyFund)

    // validate 'amountToFund'
    if (totalFund.lten(0)) {
      throw Error(`Invalid 'totalFund' provided: ${totalFund.toString(10)}`)
    } else if (totalFund.gt(myBalance.toBN())) {
      throw Error(
        `You don't have enough tokens: ${totalFund.toString(10)}<${myBalance
          .toBN()
          .toString(10)} at address ${this.pubKey.toAddress().toHex()}`
      )
    }

    try {
      await this.connector.fundChannel(counterpartyPubKey, new Balance(myFund), new Balance(counterpartyFund))
    } catch (err) {
      this.maybeEmitFundsEmptyEvent(err)
      throw new Error(`Failed to fundChannel: ${err}`)
    }
  }

  public async closeChannel(counterparty: PeerId): Promise<{ receipt: string; status: ChannelStatus }> {
    const counterpartyPubKey = PublicKey.fromPeerId(counterparty)
    const channel = await this.db.getChannelX(this.pubKey, counterpartyPubKey)

    // TODO: should we wait for confirmation?
    if (channel.status === ChannelStatus.Closed) {
      throw new Error('Channel is already closed')
    }

    if (channel.status === ChannelStatus.Open) {
      await this.strategy.onChannelWillClose(channel, this.connector)
    }

    let txHash: string
    try {
      if (channel.status === ChannelStatus.Open || channel.status == ChannelStatus.WaitingForCommitment) {
        log('initiating closure of channel', channel.getId().toHex())
        txHash = await this.connector.initializeClosure(counterpartyPubKey)
      } else {
        // verify that we passed the closure waiting period to prevent failing
        // on-chain transactions

        if (channel.closureTimePassed()) {
          log('finalizing closure of channel', channel.getId().toHex())
          txHash = await this.connector.finalizeClosure(counterpartyPubKey)
        } else {
          log(
            `ignoring finalizing closure of channel ${channel
              .getId()
              .toHex()} because closure window is still active. Need to wait ${channel
              .getRemainingClosureTime()
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

  public async getTicketStatistics() {
    const ack = await this.db.getAcknowledgedTickets()
    const pending = await this.db.getPendingTicketCount()
    const losing = await this.db.getLosingTicketCount()
    const totalValue = (ackTickets: AcknowledgedTicket[]): Balance =>
      ackTickets.map((a) => a.ticket.amount).reduce((x, y) => x.add(y), Balance.ZERO())

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
    await this.connector.redeemAllTickets()
  }

  public async getChannelsFrom(addr: Address): Promise<ChannelEntry[]> {
    return await this.db.getChannelsFrom(addr)
  }

  public async getChannelsTo(addr: Address): Promise<ChannelEntry[]> {
    return await this.db.getChannelsTo(addr)
  }

  public async getPublicKeyOf(addr: Address): Promise<PublicKey> {
    return await this.connector.getPublicKeyOf(addr)
  }

  // NB: The prefix "HOPR Signed Message: " is added as a security precaution.
  // Without it, the node could be convinced to sign a message like an Ethereum
  // transaction draining it's connected wallet funds, since they share the key.
  public async signMessage(message: Uint8Array) {
    return await this.id.privKey.sign(u8aConcat(new TextEncoder().encode('HOPR Signed Message: '), message))
  }

  public getEthereumAddress(): Address {
    return this.connector.getPublicKey().toAddress()
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
      result = await this.connector.withdraw(currency, recipient, amount)
    } catch (err) {
      this.maybeEmitFundsEmptyEvent(err)
      throw new Error(`Failed to withdraw: ${err}`)
    }

    return result
  }

  /**
   * Takes a destination and samples randomly intermediate nodes
   * that will relay that message before it reaches its destination.
   *
   * @param destination instance of peerInfo that contains the peerId of the destination
   */
  private async getIntermediateNodes(destination: PublicKey): Promise<PublicKey[]> {
    return await findPath(
      PublicKey.fromPeerId(this.getId()),
      destination,
      INTERMEDIATE_HOPS,
      (p: PublicKey) => this.networkPeers.qualityOf(p.toPeerId()),
      this.connector.getOpenChannelsFrom.bind(this.connector)
    )
  }

  /**
   * This is a utility method to wait until the node is funded.
   * A backoff is implemented, if node has not been funded and
   * MAX_DELAY is reached, this function will reject.
   */
  public async waitForFunds(): Promise<void> {
    try {
      return retryWithBackoff(
        () => {
          return new Promise<void>(async (resolve, reject) => {
            try {
              const nativeBalance = await this.getNativeBalance()
              if (nativeBalance.toBN().gte(MIN_NATIVE_BALANCE)) {
                resolve()
              } else {
                log('still unfunded, trying again soon')
                reject()
              }
            } catch (e) {
              log('error with native balance call, trying again soon')
              reject()
            }
          })
        },
        {
          minDelay: durations.seconds(30),
          maxDelay: durations.seconds(200),
          delayMultiple: 1.05
        }
      )
    } catch {
      log(`unfunded for more than 200 seconds, shutting down`)
      await this.stop()
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
export * from './constants'
export { createHoprNode } from './main'
export { PassiveStrategy, PromiscuousStrategy, SaneDefaults, findPath }
export type { ChannelsToOpen, ChannelsToClose }
export { resolveEnvironment, supportedEnvironments, type ResolvedEnvironment } from './environment'
export { createLibp2pMock } from './libp2p.mock'
export { sampleOptions } from './index.mock'
