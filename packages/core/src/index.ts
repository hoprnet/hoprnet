import { setImmediate } from 'timers/promises'
import EventEmitter, { once } from 'events'

import { protocols, Multiaddr } from '@multiformats/multiaddr'

import type BN from 'bn.js'
import { keysPBM } from '@libp2p/crypto/keys'
import { createHash } from 'crypto'
import secp256k1 from 'secp256k1'
import type { Libp2p as Libp2pType } from 'libp2p'
import type { Connection } from '@libp2p/interface-connection'
import type { Peer } from '@libp2p/interface-peer-store'
import type { PeerId } from '@libp2p/interface-peer-id'
import type { Components } from '@libp2p/interfaces/components'
import { compareAddressesLocalMode, compareAddressesPublicMode, type HoprConnectConfig } from '@hoprnet/hopr-connect'

import { PACKET_SIZE, INTERMEDIATE_HOPS, VERSION, FULL_VERSION } from './constants.js'

import AccessControl from './network/access-control.js'
import NetworkPeers, { Entry } from './network/network-peers.js'
import Heartbeat, { NetworkHealthIndicator } from './network/heartbeat.js'

import { findPath } from './path/index.js'

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
  type Ticket,
  createRelayerKey,
  createCircuitAddress,
  convertPubKeyFromPeerId
} from '@hoprnet/hopr-utils'
import HoprCoreEthereum, { type Indexer } from '@hoprnet/hopr-core-ethereum'

import {
  type StrategyTickResult,
  type ChannelStrategyInterface,
  PassiveStrategy,
  PromiscuousStrategy,
  SaneDefaults
} from './channel-strategy.js'

import { subscribeToAcknowledgements } from './interactions/packet/acknowledgement.js'
import { PacketForwardInteraction } from './interactions/packet/forward.js'

import { Packet } from './messages/index.js'
import type { ResolvedEnvironment } from './environment.js'
import { createLibp2pInstance } from './main.js'
import type { EventEmitter as Libp2pEmitter } from '@libp2p/interfaces/events'

const DEBUG_PREFIX = `hopr-core`
const log = debug(DEBUG_PREFIX)
const verbose = debug(DEBUG_PREFIX.concat(`:verbose`))
const error = debug(DEBUG_PREFIX.concat(`:error`))

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
  announce?: boolean
  dataPath: string
  createDbIfNotExist?: boolean
  forceCreateDB?: boolean
  allowLocalConnections?: boolean
  allowPrivateConnections?: boolean
  password?: string
  connector?: HoprCoreEthereum
  strategy?: ChannelStrategyInterface
  hosts?: {
    ip4?: NetOptions
    ip6?: NetOptions
  }
  heartbeatInterval?: number
  heartbeatThreshold?: number
  heartbeatVariance?: number
  networkQualityThreshold?: number
  onChainConfirmations?: number
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
  private strategy: ChannelStrategyInterface
  private networkPeers: NetworkPeers
  private heartbeat: Heartbeat
  private forward: PacketForwardInteraction
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
    private db: HoprDB,
    private connector: HoprCoreEthereum,
    private options: HoprOptions,
    private publicNodesEmitter = new (EventEmitter as new () => HoprConnectConfig['config']['publicNodes'])()
  ) {
    super()

    if (!id.privateKey || !isSecp256k1PeerId(id)) {
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
   * @param __testingLibp2p use simulated libp2p instance for testing
   */
  public async start(__testingLibp2p?: Libp2p) {
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

    // Add all initial public nodes to public nodes cache
    initialNodes.forEach((initialNode) => this.knownPublicNodesCache.add(initialNode.id.toString()))

    // Fetch all nodes that will announces themselves during startup
    const recentlyAnnouncedNodes: PeerStoreAddress[] = []
    const pushToRecentlyAnnouncedNodes = (peer: PeerStoreAddress) => recentlyAnnouncedNodes.push(peer)
    this.connector.indexer.on('peer', pushToRecentlyAnnouncedNodes)

    // Initialize libp2p object and pass configuration
    const libp2p = (await createLibp2pInstance(
      this.id,
      this.options,
      initialNodes,
      this.publicNodesEmitter,
      async (peerId: PeerId, origin: string): Promise<boolean> => {
        return accessControl.reviewConnection(peerId, origin)
      }
    )) as Libp2p

    // Needed to stop libp2p instance
    this.stopLibp2p = libp2p.stop.bind(libp2p)

    this.libp2pComponents = libp2p.components
    // Subscribe to p2p events from libp2p. Wraps our instance of libp2p.
    const subscribe = ((
      protocol: string,
      handler: LibP2PHandlerFunction<Promise<void | Uint8Array>>,
      includeReply: boolean,
      errHandler: (err: any) => void
    ) => libp2pSubscribe(this.libp2pComponents, protocol, handler, errHandler, includeReply)) as Subscribe

    const sendMessage = ((dest: PeerId, protocol: string, msg: Uint8Array, includeReply: boolean, opts: DialOpts) =>
      libp2pSendMessage(this.libp2pComponents, dest, protocol, msg, includeReply, opts)) as SendMessage

    // Attach network health measurement functionality
    const peers: Peer[] = await this.libp2pComponents.getPeerStore().all()
    this.networkPeers = new NetworkPeers(
      peers.map((p) => p.id),
      [this.id],
      this.options.networkQualityThreshold,
      (peer: PeerId) => {
        this.libp2pComponents.getPeerStore().delete(peer)
        this.publicNodesEmitter.emit('removePublicNode', peer)
      }
    )

    // Initialize AccessControl
    const accessControl = new AccessControl(
      this.networkPeers,
      this.isAllowedAccessToNetwork.bind(this),
      this.closeConnectionsTo.bind(this)
    )

    // react when network registry is enabled / disabled
    this.connector.indexer.on('network-registry-status-changed', (_enabled: boolean) => {
      accessControl.reviewConnections()
    })
    // react when an account's eligibility has changed
    this.connector.indexer.on(
      'network-registry-eligibility-changed',
      (_account: Address, node: PublicKey, _eligible: boolean) => {
        const peerId = node.toPeerId()
        const origin = this.networkPeers.has(peerId)
          ? this.networkPeers.getConnectionInfo(peerId).origin
          : 'network registry'
        accessControl.reviewConnection(peerId, origin)
      }
    )

    peers.forEach((peer) => log(`peer store: loaded peer ${peer.id.toString()}`))

    this.heartbeat = new Heartbeat(
      this.networkPeers,
      subscribe,
      sendMessage,
      this.closeConnectionsTo.bind(this),
      accessControl.reviewConnection.bind(accessControl),
      this,
      (peerId: PeerId) => this.knownPublicNodesCache.has(peerId.toString()),
      this.environment.id,
      this.options
    )

    this.libp2pComponents.getConnectionManager().addEventListener('peer:connect', (event: CustomEvent<Connection>) => {
      this.networkPeers.register(event.detail.remotePeer, 'libp2p peer connect')
    })

    const protocolMsg = `/hopr/${this.environment.id}/msg`
    const protocolAck = `/hopr/${this.environment.id}/ack`

    // Attach mixnet functionality
    await subscribeToAcknowledgements(
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
    await this.forward.start()

    // Attach socket listener and check availability of entry nodes
    await libp2p.start()
    log('libp2p started')

    this.connector.indexer.on('peer', this.onPeerAnnouncement.bind(this))

    // Add all entry nodes that were announced during startup
    this.connector.indexer.off('peer', pushToRecentlyAnnouncedNodes)
    for (const announcedNode of recentlyAnnouncedNodes) {
      await setImmediate()
      await this.onPeerAnnouncement(announcedNode)
    }

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
    await this.heartbeat.start()
    this.startPeriodicStrategyCheck()

    this.status = 'RUNNING'

    // Log information
    // Debug log used in e2e integration tests, please don't change
    log('# STARTED NODE')
    log('ID', this.getId().toString())
    log('Protocol version', VERSION)
    if (this.libp2pComponents.getAddressManager().getAddresses() !== undefined) {
      log(`Available under the following addresses:`)
      for (const ma of this.libp2pComponents.getAddressManager().getAddresses()) {
        log(` - ${ma.toString()}`)
      }
    } else {
      log(`No multiaddrs has been registered.`)
    }
    await this.maybeLogProfilingToGCloud()
    this.heartbeat.recalculateNetworkHealth()
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
  private async onPeerAnnouncement(peer: { id: PeerId; multiaddrs: Multiaddr[] }): Promise<void> {
    if (peer.id.equals(this.id)) {
      // Ignore announcements from ourself
      return
    }

    const dialables = peer.multiaddrs.filter((ma: Multiaddr) => {
      const tuples = ma.tuples()
      return tuples.length > 1 && tuples[0][0] != protocols('p2p').code
    })

    try {
      const pubKey = convertPubKeyFromPeerId(peer.id)
      await this.libp2pComponents.getPeerStore().keyBook.set(peer.id, pubKey.bytes)

      if (dialables.length > 0) {
        this.publicNodesEmitter.emit('addPublicNode', { id: peer.id, multiaddrs: dialables })

        await this.libp2pComponents.getPeerStore().addressBook.add(peer.id, dialables)
      }

      // Mark the corresponding entry as public & recalculate network health indicator
      this.knownPublicNodesCache.add(peer.id.toString())
      this.heartbeat.recalculateNetworkHealth()
    } catch (err) {
      log(`Failed to update peer-store with new peer ${peer.id.toString()} info`, err)
    }
  }

  // On the strategy interval, poll the strategy to see what channel changes
  // need to be made.
  private async tickChannelStrategy() {
    verbose('strategy tick', this.status, this.strategy.name)
    if (this.status != 'RUNNING') {
      throw new Error('node is not RUNNING')
    }

    const currentChannels: ChannelEntry[] = (await this.getAllChannels()) ?? []

    verbose(`Channels obtained:`)
    for (const currentChannel of currentChannels) {
      verbose(currentChannel.toString())
    }

    if (currentChannels === undefined) {
      throw new Error('invalid channels retrieved from database')
    }

    for (const channel of currentChannels) {
      this.networkPeers.register(channel.destination.toPeerId(), 'channel strategy tick (existing channel)') // Make sure current channels are 'interesting'
    }

    let balance: Balance
    try {
      balance = await this.getBalance()
    } catch (e) {
      throw new Error('failed to getBalance, aborting tick')
    }
    const tickResult: StrategyTickResult = await this.strategy.tick(
      balance.toBN(),
      currentChannels,
      this.networkPeers,
      this.connector.getRandomOpenChannel.bind(this.connector)
    )

    const closeChannelDestinationsCount = tickResult.toClose.length
    verbose(`strategy wants to close ${closeChannelDestinationsCount} channels`)

    for (const channel of currentChannels) {
      if (
        channel.status == ChannelStatus.PendingToClose &&
        !tickResult.toClose.find((x: typeof tickResult['toClose'][number]) => x.destination.eq(channel.destination))
      ) {
        // attempt to finalize closure
        tickResult.toClose.push({
          destination: channel.destination
        })
      }
    }

    verbose(
      `strategy wants to finalize closure of ${tickResult.toClose.length - closeChannelDestinationsCount} channels`
    )

    for (let i = 0; i < tickResult.toClose.length; i++) {
      const destination = tickResult.toClose[i].destination
      verbose(`closing channel to ${destination.toString()}`)
      try {
        await this.closeChannel(destination.toPeerId(), 'outgoing')
        verbose(`closed channel to ${destination.toString()}`)
        this.emit('hopr:channel:closed', destination.toPeerId())
      } catch (e) {
        log(`error when strategy trying to close channel to ${destination.toString()}`, e)
      }

      if (i + 1 < tickResult.toClose.length) {
        // Give other tasks CPU time to happen
        // Push next loop iteration to end of next event loop iteration
        await setImmediate()
      }
    }

    verbose(`strategy wants to open ${tickResult.toOpen.length} new channels`)

    for (let i = 0; i < tickResult.toOpen.length; i++) {
      const channel = tickResult.toOpen[i]
      this.networkPeers.register(channel.destination.toPeerId(), 'channel strategy tick (new channel)')
      try {
        // Opening channels can fail if we can't establish a connection.
        const hash = await this.openChannel(channel.destination.toPeerId(), channel.stake)
        verbose('- opened', channel, hash)
        this.emit('hopr:channel:opened', channel)
      } catch (e) {
        log(`error when strategy trying to open channel to ${channel.destination.toString()}`, e)
      }

      if (i + 1 < tickResult.toOpen.length) {
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
   * Recalculates and retrieves the current connectivity health indicator.
   */
  public getConnectivityHealth() {
    return this.heartbeat.recalculateNetworkHealth()
  }

  /**
   * Shuts down the node and saves keys and peerBook in the database
   */
  // @TODO make modules Startable
  public async stop(): Promise<void> {
    if (this.status == 'DESTROYED') {
      throw Error(`alreayd destroyed. Cannot destroy twice`)
    }
    this.status = 'DESTROYED'
    verbose('Stopping checking timeout')
    this.stopPeriodicCheck?.()
    verbose('Stopping heartbeat & indexer')
    await this.heartbeat.stop()
    verbose(`Stopping connector`)
    await this.connector.stop()
    verbose('Stopping database')
    await this.db?.close()
    log(`Database closed.`)
    verbose('Stopping libp2p')
    await this.stopLibp2p()
    log(`Libp2p closed.`)

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
          const relayAddress = createCircuitAddress(relayer.id, peer)
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
   * @param msg message to send
   * @param destination PeerId of the destination
   * @param intermediatePath optional set path manually
   */
  public async sendMessage(msg: Uint8Array, destination: PeerId, intermediatePath?: PublicKey[]): Promise<void> {
    if (this.status != 'RUNNING') {
      throw new Error('Cannot send message until the node is running')
    }

    if (msg.length > PACKET_SIZE) {
      throw Error(`Message does not fit into one packet. Please split message into chunks of ${PACKET_SIZE} bytes`)
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

        if (ticketIssuer.eq(ticketReceiver)) log(`WARNING: duplicated adjacent path entries.`)

        const channel = await this.db.getChannelX(ticketIssuer, ticketReceiver)

        if (channel.status !== ChannelStatus.Open) {
          throw Error(`Channel ${channel.getId().toHex()} is not open`)
        }
      }
    } else {
      intermediatePath = await this.getIntermediateNodes(PublicKey.fromPeerId(destination))

      if (intermediatePath == null || !intermediatePath.length) {
        throw Error(`bad path`)
      }
    }

    const path: PublicKey[] = [].concat(intermediatePath, [PublicKey.fromPeerId(destination)])

    let packet: Packet
    try {
      packet = await Packet.create(
        msg,
        path.map((x) => x.toPeerId()),
        this.getId(),
        this.db
      )
    } catch (err) {
      throw Error(`Error while creating packet ${JSON.stringify(err)}`)
    }

    await packet.storePendingAcknowledgement(this.db)

    const acknowledged: Promise<void> = once(this, 'message-acknowledged:' + packet.ackChallenge.toHex()) as any

    try {
      await this.forward.interact(path[0].toPeerId(), packet)
    } catch (err) {
      throw Error(`Error while trying to send final packet ${JSON.stringify(err)}`)
    }

    return acknowledged
  }

  /**
   * Ping a node.
   * @param destination PeerId of the node
   * @returns latency
   */
  public async ping(destination: PeerId): Promise<{ info?: string; latency: number }> {
    let start = Date.now()

    // Propagate any errors thrown upwards
    let pingResult = await this.heartbeat.pingNode(destination)

    if (pingResult.lastSeen >= 0) {
      if (this.networkPeers.has(destination)) {
        this.networkPeers.updateRecord(pingResult)
      } else {
        this.networkPeers.register(destination, 'manual ping')
      }
      return { latency: pingResult.lastSeen - start }
    } else {
      return { info: 'failure', latency: -1 }
    }
  }

  /**
   * @returns a list connected peerIds
   */
  public getConnectedPeers(): PeerId[] {
    if (!this.networkPeers) {
      return []
    }
    return this.networkPeers.all()
  }

  /**
   * Takes a look into the indexer.
   * @returns a list of announced multi addresses
   */
  public async getAddressesAnnouncedOnChain(): Promise<Multiaddr[]> {
    return this.indexer.getAddressesAnnouncedOnChain()
  }

  /**
   * @param peerId of the node we want to get the connection info for
   * @returns various information about the connection
   */
  public getConnectionInfo(peerId: PeerId): Entry {
    return this.networkPeers.getConnectionInfo(peerId)
  }

  /**
   * Closes all open connections to a peer. Used to temporarily or permanently
   * disconnect from a peer.
   * Similar to `libp2p.hangUp` but catching all errors.
   * @param peer PeerId of the peer from whom we want to disconnect
   */
  private async closeConnectionsTo(peer: PeerId): Promise<void> {
    const connections = this.libp2pComponents.getConnectionManager().getConnections(peer)

    for (const conn of connections) {
      try {
        await conn.close()
      } catch (err: any) {
        error(`Error while intentionally closing connection to ${peer.toString()}`, err)
      }
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
    const connected = this.networkPeers.debugLog()
    const announced = await this.connector.indexer.getAddressesAnnouncedOnChain()
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
    const ownAccount = await this.connector.getAccount(this.getEthereumAddress())

    // Do not announce if our last is equal to what we intend to announce
    if (ownAccount?.multiAddr?.equals(addrToAnnounce)) {
      log(`intended address has already been announced, nothing to do`)
      return
    }

    try {
      log(
        'announcing on-chain %s routable address',
        announceRoutableAddress && routableAddressAvailable ? 'with' : 'without'
      )
      const announceTxHash = await this.connector.announce(addrToAnnounce)
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

    this.connector.on('ticket:win', async (ack: AcknowledgedTicket) => {
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
    hoprNetworkRegistryAddress: string
    channelClosureSecs: number
  } {
    return this.connector.smartContractInfo()
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
      return this.connector.openChannel(counterpartyPubKey, new Balance(amountToFund))
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

  public async closeChannel(
    counterparty: PeerId,
    direction: 'incoming' | 'outgoing'
  ): Promise<{ receipt: string; status: ChannelStatus }> {
    const counterpartyPubKey = PublicKey.fromPeerId(counterparty)
    const channel =
      direction === 'outgoing'
        ? await this.db.getChannelX(this.pubKey, counterpartyPubKey)
        : await this.db.getChannelX(counterpartyPubKey, this.pubKey)

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

  public async getAllTickets(): Promise<Ticket[]> {
    return this.db.getAcknowledgedTickets().then((list) => list.map((t) => t.ticket))
  }

  public async getTickets(peerId: PeerId): Promise<Ticket[]> {
    const selfPubKey = PublicKey.fromPeerId(this.getId())
    const counterpartyPubKey = PublicKey.fromPeerId(peerId)
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
      ackTickets.map((a) => a.ticket.amount).reduce((x, y) => x.add(y), Balance.ZERO)

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

  public async redeemTicketsInChannel(peerId: PeerId) {
    const selfPubKey = PublicKey.fromPeerId(this.getId())
    const counterpartyPubKey = PublicKey.fromPeerId(peerId)
    const channel = await this.db.getChannelX(counterpartyPubKey, selfPubKey)
    await this.connector.redeemTicketsInChannel(channel)
  }

  /**
   * Get the channel entry between source and destination node.
   * @param src PeerId
   * @param dest PeerId
   * @returns the channel entry of those two nodes
   */
  public async getChannel(src: PeerId, dest: PeerId): Promise<ChannelEntry> {
    return await this.db.getChannelX(PublicKey.fromPeerId(src), PublicKey.fromPeerId(dest))
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
   * @param id the peer id of the account we want to check if it's allowed access to the network
   * @returns true if allowed access
   */
  public async isAllowedAccessToNetwork(id: PeerId): Promise<boolean> {
    return this.connector.isAllowedAccessToNetwork(PublicKey.fromPeerId(id))
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
              // call connector directly and don't use cache, since this is
              // most likely outdated during node startup
              const nativeBalance = await this.connector.getNativeBalance(false)
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
          minDelay: durations.seconds(1),
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
export * from './constants.js'
export { createHoprNode } from './main.js'
export {
  PassiveStrategy,
  PromiscuousStrategy,
  SaneDefaults,
  findPath,
  type StrategyTickResult,
  NetworkHealthIndicator,
  type ChannelStrategyInterface
}
export { resolveEnvironment, supportedEnvironments, type ResolvedEnvironment } from './environment.js'
export { sampleOptions } from './index.mock.js'
export { CONFIRMATIONS } from '@hoprnet/hopr-core-ethereum'
