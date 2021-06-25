import LibP2P from 'libp2p'
import type { Connection } from 'libp2p'

const MPLEX = require('libp2p-mplex')
const KadDHT = require('libp2p-kad-dht')
import { NOISE } from 'libp2p-noise'

const { HoprConnect } = require('@hoprnet/hopr-connect')

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
import { Address as LibP2PAddress } from 'libp2p/src/peer-store'

import { subscribeToAcknowledgements } from './interactions/packet/acknowledgement'
import { PacketForwardInteraction } from './interactions/packet/forward'

import { Packet } from './messages'
import { localAddressesFirst, AddressSorter } from '@hoprnet/hopr-utils'

const log = Debug(`hopr-core`)
const verbose = Debug('hopr-core:verbose')

interface NetOptions {
  ip: string
  port: number
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

    if (!id.privKey) {
      // TODO - assert secp256k1?
      throw new Error('Hopr Node must be initialized with an id with a private key')
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

    const chain = await this.paymentChannels
    verbose('Started HoprEthereum. Waiting for indexer to find connected nodes.')
    const publicNodes = await chain.waitForPublicNodes()
    if (publicNodes.length == 0) {
      log('No public nodes have announced yet, we cannot rely on relay')
    }
    verbose('Using public nodes:', publicNodes)

    const libp2p = await LibP2P.create({
      peerId: this.id,
      addresses: { listen: getAddrs(this.id, this.options).map((x) => x.toString()) },
      // libp2p modules
      modules: {
        transport: [HoprConnect as any], // TODO re https://github.com/hoprnet/hopr-connect/issues/78
        streamMuxer: [MPLEX],
        connEncryption: [NOISE],
        // @ts-ignore //TODO 'Libp2pModules' does not contain types for DHT as ov v0.30 see js-libp2p/659
        dht: KadDHT
      },
      config: {
        transport: {
          HoprConnect: {
            bootstrapServers: publicNodes
            // @dev Use these settings to simulate NAT behavior
            // __noDirectConnections: true,
            // __noWebRTCUpgrade: false
          }
        },
        dht: {
          enabled: true
        },
        //@ts-ignore - bug in libp2p options
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
    this.libp2p.connectionManager.on('peer:connect', (conn: Connection) => {
      this.emit('hopr:peer:connection', conn.remotePeer)
      this.networkPeers.register(conn.remotePeer)
    })

    // Feed previously announced addresses to DHT
    for (const ma of publicNodes) {
      this.libp2p.peerStore.addressBook.add(PeerId.createFromB58String(ma.getPeerId()), [ma])
    }

    this.networkPeers = new NetworkPeers(Array.from(this.libp2p.peerStore.peers.values()).map((x) => x.id))

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

    const ethereum = await this.paymentChannels
    const onMessage = (msg: Uint8Array) => this.emit('hopr:message', msg)
    this.forward = new PacketForwardInteraction(subscribe, sendMessage, this.getId(), ethereum, onMessage, this.db)

    ethereum.indexer.on('peer', ({ id, multiaddrs }: { id: PeerId; multiaddrs: Multiaddr[] }) => {
      const dialables = multiaddrs.filter((ma: Multiaddr) => {
        const tuples = ma.tuples()
        return tuples.length > 1 || tuples[0][0] != protocols.names['p2p'].code
      })

      // @ts-ignore
      this.libp2p.peerStore.keyBook.set(id)

      if (dialables.length > 0) {
        this.libp2p.peerStore.addressBook.add(id, multiaddrs)
      }
    })

    log('announcing')
    await this.announce(this.options.announce)
    log('announced, starting heartbeat')

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
      await this.closeChannel(toClose.toPeerId())
      verbose(`closed channel to ${toClose.toString()}`)
      this.emit('hopr:channel:closed', toClose)
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
   * Lists the addresses which the given node announces to other nodes
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
  public getObservedAddresses(peer: PeerId): LibP2PAddress[] {
    return this.libp2p.peerStore.get(peer)?.addresses ?? []
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

  private async checkBalances() {
    const balance = await this.getBalance()
    const address = (await this.getEthereumAddress()).toHex()
    if (balance.toBN().lten(0)) {
      log('unfunded node', address)
      this.emit('hopr:warning:unfunded', address)
    }
    const nativeBalance = await this.getNativeBalance()
    if (nativeBalance.toBN().lte(MIN_NATIVE_BALANCE)) {
      log('unfunded node', address)
      this.emit('hopr:warning:unfundedNative', address)
    }
  }

  private async periodicCheck() {
    log('periodic check', this.status)
    if (this.status != 'RUNNING') {
      return
    }
    try {
      await this.checkBalances()
      await this.tickChannelStrategy()
    } catch (e) {
      log('error in periodic check', e)
    }
    this.checkTimeout = setTimeout(() => this.periodicCheck(), CHECK_TIMEOUT)
  }

  private async announce(includeRouting: boolean = false): Promise<void> {
    log('announcing self, include routing:', includeRouting)
    const chain = await this.paymentChannels
    //const account = await chain.getAccount(await this.getEthereumAddress())
    // exit if we already announced
    //if (account.hasAnnounced()) return

    if ((await this.getNativeBalance()).toBN().lte(MIN_NATIVE_BALANCE)) {
      throw new Error('Cannot announce without funds')
    }

    const multiaddrs = await this.getAnnouncedAddresses()
    const ip4 = multiaddrs.find((s) => s.toString().includes('/ip4/'))
    const ip6 = multiaddrs.find((s) => s.toString().includes('/ip6/'))
    const p2p = new Multiaddr('/p2p/' + this.getId().toB58String())
    // exit if none of these multiaddrs are available
    if (!ip4 && !ip6 && !p2p) return

    try {
      if (includeRouting && (ip4 || ip6)) {
        log('announcing with routing', ip4 || ip6)
        await chain.announce(ip4 || ip6)
        return
      }
      log('announcing without routing', p2p.toString())
      await chain.announce(p2p)
    } catch (err) {
      log('announce failed')
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
    return {
      channelId: await channel.open(new Balance(amountToFund))
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
    await channel.fund(new Balance(myFund), new Balance(counterpartyFund))

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

    const txHash = await (channelState.status === ChannelStatus.Open
      ? channel.initializeClosure()
      : channel.finalizeClosure())

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
    return await channel.redeemTicket(ackTicket)
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
    return ethereum.withdraw(currency, recipient, amount)
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

  // This is a utility method to wait until the node is funded.
  public async waitForFunds(): Promise<void> {
    return new Promise((resolve) => {
      const tick = () => {
        this.getNativeBalance().then((nativeBalance) => {
          if (nativeBalance.toBN().gt(MIN_NATIVE_BALANCE)) {
            resolve()
          } else {
            log('still unfunded, trying again soon')
            setTimeout(tick, CHECK_TIMEOUT)
          }
        })
      }
      tick()
    })
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
