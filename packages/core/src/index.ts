import LibP2P from 'libp2p'
import type { Connection } from 'libp2p'

import MPLEX from 'libp2p-mplex'
import KadDHT from 'libp2p-kad-dht'
import { NOISE } from '@chainsafe/libp2p-noise'

const { HoprConnect } = require('@hoprnet/hopr-connect')
import type { HoprConnectOptions } from '@hoprnet/hopr-connect'

import { PACKET_SIZE, INTERMEDIATE_HOPS, VERSION, FULL_VERSION } from './constants'

import NetworkPeers from './network/network-peers'
import Heartbeat from './network/heartbeat'
import { findPath } from './path'

import { protocols, Multiaddr } from 'multiaddr'
import chalk from 'chalk'

import PeerId from 'peer-id'
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
  isMultiaddrLocal
} from '@hoprnet/hopr-utils'
import type {
  LibP2PHandlerFunction,
  AcknowledgedTicket,
  ChannelEntry,
  NativeBalance,
  Address,
  DialOpts,
  Hash,
  HalfKeyChallenge
} from '@hoprnet/hopr-utils'
import HoprCoreEthereum from '@hoprnet/hopr-core-ethereum'
import type { Indexer } from '@hoprnet/hopr-core-ethereum'
import BN from 'bn.js'
import { getAddrs } from './identity'

import EventEmitter from 'events'
import {
  ChannelStrategy,
  PassiveStrategy,
  PromiscuousStrategy,
  SaneDefaults,
  ChannelsToOpen,
  ChannelsToClose
} from './channel-strategy'
import { debug } from '@hoprnet/hopr-utils'

import { subscribeToAcknowledgements } from './interactions/packet/acknowledgement'
import { PacketForwardInteraction } from './interactions/packet/forward'

import { Packet } from './messages'
import { localAddressesFirst, AddressSorter, retryWithBackoff, durations, isErrorOutOfFunds } from '@hoprnet/hopr-utils'

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

export type HoprOptions = {
  provider: string
  announce?: boolean
  dbPath?: string
  createDbIfNotExist?: boolean
  forceCreateDB?: boolean
  password?: string
  connector?: HoprCoreEthereum
  strategy?: ChannelStrategy
  hosts?: {
    ip4?: NetOptions
    ip6?: NetOptions
  }
  // You almost certainly want this to be false, this is so we can test with
  // local testnets, and announce 127.0.0.1 addresses.
  announceLocalAddresses?: boolean

  // when true, addresses will be sorted local first
  // when false, addresses will be sorted public first
  preferLocalAddresses?: boolean
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
  private db: HoprDB
  private paymentChannels: HoprCoreEthereum
  private addressSorter: AddressSorter
  private publicNodesEmitter: HoprConnectOptions['publicNodes']

  public indexer: Indexer

  /**
   * Create an uninitialized Hopr Node
   *
   * @constructor
   *
   * @param options
   * @param provider
   */
  public constructor(private id: PeerId, private options: HoprOptions) {
    super()

    if (!id.privKey || !isSecp256k1PeerId(id)) {
      throw new Error('Hopr Node must be initialized with an id with a secp256k1 private key')
    }
    this.db = new HoprDB(
      PublicKey.fromPrivKey(id.privKey.marshal()),
      options.createDbIfNotExist,
      VERSION,
      options.dbPath,
      options.forceCreateDB
    )
    this.paymentChannels = new HoprCoreEthereum(this.db, PublicKey.fromPeerId(this.id), this.id.privKey.marshal(), {
      provider: this.options.provider
    })

    this.publicNodesEmitter = new EventEmitter()

    if (this.options.preferLocalAddresses) {
      this.addressSorter = localAddressesFirst
      log('Preferring local addresses')
    } else {
      // Overwrite libp2p's default addressSorter to make
      // sure it doesn't fail on HOPR-flavored addresses
      this.addressSorter = (x) => x
      log('Addresses are sorted by default')
    }
    this.indexer = this.paymentChannels.indexer // TODO temporary
  }

  private async startedPaymentChannels(): Promise<HoprCoreEthereum> {
    return await this.paymentChannels.start()
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
    if ((await this.getNativeBalance()).toBN().lte(MIN_NATIVE_BALANCE)) {
      throw new Error('Cannot start node without a funded wallet')
    }

    const chain = await this.startedPaymentChannels()
    verbose('Started HoprEthereum. Waiting for indexer to find connected nodes.')

    const initialNodes = await chain.waitForPublicNodes()

    const recentlyAnnouncedNodes: PeerStoreAddress[] = []
    const pushToRecentlyAnnouncedNodes = (peer: PeerStoreAddress) => recentlyAnnouncedNodes.push(peer)
    chain.on('peer', pushToRecentlyAnnouncedNodes)

    const libp2p = await LibP2P.create({
      peerId: this.id,
      addresses: { listen: getAddrs(this.id, this.options).map((x) => x.toString()) },
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
            publicNodes: this.publicNodesEmitter,
            // Tells hopr-connect to treat local and private addresses
            // as public addresses
            __useLocalAddresses: this.options.announceLocalAddresses
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
        }
      },
      dialer: {
        addressSorter: this.addressSorter,
        maxDialsPerPeer: 100
      }
    })

    await libp2p.start()
    log('libp2p started')
    this.libp2p = libp2p

    chain.indexer.on('peer', this.onPeerAnnouncement.bind(this))
    chain.indexer.off('peer', pushToRecentlyAnnouncedNodes)
    chain.indexer.on('channel-waiting-for-commitment', (c: ChannelEntry) => this.onChannelWaitingForCommitment(c))

    recentlyAnnouncedNodes.forEach(this.onPeerAnnouncement.bind(this))

    initialNodes.forEach(this.onPeerAnnouncement.bind(this))

    this.libp2p.connectionManager.on('peer:connect', (conn: Connection) => {
      this.emit('hopr:peer:connection', conn.remotePeer)
      this.networkPeers.register(conn.remotePeer)
    })

    this.networkPeers = new NetworkPeers(
      Array.from(this.libp2p.peerStore.peers.values()).map((x) => x.id),
      [this.id],
      (peer: PeerId) => this.publicNodesEmitter.emit('removePublicNode', peer)
    )

    // Subscribe to p2p events from libp2p. Wraps our instance of libp2p.
    const subscribe: Subscribe = (
      protocol: string,
      handler: LibP2PHandlerFunction<Promise<void | Uint8Array>>,
      includeReply: boolean,
      errHandler: (err: any) => void
    ) => libp2pSubscribe(this.libp2p, protocol, handler, errHandler, includeReply)

    const sendMessage: SendMessage = (
      dest: PeerId,
      protocol: string,
      msg: Uint8Array,
      includeReply: boolean,
      opts: DialOpts
    ) => libp2pSendMessage(this.libp2p, dest, protocol, msg, includeReply, opts) as any

    const hangup = this.libp2p.hangUp.bind(this.libp2p)

    this.heartbeat = new Heartbeat(this.networkPeers, subscribe, sendMessage, hangup)

    const ethereum = await this.startedPaymentChannels()

    subscribeToAcknowledgements(
      subscribe,
      this.db,
      this.getId(),
      (ackChallenge: HalfKeyChallenge) => {
        this.emit('message-acknowledged:' + ackChallenge.toHex())
      },
      (ack: AcknowledgedTicket) => ethereum.emit('ticket:win', ack),
      // TODO: automatically reinitialize commitments
      () => {}
    )

    ethereum.on('ticket:win', (ack) => {
      this.onWinningTicket(ack)
    })

    const onMessage = (msg: Uint8Array) => this.emit('hopr:message', msg)
    this.forward = new PacketForwardInteraction(subscribe, sendMessage, this.getId(), onMessage, this.db)

    await this.announce(this.options.announce)
    log('announcing done, starting heartbeat')
    this.heartbeat.start()

    this.setChannelStrategy(this.options.strategy || new PassiveStrategy())
    this.status = 'RUNNING'
    this.emit('running with strategy', this.strategy.name)

    // Log information
    // Debug log used in e2e integration tests, please don't change
    log('# STARTED NODE')
    log('ID', this.getId().toB58String())
    log('Protocol version', VERSION)
    log(`Available under the following addresses:`)
    libp2p.multiaddrs.forEach((ma: Multiaddr) => log(ma.toString()))
    this.maybeLogProfilingToGCloud()
    this.periodicCheck()
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
      const chain = await this.startedPaymentChannels()
      log(`Found channel ${c.getId().toHex()} to us with unset commitment. Setting commitment`)
      retryWithBackoff(() => chain.commitToChannel(c))
    }
  }

  /**
   * If error provided is considered an out of funds error
   * - it will emit that the node is out of funds
   * @param error error thrown by an ethereum transaction
   */
  private async isOutOfFunds(error: any): Promise<void> {
    const isOutOfFunds = isErrorOutOfFunds(error)
    if (!isOutOfFunds) return

    const address = (await this.getEthereumAddress()).toHex()

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

    const dialables = peer.multiaddrs.filter((ma: Multiaddr) => {
      const tuples = ma.tuples()
      return tuples.length > 1 && tuples[0][0] != protocols.names['p2p'].code
    })

    // @ts-ignore
    this.libp2p.peerStore.keyBook.set(peer.id)

    if (dialables.length > 0) {
      this.publicNodesEmitter.emit('addPublicNode', { id: peer.id, multiaddrs: dialables })

      this.libp2p.peerStore.addressBook.add(peer.id, dialables)
    }
  }

  // On the strategy interval, poll the strategy to see what channel changes
  // need to be made.
  private async tickChannelStrategy() {
    verbose('strategy tick', this.status, this.strategy.name)
    if (this.status != 'RUNNING') {
      return
    }

    const currentChannels = await this.getAllChannels()
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
    const chain = await this.startedPaymentChannels()
    const [nextChannels, closeChannels] = await this.strategy.tick(
      balance.toBN(),
      currentChannels,
      this.networkPeers,
      chain.getRandomOpenChannel.bind(chain)
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
    clearTimeout(this.checkTimeout)
    await Promise.all([this.heartbeat.stop(), (await this.startedPaymentChannels()).stop()])

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

    return (await this.libp2p.peerRouting.findPeer(peer))?.multiaddrs || []
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
    return (this.libp2p.peerStore.get(peer).addresses ?? []).map((addr) => addr.multiaddr)
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
  public async ping(destination: PeerId): Promise<{ info: string; latency: number }> {
    if (!PeerId.isPeerId(destination)) {
      throw Error(`Expecting a non-empty destination.`)
    }
    let start = Date.now()
    try {
      const success = await this.heartbeat.pingNode(destination)
      if (success) {
        return { latency: Date.now() - start, info: '' }
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
    const announced = await this.paymentChannels.indexer.getAnnouncedAddresses()
    return `${connected}
    \n${announced.length} peers have announced themselves on chain:
    \n${announced.map((x: Multiaddr) => x.toString()).join('\n')}`
  }

  private async periodicCheck() {
    log('periodic check', this.status)
    if (this.status != 'RUNNING') {
      return
    }
    const logTimeout = setTimeout(() => {
      log('strategy tick took longer than 10 secs')
    }, 10000)
    try {
      await this.tickChannelStrategy()
    } catch (e) {
      log('error in periodic check', e)
    }
    clearTimeout(logTimeout)

    this.checkTimeout = setTimeout(() => this.periodicCheck(), this.strategy.tickInterval)
  }

  /**
   * Announces address of node on-chain to be reachable by other nodes.
   * @dev Promise resolves before own announcment appears in the indexer
   * @param includeRouting publish routable address if true
   * @returns Promise that resolves once announce transaction has been published
   */
  private async announce(includeRouting = false): Promise<void> {
    const chain = await this.startedPaymentChannels()

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
    const ownAccount = await chain.getAccount(await this.getEthereumAddress())

    // Do not announce if our last is equal to what we intend to announce
    if (ownAccount?.multiAddr?.equals(addrToAnnounce)) {
      log(`intended address has already been announced, nothing to do`)
      return
    }

    try {
      log(`announcing ${includeRouting && isRoutableAddress ? 'with' : 'without'} routing`)

      await chain.announce(addrToAnnounce)
    } catch (err) {
      log('announce failed')
      await this.isOutOfFunds(err)
      throw new Error(`Failed to announce: ${err}`)
    }
  }

  public async setChannelStrategy(strategy: ChannelStrategy) {
    log('setting channel strategy from', this.strategy?.name, 'to', strategy.name)
    this.strategy = strategy
  }

  private async onWinningTicket(ack) {
    this.strategy.onWinningTicket(ack, await this.startedPaymentChannels())
  }

  public getChannelStrategy(): string {
    return this.strategy.name
  }

  public async getBalance(): Promise<Balance> {
    const chain = await this.startedPaymentChannels()
    return await chain.getBalance(true)
  }

  public async getNativeBalance(): Promise<NativeBalance> {
    const chain = await this.startedPaymentChannels()
    return await chain.getNativeBalance(true)
  }

  public async smartContractInfo(): Promise<{
    network: string
    hoprTokenAddress: string
    hoprChannelsAddress: string
    channelClosureSecs: number
  }> {
    const chain = await this.startedPaymentChannels()
    return chain.smartContractInfo()
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
    const ethereum = this.paymentChannels
    const selfPubKey = new PublicKey(this.getId().pubKey.marshal())
    const counterpartyPubKey = new PublicKey(counterparty.pubKey.marshal())
    const myAvailableTokens = await ethereum.getBalance(true)

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
        channelId: await ethereum.openChannel(counterpartyPubKey, new Balance(amountToFund))
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
    const ethereum = await this.startedPaymentChannels()
    const selfPubKey = new PublicKey(this.getId().pubKey.marshal())
    const counterpartyPubKey = new PublicKey(counterparty.pubKey.marshal())
    const myBalance = await ethereum.getBalance(false)
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
      await ethereum.fundChannel(counterpartyPubKey, new Balance(myFund), new Balance(counterpartyFund))
    } catch (err) {
      await this.isOutOfFunds(err)
      throw new Error(`Failed to fundChannel: ${err}`)
    }
  }

  public async closeChannel(counterparty: PeerId): Promise<{ receipt: string; status: ChannelStatus }> {
    const ethereum = await this.startedPaymentChannels()
    const selfPubKey = new PublicKey(this.getId().pubKey.marshal())
    const counterpartyPubKey = new PublicKey(counterparty.pubKey.marshal())
    const channel = await this.db.getChannelX(selfPubKey, counterpartyPubKey)

    // TODO: should we wait for confirmation?
    if (channel.status === ChannelStatus.Closed) {
      throw new Error('Channel is already closed')
    }

    if (channel.status === ChannelStatus.Open) {
      await this.strategy.onChannelWillClose(channel, ethereum)
    }

    log('closing channel', channel.getId())
    let txHash: string
    try {
      if (channel.status === ChannelStatus.Open || channel.status == ChannelStatus.WaitingForCommitment) {
        log('initiating closure')
        txHash = await ethereum.initializeClosure(counterpartyPubKey)
      } else {
        log('finalizing closure')
        txHash = await ethereum.finalizeClosure(counterpartyPubKey)
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
    const ethereum = await this.startedPaymentChannels()
    await ethereum.redeemAllTickets()
  }

  public async getChannelsFrom(addr: Address): Promise<ChannelEntry[]> {
    return await this.db.getChannelsFrom(addr)
  }

  public async getChannelsTo(addr: Address): Promise<ChannelEntry[]> {
    return await this.db.getChannelsTo(addr)
  }

  public async getPublicKeyOf(addr: Address): Promise<PublicKey> {
    const ethereum = await this.startedPaymentChannels()
    return await ethereum.getPublicKeyOf(addr)
  }

  // NB: The prefix "HOPR Signed Message: " is added as a security precaution.
  // Without it, the node could be convinced to sign a message like an Ethereum
  // transaction draining it's connected wallet funds, since they share the key.
  public async signMessage(message: Uint8Array) {
    return await this.id.privKey.sign(u8aConcat(new TextEncoder().encode('HOPR Signed Message: '), message))
  }

  public async getEthereumAddress(): Promise<Address> {
    const ethereum = await this.startedPaymentChannels()
    return ethereum.getPublicKey().toAddress()
  }

  public async withdraw(currency: 'NATIVE' | 'HOPR', recipient: string, amount: string): Promise<string> {
    const ethereum = await this.startedPaymentChannels()

    try {
      return ethereum.withdraw(currency, recipient, amount)
    } catch (err) {
      await this.isOutOfFunds(err)
      throw new Error(`Failed to withdraw: ${err}`)
    }
  }

  /**
   * Takes a destination and samples randomly intermediate nodes
   * that will relay that message before it reaches its destination.
   *
   * @param destination instance of peerInfo that contains the peerId of the destination
   */
  private async getIntermediateNodes(destination: PublicKey): Promise<PublicKey[]> {
    const ethereum = await this.startedPaymentChannels()
    return await findPath(
      PublicKey.fromPeerId(this.getId()),
      destination,
      INTERMEDIATE_HOPS,
      (p: PublicKey) => this.networkPeers.qualityOf(p.toPeerId()),
      ethereum.getOpenChannelsFrom.bind(ethereum)
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

export { Hopr as default, LibP2P }
export * from './constants'
export { PassiveStrategy, PromiscuousStrategy, SaneDefaults, findPath }
export type { ChannelsToOpen, ChannelsToClose }
