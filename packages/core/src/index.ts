import { type default as LibP2P, type Connection } from 'libp2p'

import type { HoprConnectOptions } from '@hoprnet/hopr-connect'

import { PACKET_SIZE, INTERMEDIATE_HOPS, VERSION, FULL_VERSION } from './constants'

import NetworkPeers from './network/network-peers'
import Heartbeat from './network/heartbeat'
import { findPath } from './path'

import { protocols, Multiaddr } from 'multiaddr'
import chalk from 'chalk'

import type PeerId from 'peer-id'
import {
  PublicKey,
  Balance,
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
  type LibP2PHandlerFunction,
  type AcknowledgedTicket,
  type ChannelEntry,
  type NativeBalance,
  type Address,
  type DialOpts,
  type Hash,
  type HalfKeyChallenge
} from '@hoprnet/hopr-utils'
import { type default as HoprCoreEthereum, type Indexer } from '@hoprnet/hopr-core-ethereum'
import type BN from 'bn.js'

import EventEmitter from 'events'
import {
  ChannelStrategy,
  PassiveStrategy,
  PromiscuousStrategy,
  SaneDefaults,
  ChannelsToOpen,
  ChannelsToClose
} from './channel-strategy'

import { subscribeToAcknowledgements } from './interactions/packet/acknowledgement'
import { PacketForwardInteraction } from './interactions/packet/forward'

import { Packet } from './messages'
import type { ResolvedEnvironment } from './environment'
import { createLibp2pInstance } from './main'

const log = debug(`hopr-core`)
const verbose = debug('hopr-core:verbose')

interface NetOptions {
  ip: string
  port: number
}

type PeerStoreAddress = {
  id: PeerId
  multiaddrs: Multiaddr[]
}

export class HoprOptions {
  constructor(
    public environment: ResolvedEnvironment,
    public announce?: boolean,
    public dbPath?: string,
    public createDbIfNotExist?: boolean,
    public forceCreateDB?: boolean,
    public password?: string,
    public connector?: HoprCoreEthereum,
    public strategy?: ChannelStrategy,
    public hosts?: {
      ip4?: NetOptions
      ip6?: NetOptions
    },
    // You almost certainly want this to be false, this is so we can test with
    // local testnets, and announce 127.0.0.1 addresses.
    public announceLocalAddresses?: boolean,
    // when true, addresses will be sorted local first
    // when false, addresses will be sorted public first
    public preferLocalAddresses?: boolean
  ) {}
}

export type NodeStatus = 'UNINITIALIZED' | 'INITIALIZING' | 'RUNNING' | 'DESTROYED'

export type Subscribe = ((
  protocol: string,
  handler: LibP2PHandlerFunction<Promise<void>>,
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

  private checkTimeout: NodeJS.Timeout
  private strategy: ChannelStrategy
  private networkPeers: NetworkPeers
  private heartbeat: Heartbeat
  private forward: PacketForwardInteraction
  private libp2p: LibP2P
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
    private publicNodesEmitter: HoprConnectOptions['publicNodes'] = new EventEmitter()
  ) {
    super()

    if (!id.privKey || !isSecp256k1PeerId(id)) {
      throw new Error('Hopr Node must be initialized with an id with a secp256k1 private key')
    }
    this.environment = options.environment
    log(`using environment: ${this.environment.id}`)
    this.indexer = this.connector.indexer // TODO temporary
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
    const nativeCurrency =
      require('../protocol-config.json')['networks'][this.environment.network.id]['native_token_name']
    verbose(
      `Ethereum account ${this.getEthereumAddress().toHex()} has ${balance.toBN()} ${nativeCurrency}. Mininum balance is ${MIN_NATIVE_BALANCE} ${nativeCurrency}`
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
      errHandler: (err: any) => void
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
    this.heartbeat = new Heartbeat(this.networkPeers, subscribe, sendMessage, hangup, this.environment.id)

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

    await this.announce(this.options.announce)

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
        log(ma.toString())
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

  private async onChannelWaitingForCommitment(c: ChannelEntry) {
    if (this.strategy.shouldCommitToChannel(c)) {
      log(`Found channel ${c.getId().toHex()} to us with unset commitment. Setting commitment`)
      retryWithBackoff(() => this.connector.commitToChannel(c))
    }
  }

  /**
   * If error provided is considered an out of funds error
   * - it will emit that the node is out of funds
   * @param error error thrown by an ethereum transaction
   */
  private isOutOfFunds(error: any): void {
    const isOutOfFunds = isErrorOutOfFunds(error)
    if (!isOutOfFunds) return

    const address = this.getEthereumAddress().toHex()

    if (isOutOfFunds === 'NATIVE') {
      log('unfunded node', address)
      this.emit('hopr:warning:unfundedNative', address)
    } else if (isOutOfFunds === 'HOPR') {
      log('unfunded node', address)
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
      return tuples.length > 1 && tuples[0][0] != protocols.names['p2p'].code
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
      return
    }

    const currentChannels: ChannelEntry[] | undefined = await this.getAllChannels()
    verbose('Channels obtained', currentChannels)

    if (currentChannels === undefined) {
      log('invalid channels retrieved from database')
      return
    }

    for (const channel of currentChannels) {
      this.networkPeers.register(channel.destination.toPeerId()) // Make sure current channels are 'interesting'
    }

    let balance: Balance
    try {
      balance = await this.getBalance()
    } catch (e) {
      log('failed to getBalance, aborting tick')
      return
    }
    const [nextChannels, closeChannels] = await this.strategy.tick(
      balance.toBN(),
      currentChannels,
      this.networkPeers,
      this.connector.getRandomOpenChannel.bind(this.connector)
    )
    verbose(`strategy wants to close ${closeChannels.length} channels`)

    for (let channel of currentChannels) {
      if (channel.status == ChannelStatus.PendingToClose) {
        // attempt to finalize closure
        closeChannels.push(channel.destination)
      }
    }

    for (let toClose of closeChannels) {
      verbose(`closing ${toClose}`)
      try {
        await this.closeChannel(toClose.toPeerId())
        verbose(`closed channel to ${toClose.toString()}`)
        this.emit('hopr:channel:closed', toClose.toPeerId())
      } catch (e) {
        log('error when trying to close strategy channels', e)
      }
    }
    verbose(`strategy wants to open`, nextChannels.length, 'new channels')
    for (let channelToOpen of nextChannels) {
      this.networkPeers.register(channelToOpen[0].toPeerId())
      try {
        // Opening channels can fail if we can't establish a connection.
        const hash = await this.openChannel(channelToOpen[0].toPeerId(), channelToOpen[1])
        verbose('- opened', channelToOpen, hash)
        this.emit('hopr:channel:opened', channelToOpen)
      } catch (e) {
        log('error when trying to open strategy channels', e)
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
    clearTimeout(this.checkTimeout)
    verbose('Stopping heartbeat & indexer')
    await Promise.all([this.heartbeat.stop(), this.connector.stop()])
    verbose('Stopping database & libp2p', this.db)
    await Promise.all([this.db?.close().then(() => log(`Database closed.`)), this.libp2p.stop()])

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
   */
  public async getAnnouncedAddresses(peer: PeerId = this.getId()): Promise<Multiaddr[]> {
    if (peer.equals(this.getId())) {
      const addrs = this.libp2p.multiaddrs

      // Most of the time we want to only return 'public' addresses, that is,
      // addresses that have a good chance of being reachable by other nodes,
      // therefore only ones that include a public IP etc.
      //
      // We also have a setting announceLocalAddresses that inverts this so we
      // can test on closed local networks.
      if (this.options.announceLocalAddresses) {
        return addrs.filter((ma) => ma.toString().includes('127.0.0.1')) // TODO - proper filtering
      }
      return addrs.filter((ma) => !ma.toString().includes('127.0.0.1')) // TODO - proper filtering
    }

    let dhtResult: Awaited<ReturnType<LibP2P['peerRouting']['findPeer']>>

    try {
      dhtResult = await this.libp2p.peerRouting.findPeer(peer)
    } catch (err) {
      log(`Cannot find any announced addresses for peer ${peer.toB58String()} in the DHT.`)
      return []
    }

    return dhtResult.multiaddrs
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
    try {
      const success = await this.heartbeat.pingNode(destination)
      if (success) {
        return { latency: Date.now() - start }
      } else {
        return { info: 'failure', latency: -1 }
      }
    } catch (e) {
      log(e)
      return { latency: -1, info: 'error' }
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
      log('periodic check', this.status)
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
      this.checkTimeout = setTimeout(periodicCheck, this.strategy.tickInterval)
    }.bind(this)

    log(`Starting periodicCheck interval with ${this.strategy.tickInterval}ms`)

    this.checkTimeout = setTimeout(periodicCheck, this.strategy.tickInterval)
  }

  /**
   * Announces address of node on-chain to be reachable by other nodes.
   * @dev Promise resolves before own announcment appears in the indexer
   * @param includeRouting publish routable address if true
   * @returns Promise that resolves once announce transaction has been published
   */
  private async announce(includeRouting = false): Promise<void> {
    let isRoutableAddress = false
    let addrToAnnounce: Multiaddr

    if (includeRouting) {
      const multiaddrs = await this.getAnnouncedAddresses()

      const ip4 = multiaddrs.find((s) => s.toString().startsWith('/ip4/'))
      const ip6 = multiaddrs.find((s) => s.toString().startsWith('/ip6/'))

      // Prefer IPv4 addresses over IPv6 addresses, if any
      addrToAnnounce = ip4 ?? ip6

      // Submit P2P address if IPv4 or IPv6 address is not routable because link-locale, reserved or private address
      // except if testing locally, e.g. as part of an integration test
      if (addrToAnnounce == undefined || (isMultiaddrLocal(addrToAnnounce) && !this.options.preferLocalAddresses)) {
        addrToAnnounce = new Multiaddr('/p2p/' + this.getId().toB58String())
      } else {
        isRoutableAddress = true
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
      log(`announcing ${includeRouting && isRoutableAddress ? 'with' : 'without'} routing`)

      await this.connector.announce(addrToAnnounce)
    } catch (err) {
      log('announce failed')
      await this.isOutOfFunds(err)
      throw new Error(`Failed to announce: ${err}`)
    }
  }

  public setChannelStrategy(strategy: ChannelStrategy): void {
    log('setting channel strategy from', this.strategy?.name, 'to', strategy.name)
    this.strategy = strategy

    this.connector.on('ticket:win', (ack) => {
      this.strategy.onWinningTicket(ack, this.connector)
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
    const selfPubKey = new PublicKey(this.getId().pubKey.marshal())
    const counterpartyPubKey = new PublicKey(counterparty.pubKey.marshal())
    const myAvailableTokens = await this.connector.getBalance(true)

    // validate 'amountToFund'
    if (amountToFund.lten(0)) {
      throw Error(`Invalid 'amountToFund' provided: ${amountToFund.toString(10)}`)
    } else if (amountToFund.gt(myAvailableTokens.toBN())) {
      throw Error(
        `You don't have enough tokens: ${amountToFund.toString(10)}<${myAvailableTokens
          .toBN()
          .toString(10)} at address ${selfPubKey.toAddress().toHex()}`
      )
    }

    try {
      return {
        channelId: await this.connector.openChannel(counterpartyPubKey, new Balance(amountToFund))
      }
    } catch (err) {
      await this.isOutOfFunds(err)
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
    const selfPubKey = new PublicKey(this.getId().pubKey.marshal())
    const counterpartyPubKey = new PublicKey(counterparty.pubKey.marshal())
    const myBalance = await this.connector.getBalance(false)
    const totalFund = myFund.add(counterpartyFund)

    // validate 'amountToFund'
    if (totalFund.lten(0)) {
      throw Error(`Invalid 'totalFund' provided: ${totalFund.toString(10)}`)
    } else if (totalFund.gt(myBalance.toBN())) {
      throw Error(
        `You don't have enough tokens: ${totalFund.toString(10)}<${myBalance
          .toBN()
          .toString(10)} at address ${selfPubKey.toAddress().toHex()}`
      )
    }

    try {
      await this.connector.fundChannel(counterpartyPubKey, new Balance(myFund), new Balance(counterpartyFund))
    } catch (err) {
      await this.isOutOfFunds(err)
      throw new Error(`Failed to fundChannel: ${err}`)
    }
  }

  public async closeChannel(counterparty: PeerId): Promise<{ receipt: string; status: ChannelStatus }> {
    const selfPubKey = new PublicKey(this.getId().pubKey.marshal())
    const counterpartyPubKey = new PublicKey(counterparty.pubKey.marshal())
    const channel = await this.db.getChannelX(selfPubKey, counterpartyPubKey)

    // TODO: should we wait for confirmation?
    if (channel.status === ChannelStatus.Closed) {
      throw new Error('Channel is already closed')
    }

    if (channel.status === ChannelStatus.Open) {
      await this.strategy.onChannelWillClose(channel, this.connector)
    }

    log('closing channel', channel.getId())
    let txHash: string
    try {
      if (channel.status === ChannelStatus.Open || channel.status == ChannelStatus.WaitingForCommitment) {
        log('initiating closure')
        txHash = await this.connector.initializeClosure(counterpartyPubKey)
      } else {
        log('finalizing closure')
        txHash = await this.connector.finalizeClosure(counterpartyPubKey)
      }
    } catch (err) {
      log('failed to close channel', err)
      await this.isOutOfFunds(err)
      throw new Error(`Failed to closeChannel: ${err}`)
    }

    log(`closed channel, ${channel.getId()}`)
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
      await this.isOutOfFunds(err)
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
export { resolveEnvironment, supportedEnvironments, ResolvedEnvironment } from './environment'
export { libp2pMock } from './libp2p.mock'
export { sampleOptions } from './index.mock'
