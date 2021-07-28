import LibP2P from 'libp2p'
import type { Connection } from 'libp2p'

const MPLEX = require('libp2p-mplex')
import KadDHT from 'libp2p-kad-dht'
import { NOISE } from 'libp2p-noise'

const { HoprConnect } = require('@hoprnet/hopr-connect')
import type { HoprConnectOptions } from '@hoprnet/hopr-connect'

import { PACKET_SIZE, INTERMEDIATE_HOPS, VERSION, CHECK_TIMEOUT, PATH_RANDOMNESS, FULL_VERSION } from './constants'

import NetworkPeers from './network/network-peers'
import Heartbeat from './network/heartbeat'
import { findPath } from './path'

import { protocols, Multiaddr } from 'multiaddr'
import chalk from 'chalk'

import PeerId from 'peer-id'
import {
  PublicKey,
  Balance,
  Address,
  ChannelEntry,
  NativeBalance,
  Hash,
  DialOpts,
  HoprDB,
  libp2pSendMessageAndExpectResponse,
  libp2pSubscribe,
  libp2pSendMessage,
  LibP2PHandlerFunction,
  isSecp256k1PeerId,
  AcknowledgedTicket,
  ChannelStatus,
  MIN_NATIVE_BALANCE
} from '@hoprnet/hopr-utils'
import HoprCoreEthereum from '@hoprnet/hopr-core-ethereum'
import BN from 'bn.js'
import { getAddrs } from './identity'

import EventEmitter from 'events'
import { ChannelStrategy, PassiveStrategy, PromiscuousStrategy } from './channel-strategy'
import Debug from 'debug'

import { subscribeToAcknowledgements } from './interactions/packet/acknowledgement'
import { PacketForwardInteraction } from './interactions/packet/forward'

import { Packet } from './messages'
import { localAddressesFirst, AddressSorter, backoff, durations, isErrorOutOfFunds } from '@hoprnet/hopr-utils'

const log = Debug(`hopr-core`)
const verbose = Debug('hopr-core:verbose')

interface NetOptions {
  ip: string
  port: number
}

type PeerStoreAddress = {
  id: PeerId
  multiaddrs: Multiaddr[]
}

export type ChannelStrategyNames = 'passive' | 'promiscuous'

export type HoprOptions = {
  network: string
  provider: string
  announce?: boolean
  dbPath?: string
  createDbIfNotExist?: boolean
  password?: string
  connector?: HoprCoreEthereum
  strategy?: ChannelStrategyNames
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

class Hopr extends EventEmitter {
  public status: NodeStatus = 'UNINITIALIZED'

  private checkTimeout: NodeJS.Timeout
  private strategy: ChannelStrategy
  private networkPeers: NetworkPeers
  private heartbeat: Heartbeat
  private forward: PacketForwardInteraction
  private libp2p: LibP2P
  private db: HoprDB
  private paymentChannels: Promise<HoprCoreEthereum>
  private addressSorter: AddressSorter
  private publicNodesEmitter: HoprConnectOptions['publicNodes']

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
      PublicKey.fromPrivKey(id.privKey.marshal()).toAddress(),
      options.createDbIfNotExist,
      VERSION,
      options.dbPath
    )
    this.paymentChannels = HoprCoreEthereum.create(this.db, this.id.privKey.marshal(), {
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

    verbose('Started HoprEthereum. Waiting for indexer to find connected nodes.')

    const ethereum = await this.paymentChannels
    const initialNodes = await ethereum.waitForPublicNodes()

    const recentlyAnnouncedNodes: PeerStoreAddress[] = []
    const pushToInitialNodes = (peer: { id: PeerId; multiaddrs: Multiaddr[] }) => recentlyAnnouncedNodes.push(peer)
    ethereum.on('peer', pushToInitialNodes)

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
        transport: {
          HoprConnect: {
            initialNodes: initialNodes.map((addr) => addr.multiaddrs).flat(),
            publicNodes: this.publicNodesEmitter
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

    console.log(`announced nodes`)

    initialNodes.concat(recentlyAnnouncedNodes).forEach(this.onPeerAnnouncement.bind(this))

    ethereum.indexer.off('peer', pushToInitialNodes)
    ethereum.indexer.on('peer', this.onPeerAnnouncement.bind(this))

    this.libp2p.connectionManager.on('peer:connect', (conn: Connection) => {
      this.emit('hopr:peer:connection', conn.remotePeer)
      this.networkPeers.register(conn.remotePeer)
    })

    this.networkPeers = new NetworkPeers(
      Array.from(this.libp2p.peerStore.peers.values()).map((x) => x.id),
      [this.id],
      (peer: PeerId) => this.publicNodesEmitter.emit('removePublicNode', peer)
    )

    const subscribe = (protocol: string, handler: LibP2PHandlerFunction, includeReply = false) =>
      libp2pSubscribe(this.libp2p, protocol, handler, includeReply)
    const sendMessageAndExpectResponse = (dest: PeerId, protocol: string, msg: Uint8Array, opts: DialOpts) =>
      libp2pSendMessageAndExpectResponse(this.libp2p, dest, protocol, msg, opts)
    const sendMessage = (dest: PeerId, protocol: string, msg: Uint8Array, opts: DialOpts) =>
      libp2pSendMessage(this.libp2p, dest, protocol, msg, opts)
    const hangup = this.libp2p.hangUp.bind(this.libp2p)

    this.heartbeat = new Heartbeat(this.networkPeers, subscribe, sendMessageAndExpectResponse, hangup)

    subscribeToAcknowledgements(subscribe, this.db, await this.paymentChannels, this.getId(), (ack) =>
      this.emit('message-acknowledged:' + ack.ackChallenge.toHex())
    )

    const onMessage = (msg: Uint8Array) => this.emit('hopr:message', msg)
    this.forward = new PacketForwardInteraction(subscribe, sendMessage, this.getId(), ethereum, onMessage, this.db)

    await this.announce(this.options.announce)
    log('announcing done, starting heartbeat')

    this.heartbeat.start()
    this.setChannelStrategy(this.options.strategy || 'passive')
    this.status = 'RUNNING'
    this.emit('running')

    // Log information
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

  /**
   * If error provided is considered an out of funds error
   * - it will emit that the node is out of funds
   * - it will set channel strategy to passive
   * @param error error thrown by an ethereum transaction
   */
  private async isOutOfFunds(error: any): Promise<void> {
    const isOutOfFunds = isErrorOutOfFunds(error)
    if (!isOutOfFunds) return

    const address = (await this.getEthereumAddress()).toHex()

    if (isOutOfFunds === 'NATIVE') {
      log('unfunded node', address)
      this.emit('hopr:warning:unfundedNative', address)
      await this.getNativeBalance()
    } else if (isOutOfFunds === 'HOPR') {
      log('unfunded node', address)
      this.emit('hopr:warning:unfunded', address)
      await this.getBalance()
    }

    await this.setChannelStrategy('passive')
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

    // @ts-ignore
    this.libp2p.peerStore.keyBook.set(peer.id)

    const dialables: Multiaddr[] = []

    for (const ma of peer.multiaddrs) {
      const tuples = ma.tuples()

      if (tuples.length == 1 || tuples[0][0] == protocols.names['p2p'].code) {
        continue
      }

      this.publicNodesEmitter.emit('addPublicNode', ma)

      dialables.push(ma)
    }

    if (dialables.length > 0) {
      this.libp2p.peerStore.addressBook.add(peer.id, dialables)
    }
  }

  private async tickChannelStrategy() {
    verbose('strategy tick', this.status)
    if (this.status != 'RUNNING') {
      return
    }

    // TODO: replace with newChannels
    const rndChannels = await this.getRandomOpenChannels()

    for (const channel of rndChannels) {
      this.networkPeers.register(channel.source.toPeerId()) // Listen to nodes with outgoing stake
    }
    const currentChannels = await this.getOpenChannels()
    for (const channel of currentChannels) {
      this.networkPeers.register(channel.destination.toPeerId()) // Make sure current channels are 'interesting'
    }
    const balance = await this.getBalance()
    const chain = await this.paymentChannels
    const [nextChannels, closeChannels] = await this.strategy.tick(
      balance.toBN(),
      rndChannels,
      currentChannels,
      this.networkPeers,
      chain.getRandomOpenChannel.bind(this.paymentChannels)
    )
    verbose(`strategy wants to close ${closeChannels.length} channels`)
    for (let toClose of closeChannels) {
      verbose(`closing ${toClose}`)
      try {
        await this.closeChannel(toClose.toPeerId())
        verbose(`closed channel to ${toClose.toString()}`)
        this.emit('hopr:channel:closed', toClose)
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

  /**
   * Randomly pick 10 open channels
   * @returns maximum 10 open channels
   */
  private async getRandomOpenChannels(): Promise<ChannelEntry[]> {
    const chain = await this.paymentChannels
    const channels = new Map<string, ChannelEntry>()

    for (let i = 0; i < 100; i++) {
      if (channels.size >= 10) break

      const channel = await chain.getRandomOpenChannel()
      if (!channel) break

      if (channels.has(channel.getId().toHex())) continue
      channels.set(channel.getId().toHex(), channel)
    }

    return Array.from(channels.values())
  }

  private async getOpenChannels(): Promise<ChannelEntry[]> {
    return (await this.paymentChannels).getOpenChannelsFrom(PublicKey.fromPeerId(this.getId()))
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
    await Promise.all([this.heartbeat.stop(), (await this.paymentChannels).stop()])

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
    return this.libp2p.peerStore.get(peer).addresses ?? []
  }

  /**
   * @param msg message to send
   * @param destination PeerId of the destination
   * @param intermediateNodes optional set path manually
   */
  public async sendMessage(msg: Uint8Array, destination: PeerId, intermediatePath?: PublicKey[]): Promise<void> {
    const promises: Promise<void>[] = []
    const ethereum = await this.paymentChannels

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

        const channel = ethereum.getChannel(ticketIssuer, ticketReceiver)
        const channelState = await channel.usToThem()

        if (typeof channelState === 'undefined') {
          throw Error(
            `Channel from ${ticketIssuer.toPeerId().toB58String()} to ${ticketReceiver
              .toPeerId()
              .toB58String()} not found`
          )
        } else if (channelState.status !== ChannelStatus.Open) {
          throw Error(`Channel ${channelState.getId().toHex()} is not open`)
        }

        if (channelState.ticketEpoch.toBN().isZero()) {
          throw Error(
            `Cannot use manually set path because apparently there is no commitment set for the channel between ${ticketIssuer
              .toPeerId()
              .toB58String()} and ${ticketReceiver.toPeerId().toB58String()}`
          )
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
          console.log('>>>', path)

          let packet: Packet
          try {
            packet = await Packet.create(
              msg.slice(n * PACKET_SIZE, Math.min(msg.length, (n + 1) * PACKET_SIZE)),
              path.map((x) => x.toPeerId()),
              this.getId(),
              await this.paymentChannels
            )
          } catch (err) {
            return reject(err)
          }

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
    const announced = await (await this.paymentChannels).indexer.getAnnouncedAddresses()
    return `${connected}
    \n${announced.length} peers have announced themselves on chain:
    \n${announced.map((x: Multiaddr) => x.toString()).join('\n')}`
  }

  private async periodicCheck() {
    log('periodic check', this.status)
    if (this.status != 'RUNNING') {
      return
    }
    try {
      await this.tickChannelStrategy()
    } catch (e) {
      log('error in periodic check', e)
    }
    this.checkTimeout = setTimeout(() => this.periodicCheck(), CHECK_TIMEOUT)
  }

  private async announce(includeRouting: boolean = false): Promise<void> {
    const chain = await this.paymentChannels

    const multiaddrs = await this.getAnnouncedAddresses()

    const ip4 = multiaddrs.find((s) => s.toString().startsWith('/ip4/'))
    const ip6 = multiaddrs.find((s) => s.toString().startsWith('/ip6/'))

    const p2p = new Multiaddr('/p2p/' + this.getId().toB58String())

    const addrToAnnounce = ip4 ?? ip6 ?? p2p
    const isRoutableAddress = (ip4 ?? ip6) != undefined

    const ownAccount = await chain.getAccount(await this.getEthereumAddress())

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

  public async setChannelStrategy(strategy: ChannelStrategyNames) {
    if (strategy == 'passive') {
      this.strategy = new PassiveStrategy()
      return
    }
    if (strategy == 'promiscuous') {
      this.strategy = new PromiscuousStrategy()
      return
    }

    const ethereum = await this.paymentChannels
    ethereum.on('ticket:win', (ack, channel) => {
      this.strategy.onWinningTicket(ack, channel)
    })
    throw new Error('Unknown strategy')
  }

  public getChannelStrategy(): string {
    return this.strategy.name
  }

  public async getBalance(): Promise<Balance> {
    const chain = await this.paymentChannels
    return await chain.getBalance(true)
  }

  public async getNativeBalance(): Promise<NativeBalance> {
    const chain = await this.paymentChannels
    return await chain.getNativeBalance(true)
  }

  public async smartContractInfo(): Promise<{
    network: string
    hoprTokenAddress: string
    hoprChannelsAddress: string
    channelClosureSecs: number
  }> {
    const chain = await this.paymentChannels
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
    const ethereum = await this.paymentChannels
    const selfPubKey = new PublicKey(this.getId().pubKey.marshal())
    const counterpartyPubKey = new PublicKey(counterparty.pubKey.marshal())
    const myAvailableTokens = await ethereum.getBalance(true)

    // validate 'amountToFund'
    if (amountToFund.lten(0)) {
      throw Error(`Invalid 'amountToFund' provided: ${amountToFund.toString(10)}`)
    } else if (amountToFund.gt(myAvailableTokens.toBN())) {
      throw Error(`You don't have enough tokens: ${amountToFund.toString(10)}<${myAvailableTokens.toBN().toString(10)}`)
    }

    const channel = ethereum.getChannel(selfPubKey, counterpartyPubKey)
    let channelId: Hash

    try {
      channelId = await channel.open(new Balance(amountToFund))
    } catch (err) {
      await this.isOutOfFunds(err)
      throw new Error(`Failed to openChannel: ${err}`)
    }

    return {
      channelId
    }
  }

  /**
   * Fund a payment channel
   *
   * @param counterparty the counter party's peerId
   * @param myFund the amount to fund the channel in my favor HOPR(wei)
   * @param counterpartyFund the amount to fund the channel in counterparty's favor HOPR(wei)
   */
  public async fundChannel(
    counterparty: PeerId,
    myFund: BN,
    counterpartyFund: BN
  ): Promise<{
    channelId: Hash
  }> {
    const ethereum = await this.paymentChannels
    const selfPubKey = new PublicKey(this.getId().pubKey.marshal())
    const counterpartyPubKey = new PublicKey(counterparty.pubKey.marshal())
    const myBalance = await ethereum.getBalance(false)
    const totalFund = myFund.add(counterpartyFund)

    // validate 'amountToFund'
    if (totalFund.lten(0)) {
      throw Error(`Invalid 'totalFund' provided: ${totalFund.toString(10)}`)
    } else if (totalFund.gt(myBalance.toBN())) {
      throw Error(`You don't have enough tokens: ${totalFund.toString(10)}<${myBalance.toBN().toString(10)}`)
    }

    const channel = ethereum.getChannel(selfPubKey, counterpartyPubKey)

    try {
      await channel.fund(new Balance(myFund), new Balance(counterpartyFund))
    } catch (err) {
      await this.isOutOfFunds(err)
      throw new Error(`Failed to fundChannel: ${err}`)
    }

    return {
      channelId: (await channel.usToThem()).getId()
    }
  }

  public async closeChannel(counterparty: PeerId): Promise<{ receipt: string; status: ChannelStatus }> {
    const ethereum = await this.paymentChannels
    const selfPubKey = new PublicKey(this.getId().pubKey.marshal())
    const counterpartyPubKey = new PublicKey(counterparty.pubKey.marshal())
    const channel = ethereum.getChannel(selfPubKey, counterpartyPubKey)
    const channelState = await channel.usToThem()

    // TODO: should we wait for confirmation?
    if (channelState.status === ChannelStatus.Closed) {
      throw new Error('Channel is already closed')
    }

    if (channelState.status === ChannelStatus.Open) {
      await this.strategy.onChannelWillClose(channel)
    }

    let txHash: string
    try {
      txHash = await (channelState.status === ChannelStatus.Open
        ? channel.initializeClosure()
        : channel.finalizeClosure())
    } catch (err) {
      await this.isOutOfFunds(err)
      throw new Error(`Failed to closeChannel: ${err}`)
    }

    return { receipt: txHash, status: channelState.status }
  }

  public async getAcknowledgedTickets() {
    return this.db.getAcknowledgedTickets()
  }

  public async getTicketStatistics() {
    const ack = await this.getAcknowledgedTickets()
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
      redeemedValue: await this.db.getRedeemedTicketsValue()
    }
  }

  public async redeemAllTickets() {
    let count = 0,
      redeemed = 0,
      total = new BN(0)

    for (const ackTicket of await this.getAcknowledgedTickets()) {
      count++
      const result = await this.redeemAcknowledgedTicket(ackTicket)

      if (result.status === 'SUCCESS') {
        redeemed++
        total.iadd(ackTicket.ticket.amount.toBN())
        console.log(`Redeemed ticket ${count}`)
      } else {
        console.log(`Failed to redeem ticket ${count}`)
      }
    }
    return {
      count,
      redeemed,
      total: new Balance(total)
    }
  }

  public async redeemAcknowledgedTicket(ackTicket: AcknowledgedTicket) {
    const ethereum = await this.paymentChannels
    const channel = ethereum.getChannel(ethereum.getPublicKey(), ackTicket.signer)

    try {
      return await channel.redeemTicket(ackTicket)
    } catch (err) {
      await this.isOutOfFunds(err)
      throw new Error(`Failed to redeemAcknowledgedTicket: ${err}`)
    }
  }

  public async getChannelsFrom(addr: Address): Promise<ChannelEntry[]> {
    const ethereum = await this.paymentChannels
    return await ethereum.getChannelsFrom(addr)
  }

  public async getChannelsTo(addr: Address): Promise<ChannelEntry[]> {
    const ethereum = await this.paymentChannels
    return await ethereum.getChannelsTo(addr)
  }

  public async getPublicKeyOf(addr: Address): Promise<PublicKey> {
    const ethereum = await this.paymentChannels
    return await ethereum.getPublicKeyOf(addr)
  }

  public async getEthereumAddress(): Promise<Address> {
    const ethereum = await this.paymentChannels
    return ethereum.getAddress()
  }

  public async withdraw(currency: 'NATIVE' | 'HOPR', recipient: string, amount: string): Promise<string> {
    const ethereum = await this.paymentChannels

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
    const ethereum = await this.paymentChannels
    return await findPath(
      PublicKey.fromPeerId(this.getId()),
      destination,
      INTERMEDIATE_HOPS,
      this.networkPeers,
      ethereum.getOpenChannelsFrom.bind(ethereum),
      PATH_RANDOMNESS
    )
  }

  /**
   * This is a utility method to wait until the node is funded.
   * A backoff is implemented, if node has not been funded and
   * MAX_DELAY is reached, this function will reject.
   */
  public async waitForFunds(): Promise<void> {
    const MIN_DELAY = durations.seconds(30)
    const MAX_DELAY = durations.seconds(200)

    try {
      return backoff(
        () => {
          return new Promise<void>(async (resolve, reject) => {
            const nativeBalance = await this.getNativeBalance()

            if (nativeBalance.toBN().gte(MIN_NATIVE_BALANCE)) {
              resolve()
            } else {
              log('still unfunded, trying again soon')
              reject()
            }
          })
        },
        {
          minDelay: MIN_DELAY,
          maxDelay: MAX_DELAY,
          delayMultiple: 1.05
        }
      )
    } catch {
      log(`unfunded for more than ${Math.round(MAX_DELAY / 60e3)} minutes, shutting down`)
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
