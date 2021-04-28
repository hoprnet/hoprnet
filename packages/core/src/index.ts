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
  TICKET_AMOUNT,
  TICKET_WIN_PROB,
  PATH_RANDOMNESS,
  MIN_NATIVE_BALANCE,
  FULL_VERSION
} from './constants'

import NetworkPeers from './network/network-peers'
import Heartbeat from './network/heartbeat'
import { findPath } from './path'

import { getAcknowledgements, submitAcknowledgedTicket } from './utils'
import { u8aToHex, DialOpts } from '@hoprnet/hopr-utils'
import { existsSync, mkdirSync } from 'fs'
import getIdentity from './identity'

import levelup, { LevelUp } from 'levelup'
import leveldown from 'leveldown'
import Multiaddr from 'multiaddr'
import chalk from 'chalk'

import PeerId from 'peer-id'
import HoprCoreEthereum, {
  PublicKey,
  Balance,
  NativeBalance,
  Hash,
  Acknowledgement,
  RoutingChannel
} from '@hoprnet/hopr-core-ethereum'
import BN from 'bn.js'

import * as DbKeys from './dbKeys'
import EventEmitter from 'events'
import path from 'path'
import { ChannelStrategy, PassiveStrategy, PromiscuousStrategy } from './channel-strategy'
import Debug from 'debug'
import { Address } from 'libp2p/src/peer-store'
import {
  libp2pSendMessageAndExpectResponse,
  libp2pSubscribe,
  libp2pSendMessage,
  LibP2PHandlerFunction
} from '@hoprnet/hopr-utils'
import { subscribeToAcknowledgements } from './interactions/packet/acknowledgement'
import { PacketForwardInteraction } from './interactions/packet/forward'

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
  db?: LevelUp
  dbPath?: string
  createDbIfNotExist?: boolean
  peerId?: PeerId
  password?: string
  connector?: HoprCoreEthereum
  strategy?: ChannelStrategyNames
  hosts?: {
    ip4?: NetOptions
    ip6?: NetOptions
  }
}

const defaultDBPath = (): string => {
  return path.join(process.cwd(), 'db', VERSION, 'node')
}

class Hopr extends EventEmitter {
  // TODO make these actually private - Do not rely on any of these properties!
  // Allows us to construct HOPR with falsy options
  public _dbKeys = DbKeys

  public initializedWithOptions: HoprOptions
  public ticketAmount: string = TICKET_AMOUNT
  public ticketWinProb: number = TICKET_WIN_PROB

  private running: boolean
  private checkTimeout: NodeJS.Timeout
  private strategy: ChannelStrategy
  private networkPeers: NetworkPeers
  private heartbeat: Heartbeat
  private forward: PacketForwardInteraction
  private announcing = false

  /**
   * @constructor
   *
   * @param _options
   * @param provider
   */
  private constructor(
    options: HoprOptions,
    private libp2p: LibP2P,
    public db: LevelUp,
    public paymentChannels: HoprCoreEthereum
  ) {
    super()

    this.libp2p.connectionManager.on('peer:connect', (conn: Connection) => {
      this.emit('hopr:peer:connection', conn.remotePeer)
      this.networkPeers.register(conn.remotePeer)
    })

    this.setChannelStrategy(options.strategy || 'passive')
    this.initializedWithOptions = options

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

    this.networkPeers = new NetworkPeers(Array.from(this.libp2p.peerStore.peers.values()).map((x) => x.id))

    const subscribe = (protocol: string, handler: LibP2PHandlerFunction, includeReply = false) =>
      libp2pSubscribe(this.libp2p, protocol, handler, includeReply)
    const sendMessageAndExpectResponse = (dest: PeerId, protocol: string, msg: Uint8Array, opts: DialOpts) =>
      libp2pSendMessageAndExpectResponse(this.libp2p, dest, protocol, msg, opts)
    const sendMessage = (dest: PeerId, protocol: string, msg: Uint8Array, opts: DialOpts) =>
      libp2pSendMessage(this.libp2p, dest, protocol, msg, opts)
    const hangup = this.libp2p.hangUp.bind(this.libp2p)

    this.heartbeat = new Heartbeat(this.networkPeers, subscribe, sendMessageAndExpectResponse, hangup)

    subscribeToAcknowledgements(subscribe, this.db, this.paymentChannels, (ack) =>
      this.emit('message-acknowledged:' + ack.getKey())
    )
    this.forward = new PacketForwardInteraction(this, this.libp2p, subscribe, sendMessage)

    if (options.ticketAmount) this.ticketAmount = String(options.ticketAmount)
    if (options.ticketWinProb) this.ticketWinProb = options.ticketWinProb

    verbose('# STARTED NODE')
    verbose('ID', this.getId().toB58String())
    verbose('Protocol version', VERSION)
  }

  /**
   * Creates a new node
   * This is necessary as some of the constructor for the node needs to be
   * asynchronous..
   *
   * @param options the parameters
   */
  public static async create(options: HoprOptions): Promise<Hopr> {
    const db = Hopr.openDatabase(options)

    const { id, addresses } = await getIdentity({
      ...options,
      db
    })

    let connector = await HoprCoreEthereum.create(db, id.privKey.marshal(), {
      provider: options.provider
    })

    verbose('Started HoprEthereum. Waiting for indexer to find connected nodes.')
    const publicNodes = await connector.waitForPublicNodes()
    if (publicNodes.length == 0) {
      log('no nodes have announced yet, we cannot rely on relay')
    }

    const libp2p = await LibP2P.create({
      peerId: id,
      addresses: { listen: addresses.map((x) => x.toString()) },
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
    const node = new Hopr(options, libp2p, db, connector)
    await node.heartbeat.start()
    log(`Available under the following addresses:`)
    libp2p.multiaddrs.forEach((ma: Multiaddr) => log(ma.toString()))
    node.periodicCheck()
    return node
  }

  private async tickChannelStrategy(newChannels: RoutingChannel[]) {
    verbose('new payment channels, auto opening tick', this.running)
    if (!this.running) {
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
    const [nextChannels, closeChannels] = await this.strategy.tick(
      balance.toBN(),
      newChannels,
      currentChannels,
      this.networkPeers,
      this.paymentChannels.getRandomChannel.bind(this.paymentChannels)
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
    return this.paymentChannels.getChannelsFromPeer(this.getId())
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
    if (!this.running) {
      return Promise.resolve()
    }
    clearTimeout(this.checkTimeout)
    this.running = false
    await Promise.all([this.heartbeat.stop(), this.paymentChannels.stop()])

    await Promise.all([this.db?.close().then(() => log(`Database closed.`)), this.libp2p.stop()])

    // Give the operating system some extra time to close the sockets
    await new Promise((resolve) => setTimeout(resolve, 100))
  }

  public isRunning(): boolean {
    return this.running
  }

  public getId(): PeerId {
    return this.libp2p.peerId // Not a documented API, but in the source
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
  public getObservedAddresses(peer: PeerId): Address[] {
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
              this,
              this.libp2p,
              msg.slice(n * PACKET_SIZE, Math.min(msg.length, (n + 1) * PACKET_SIZE)),
              path
            )
          } catch (err) {
            return reject(err)
          }

          const unAcknowledgedDBKey = this._dbKeys.UnAcknowledgedTickets(packet.challenge.hash.serialize())

          await this.db.put(Buffer.from(unAcknowledgedDBKey), Buffer.from(''))

          this.once('message-acknowledged:' + u8aToHex(unAcknowledgedDBKey), () => {
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
    if (balance.toBN().lten(0)) {
      const address = await this.paymentChannels.hexAccountAddress()
      log('unfunded node', address)
      this.emit('hopr:warning:unfunded', address)
    }
    const nativeBalance = await this.getNativeBalance()
    if (nativeBalance.toBN().lte(MIN_NATIVE_BALANCE)) {
      const address = await this.paymentChannels.hexAccountAddress()
      log('unfunded node', address)
      this.emit('hopr:warning:unfundedNative', address)
    }
  }

  private async periodicCheck() {
    log('periodic check', this.running)
    if (!this.running) {
      return
    }
    try {
      await this.checkBalances()
      await this.tickChannelStrategy([])
      await this.announce(this.initializedWithOptions.announce)
    } catch (e) {
      log('error in periodic check', e)
    }
    this.checkTimeout = setTimeout(() => this.periodicCheck(), CHECK_TIMEOUT)
  }

  private async announce(includeRouting: boolean = false): Promise<void> {
    // exit if we are already announcing
    if (this.announcing) return

    const account = await this.paymentChannels.getAccount(this.paymentChannels.getAddress())
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
      this.announcing = true

      if (includeRouting && (ip4 || ip6 || p2p)) {
        await this.paymentChannels.announce(ip4 || ip6 || p2p)
      } else if (!includeRouting && p2p) {
        await this.paymentChannels.announce(p2p)
      }

      this.announcing = false
    } catch (err) {
      this.announcing = false
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
    return await this.paymentChannels.getBalance(true)
  }

  public async getNativeBalance(): Promise<NativeBalance> {
    return await this.paymentChannels.getNativeBalance(true)
  }

  public smartContractInfo(): string {
    return this.paymentChannels.smartContractInfo()
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
    const ethereum = this.paymentChannels
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
    const ethereum = this.paymentChannels
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
    return getAcknowledgements(this.db)
  }

  public async submitAcknowledgedTicket(ackTicket: Acknowledgement, index: Uint8Array) {
    return submitAcknowledgedTicket(this, ackTicket, index)
  }

  /**
   * Takes a destination and samples randomly intermediate nodes
   * that will relay that message before it reaches its destination.
   *
   * @param destination instance of peerInfo that contains the peerId of the destination
   */
  private async getIntermediateNodes(destination: PeerId): Promise<PeerId[]> {
    return await findPath(
      this.getId(),
      destination,
      MAX_HOPS - 1,
      this.networkPeers,
      this.paymentChannels.getChannelsFromPeer.bind(this.paymentChannels),
      PATH_RANDOMNESS
    )
  }

  private static openDatabase(options: HoprOptions): LevelUp {
    if (options.db) {
      return options.db
    }

    let dbPath: string
    if (options.dbPath) {
      dbPath = options.dbPath
    } else {
      dbPath = defaultDBPath()
    }

    dbPath = path.resolve(dbPath)

    verbose('using db at ', dbPath)
    if (!existsSync(dbPath)) {
      verbose('db does not exist, creating?:', options.createDbIfNotExist)
      if (options.createDbIfNotExist) {
        mkdirSync(dbPath, { recursive: true })
      } else {
        throw new Error('Database does not exist: ' + dbPath)
      }
    }

    return levelup(leveldown(dbPath))
  }
}

export { Hopr as default, LibP2P }
export * from './constants'
