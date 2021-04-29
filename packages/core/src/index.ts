import LibP2P from 'libp2p'
import type { Connection } from 'libp2p'

const MPLEX = require('libp2p-mplex')
const KadDHT = require('libp2p-kad-dht')
import { NOISE } from 'libp2p-noise'

const { HoprConnect } = require('@hoprnet/hopr-connect')

import { Packet } from './messages/packet'
import {
  PACKET_SIZE,
  MAX_HOPS,
  VERSION,
  CHECK_TIMEOUT,
  PATH_RANDOMNESS,
  MIN_NATIVE_BALANCE,
  FULL_VERSION
} from './constants'

import NetworkPeers from './network/network-peers'
import Heartbeat from './network/heartbeat'
import { findPath } from './path'

import { u8aToHex, DialOpts } from '@hoprnet/hopr-utils'

import Multiaddr from 'multiaddr'
import chalk from 'chalk'

import PeerId from 'peer-id'
import HoprCoreEthereum, {
  PublicKey,
  Balance,
  Address,
  ChannelEntry,
  NativeBalance,
  Hash,
  Acknowledgement,
  RoutingChannel
} from '@hoprnet/hopr-core-ethereum'
import BN from 'bn.js'
import { getAddrs } from './identity'

import EventEmitter from 'events'
import { ChannelStrategy, PassiveStrategy, PromiscuousStrategy } from './channel-strategy'
import Debug from 'debug'
import { Address as LibP2PAddress } from 'libp2p/src/peer-store'
import {
  libp2pSendMessageAndExpectResponse,
  libp2pSubscribe,
  libp2pSendMessage,
  LibP2PHandlerFunction
} from '@hoprnet/hopr-utils'
import { subscribeToAcknowledgements } from './interactions/packet/acknowledgement'
import { PacketForwardInteraction } from './interactions/packet/forward'
import { CoreDB } from './db'

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
  ticketAmount?: number
  ticketWinProb?: number
  dbPath?: string
  createDbIfNotExist?: boolean
  password?: string
  connector?: HoprCoreEthereum
  strategy?: ChannelStrategyNames
  hosts?: {
    ip4?: NetOptions
    ip6?: NetOptions
  }
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
  private db: CoreDB
  private paymentChannels: Promise<HoprCoreEthereum>

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
    this.db = new CoreDB(options, PublicKey.fromPrivKey(id.privKey.marshal()).toAddress())
    this.paymentChannels = HoprCoreEthereum.create(this.db.getLevelUpTempUntilRefactored(), this.id.privKey.marshal(), {
      provider: this.options.provider
    })
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
        peerDiscovery: {
          autoDial: false
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
        // Temporary fix, see https://github.com/hoprnet/hopr-connect/issues/77
        addressSorter: (a) => a,
        concurrency: 100
      }
    })

    await libp2p.start()
    this.libp2p = libp2p
    this.libp2p.connectionManager.on('peer:connect', (conn: Connection) => {
      this.emit('hopr:peer:connection', conn.remotePeer)
      this.networkPeers.register(conn.remotePeer)
    })
    this.networkPeers = new NetworkPeers(Array.from(this.libp2p.peerStore.peers.values()).map((x) => x.id))

    const subscribe = (protocol: string, handler: LibP2PHandlerFunction, includeReply = false) =>
      libp2pSubscribe(this.libp2p, protocol, handler, includeReply)
    const sendMessageAndExpectResponse = (dest: PeerId, protocol: string, msg: Uint8Array, opts: DialOpts) =>
      libp2pSendMessageAndExpectResponse(this.libp2p, dest, protocol, msg, opts)
    const sendMessage = (dest: PeerId, protocol: string, msg: Uint8Array, opts: DialOpts) =>
      libp2pSendMessage(this.libp2p, dest, protocol, msg, opts)
    const hangup = this.libp2p.hangUp.bind(this.libp2p)

    this.heartbeat = new Heartbeat(this.networkPeers, subscribe, sendMessageAndExpectResponse, hangup)

    subscribeToAcknowledgements(subscribe, this.db, await this.paymentChannels, (ack) =>
      this.emit('message-acknowledged:' + ack.getKey())
    )

    const onMessage = (msg: Uint8Array) => this.emit('hopr:message', msg)
    this.forward = new PacketForwardInteraction(
      this.db,
      await this.paymentChannels,
      this.getId(),
      this.libp2p,
      subscribe,
      sendMessage,
      onMessage
    )

    await this.heartbeat.start()
    this.periodicCheck()
    this.setChannelStrategy(this.options.strategy || 'passive')
    this.status = 'RUNNING'

    // Log information
    log('# STARTED NODE')
    log('ID', this.getId().toB58String())
    log('Protocol version', VERSION)
    log(`Available under the following addresses:`)
    libp2p.multiaddrs.forEach((ma: Multiaddr) => log(ma.toString()))
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

  private async tickChannelStrategy(newChannels: RoutingChannel[]) {
    verbose('strategy tick', this.status)
    if (this.status != 'RUNNING') {
      return
    }
    for (const channel of newChannels) {
      this.networkPeers.register(channel[0]) // Listen to nodes with outgoing stake
    }
    const currentChannels = await this.getOpenChannels()
    for (const channel of currentChannels) {
      this.networkPeers.register(channel[1]) // Make sure current channels are 'interesting'
    }
    const balance = await this.getBalance()
    const chain = await this.paymentChannels
    const [nextChannels, closeChannels] = await this.strategy.tick(
      balance.toBN(),
      newChannels,
      currentChannels,
      this.networkPeers,
      chain.getRandomChannel.bind(this.paymentChannels)
    )
    verbose(`strategy wants to close ${closeChannels.length} channels`)
    for (let toClose of closeChannels) {
      verbose(`closing ${toClose}`)
      await this.closeChannel(toClose)
      verbose(`closed channel to ${toClose.toB58String()}`)
      this.emit('hopr:channel:closed', toClose)
    }
    verbose(`strategy wants to open`, nextChannels.length, 'new channels')
    for (let channelToOpen of nextChannels) {
      this.networkPeers.register(channelToOpen[0])
      try {
        // Opening channels can fail if we can't establish a connection.
        const hash = await this.openChannel(...channelToOpen)
        verbose('- opened', channelToOpen, hash)
        this.emit('hopr:channel:opened', channelToOpen)
      } catch (e) {
        log('error when trying to open strategy channels', e)
      }
    }
  }

  private async getOpenChannels(): Promise<RoutingChannel[]> {
    return (await this.paymentChannels).getChannelsFromPeer(this.getId())
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
      return this.libp2p.multiaddrs
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
   * Sends a message.
   *
   * @notice THIS METHOD WILL SPEND YOUR ETHER.
   * @notice This method will fail if there are not enough funds to open
   * the required payment channels. Please make sure that there are enough
   * funds controlled by the given key pair.
   *
   * @param msg message to send
   * @param destination PeerId of the destination
   * @param intermediateNodes optional set path manually
   * the acknowledgement of the first hop
   */
  public async sendMessage(
    msg: Uint8Array,
    destination: PeerId,
    getIntermediateNodesManually?: () => Promise<PeerId[]>
  ): Promise<void> {
    const promises: Promise<void>[] = []

    for (let n = 0; n < msg.length / PACKET_SIZE; n++) {
      promises.push(
        new Promise<void>(async (resolve, reject) => {
          let intermediatePath: PeerId[]
          if (getIntermediateNodesManually != undefined) {
            verbose('manually creating intermediatePath')
            intermediatePath = await getIntermediateNodesManually()
          } else {
            try {
              intermediatePath = await this.getIntermediateNodes(destination)
            } catch (e) {
              reject(e)
              return
            }
            if (!intermediatePath || !intermediatePath.length) {
              reject(new Error('bad path'))
            }
          }

          const path: PeerId[] = [].concat(intermediatePath, [destination])

          let packet: Packet
          try {
            packet = await Packet.create(
              await this.paymentChannels,
              this.db,
              this.getId(),
              this.libp2p,
              msg.slice(n * PACKET_SIZE, Math.min(msg.length, (n + 1) * PACKET_SIZE)),
              path
            )
          } catch (err) {
            return reject(err)
          }

          let packetKey = await this.db.storeUnacknowledgedTicket(packet.challenge.hash)

          this.once('message-acknowledged:' + u8aToHex(packetKey), () => {
            resolve()
          })

          try {
            await this.forward.interact(path[0], packet)
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
      await this.heartbeat.pingNode(destination)
      return { latency: Date.now() - start, info: '' }
    } catch (e) {
      //TODO
      return { latency: -1, info: 'error' }
    }
  }

  public getConnectedPeers(): PeerId[] {
    return this.networkPeers.all()
  }

  public connectionReport(): string {
    return this.networkPeers.debugLog()
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
      await this.tickChannelStrategy([])
      await this.announce(this.options.announce)
    } catch (e) {
      log('error in periodic check', e)
    }
    this.checkTimeout = setTimeout(() => this.periodicCheck(), CHECK_TIMEOUT)
  }

  private async announce(includeRouting: boolean = false): Promise<void> {
    const chain = await this.paymentChannels
    const account = await chain.getAccount(await this.getEthereumAddress())
    // exit if we already announced
    if (account.hasAnnounced()) return

    // exit if we don't have enough ETH
    const nativeBalance = await this.getNativeBalance()
    if (nativeBalance.toBN().lte(MIN_NATIVE_BALANCE)) return

    const multiaddrs = await this.getAnnouncedAddresses()
    const ip4 = multiaddrs.find((s) => s.toString().includes('/ip4/'))
    const ip6 = multiaddrs.find((s) => s.toString().includes('/ip6/'))
    const p2p = multiaddrs.find((s) => s.toString().includes('/p2p/'))
    // exit if none of these multiaddrs are available
    if (!ip4 && !ip6 && !p2p) return

    try {
      if (includeRouting && (ip4 || ip6 || p2p)) {
        log('announcing with routing', ip4 || ip6 || p2p)
        await chain.announce(ip4 || ip6 || p2p)
      } else if (!includeRouting && p2p) {
        log('announcing without routing')
        await chain.announce(p2p)
      }
    } catch (err) {
      log('announce failed')
      throw new Error(`Failed to announce: ${err}`)
    }
  }

  public setChannelStrategy(strategy: ChannelStrategyNames) {
    if (strategy == 'passive') {
      this.strategy = new PassiveStrategy()
      return
    }
    if (strategy == 'promiscuous') {
      this.strategy = new PromiscuousStrategy()
      return
    }
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

  public async smartContractInfo(): Promise<string> {
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
    const myAvailableTokens = await ethereum.getBalance(false)

    // validate 'amountToFund'
    if (amountToFund.lten(0)) {
      throw Error(`Invalid 'amountToFund' provided: ${amountToFund.toString(10)}`)
    } else if (amountToFund.gt(myAvailableTokens.toBN())) {
      throw Error(`You don't have enough tokens: ${amountToFund.toString(10)}<${myAvailableTokens.toBN().toString(10)}`)
    }

    const channel = ethereum.getChannel(selfPubKey, counterpartyPubKey)
    await channel.open(new Balance(amountToFund))

    return {
      channelId: await channel.getId()
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
      channelId: channel.getId()
    }
  }

  public async closeChannel(counterparty: PeerId): Promise<{ receipt: string; status: string }> {
    const ethereum = await this.paymentChannels
    const selfPubKey = new PublicKey(this.getId().pubKey.marshal())
    const counterpartyPubKey = new PublicKey(counterparty.pubKey.marshal())
    const channel = ethereum.getChannel(selfPubKey, counterpartyPubKey)
    const channelState = await channel.getState()

    // TODO: should we wait for confirmation?
    if (channelState.status === 'CLOSED') {
      throw new Error('Channel is already closed')
    }

    const txHash = await (channelState.status === 'OPEN' ? channel.initializeClosure() : channel.finalizeClosure())

    return { receipt: txHash, status: channelState.status }
  }

  public async getAcknowledgedTickets() {
    return this.db.getAcknowledgements()
  }

  public async submitAcknowledgedTicket(ackTicket: Acknowledgement, index: Uint8Array) {
    return this.db.submitAcknowledgedTicket(await this.paymentChannels, ackTicket, index)
  }

  public async getChannelsOf(addr: Address): Promise<ChannelEntry[]> {
    const ethereum = await this.paymentChannels
    return await ethereum.getChannelsOf(addr)
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
  private async getIntermediateNodes(destination: PeerId): Promise<PeerId[]> {
    const ethereum = await this.paymentChannels
    return await findPath(
      this.getId(),
      destination,
      MAX_HOPS - 1,
      this.networkPeers,
      ethereum.getChannelsFromPeer.bind(this.paymentChannels),
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
}

export { Hopr as default, LibP2P }
export * from './constants'
