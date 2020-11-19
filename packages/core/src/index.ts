/// <reference path="./@types/libp2p.ts" />
import LibP2P from 'libp2p'
import type { Connection } from 'libp2p'
import MPLEX = require('libp2p-mplex')
// @ts-ignore
import KadDHT = require('libp2p-kad-dht')
// @ts-ignore
import SECIO = require('libp2p-secio')

import TCP from './network/transport'

import { Packet } from './messages/packet'
import {
  PACKET_SIZE,
  MAX_HOPS,
  VERSION,
  CRAWL_TIMEOUT,
  TICKET_AMOUNT,
  TICKET_WIN_PROB,
  PATH_RANDOMNESS
} from './constants'

import { Network } from './network'
import { findPath } from './path'

import { addPubKey, getPeerId, getAddrs, getAcknowledgedTickets, submitAcknowledgedTicket } from './utils'
import { createDirectoryIfNotExists, u8aToHex, pubKeyToPeerId } from '@hoprnet/hopr-utils'
import { existsSync } from 'fs'

import levelup, { LevelUp } from 'levelup'
import leveldown from 'leveldown'
import Multiaddr from 'multiaddr'
import chalk from 'chalk'

import Debug from 'debug'
const log = Debug(`hopr-core`)

import PeerId from 'peer-id'
import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type { HoprCoreConnectorStatic, Types } from '@hoprnet/hopr-core-connector-interface'
import type { CrawlInfo } from './network/crawler'
import HoprCoreEthereum from '@hoprnet/hopr-core-ethereum'
import BN from 'bn.js'

import { Interactions } from './interactions'
import * as DbKeys from './dbKeys'
import EventEmitter from 'events'
import path from 'path'
import { ChannelStrategy, PassiveStrategy, PromiscuousStrategy } from './channel-strategy'
import { Mixer } from './mixer'

const verbose = Debug('hopr-core:verbose')

interface NetOptions {
  ip: string
  port: number
}

export type ChannelStrategyNames = 'PASSIVE' | 'PROMISCUOUS'

export type HoprOptions = {
  debug: boolean
  network: string
  provider: string
  ticketAmount?: number
  ticketWinProb?: number
  db?: LevelUp
  dbPath?: string
  createDbIfNotExist?: boolean
  peerId?: PeerId
  password?: string
  id?: number // TODO - kill this opaque accessor of db files...
  bootstrapNode?: boolean
  connector?: HoprCoreConnectorStatic
  bootstrapServers?: Multiaddr[]
  output?: (encoded: Uint8Array) => void
  strategy?: ChannelStrategyNames
  hosts?: {
    ip4?: NetOptions
    ip6?: NetOptions
  }
}

class Hopr<Chain extends HoprCoreConnector> extends EventEmitter {
  // TODO make these actually private - Do not rely on any of these properties!
  public _interactions: Interactions<Chain>
  public _network: Network
  // Allows us to construct HOPR with falsy options
  public _debug: boolean
  public _dbKeys = DbKeys

  public output: (arr: Uint8Array) => void
  public isBootstrapNode: boolean
  public bootstrapServers: Multiaddr[]
  public initializedWithOptions: HoprOptions
  public ticketAmount: number = TICKET_AMOUNT
  public ticketWinProb: number = TICKET_WIN_PROB

  private running: boolean
  private crawlTimeout: NodeJS.Timeout
  private mixer: Mixer<Chain>
  private strategy: ChannelStrategy

  /**
   * @constructor
   *
   * @param _options
   * @param provider
   */
  private constructor(options: HoprOptions, public _libp2p: LibP2P, public db: LevelUp, public paymentChannels: Chain) {
    super()
    this._libp2p.connectionManager.on('peer:connect', (conn: Connection) => {
      this.emit('hopr:peer:connection', conn.remotePeer)
      this._network.networkPeers.register(conn.remotePeer)
    })

    this.mixer = new Mixer()
    this.setChannelStrategy(options.strategy || 'PASSIVE')
    this.initializedWithOptions = options
    this.output = (arr: Uint8Array) => {
      this.emit('hopr:message', arr)
      if (options.output) {
        log('DEPRECATED: options.output is replaced with a hopr:message event')
        options.output(arr)
      }
    }
    this.bootstrapServers = options.bootstrapServers || []
    this.isBootstrapNode = options.bootstrapNode || false
    this._interactions = new Interactions(
      this,
      this.mixer,
      (conn: Connection) => this._network.crawler.handleCrawlRequest(conn),
      (remotePeer: PeerId) => this._network.heartbeat.emit('beat', remotePeer)
    )
    this._network = new Network(this._libp2p, this._interactions, options)

    if (options.ticketAmount) this.ticketAmount = options.ticketAmount
    if (options.ticketWinProb) this.ticketWinProb = options.ticketWinProb

    verbose('# STARTED NODE')
    verbose('ID', this.getId().toB58String())
    verbose('Protocol version', VERSION)
    this._debug = options.debug
  }

  /**
   * Creates a new node
   * This is necessary as some of the constructor for the node needs to be
   * asynchronous..
   *
   * @param options the parameters
   */
  public static async create<CoreConnector extends HoprCoreConnector>(
    options: HoprOptions
  ): Promise<Hopr<CoreConnector>> {
    const Connector = options.connector ?? HoprCoreEthereum
    const db = Hopr.openDatabase(options, Connector.constants.CHAIN_NAME, Connector.constants.NETWORK)
    const id = await getPeerId(options, db)
    const addresses = await getAddrs(id, options)

    if (
      !options.debug &&
      !options.bootstrapNode &&
      (options.bootstrapServers == null || options.bootstrapServers.length == 0)
    ) {
      throw Error(`Cannot start node without a bootstrap server`)
    }

    let connector = (await Connector.create(db, id.privKey.marshal(), {
      provider: options.provider,
      debug: options.debug
    })) as CoreConnector

    verbose('Created connector, now creating node')

    const libp2p = await LibP2P.create({
      peerId: id,
      addresses: { listen: addresses },
      // Disable libp2p-switch protections for the moment
      switch: {
        denyTTL: 1,
        denyAttempts: Infinity
      },
      // The libp2p modules for this libp2p bundle
      modules: {
        transport: [TCP],
        streamMuxer: [MPLEX],
        connEncryption: [SECIO],
        dht: KadDHT
      },
      config: {
        transport: {
          TCP: {
            bootstrapServers: options.bootstrapServers
          }
        },
        peerDiscovery: {
          autoDial: false
        },
        dht: {
          enabled: true
        },
        relay: {
          enabled: false
        }
      }
    })

    return await new Hopr<CoreConnector>(options, libp2p, db, connector).start()
  }

  /**
   * Parses the bootstrap servers given in options` and tries to connect to each of them.
   *
   * @throws an error if none of the bootstrapservers is online
   */
  private async connectToBootstrapServers(): Promise<void> {
    const potentialBootstrapServers = this.bootstrapServers.filter(
      (addr) => addr.getPeerId() != this.getId().toB58String()
    )
    verbose('bootstrap', potentialBootstrapServers)

    if (potentialBootstrapServers.length == 0) {
      if (this._debug != true && !this.isBootstrapNode) {
        throw Error(
          `Can't start HOPR without any known bootstrap server. You might want to start this node as a bootstrap server.`
        )
      }

      return
    }

    const results = await Promise.all(
      potentialBootstrapServers.map((addr: Multiaddr) =>
        this._libp2p.dial(addr).then(
          () => true,
          () => false
        )
      )
    )
    verbose('bootstrap status', results)

    if (!results.some((online: boolean) => online)) {
      throw Error('Unable to connect to any known bootstrap server.')
    }
  }

  /**
   * This method starts the node and registers all necessary handlers. It will
   * also open the database and creates one if it doesn't exists.
   *
   * @param options
   */
  public async start(): Promise<Hopr<Chain>> {
    await Promise.all([
      this._libp2p.start().then(() => Promise.all([this.connectToBootstrapServers(), this._network.start()])),
      this.paymentChannels?.start()
    ])

    this.paymentChannels.indexer.onNewChannels(async () => {
      //TODO async(newChannels) => {
      verbose('new payment channels, auto opening tick')
      //TODO this._network.networkPeers.addInterestingPeer(newPeer)
      //TODO let currentChannels = this.getOpenChannels()
      const balance = await this.getBalance()
      const nextChannels = await this.strategy.tick(balance, this.paymentChannels.indexer)
      verbose('strategy wants to open', nextChannels.length, 'new channels')
      for (let channelToOpen of nextChannels) {
        const hash = await this.openChannel(channelToOpen[0], channelToOpen[1])
        verbose('- opened', hash)
      }
    })

    log(`Available under the following addresses:`)

    this._libp2p.multiaddrs.forEach((ma: Multiaddr) => log(ma.toString()))
    await this.periodicCrawl()
    this.running = true
    return this
  }

  /**
   * Shuts down the node and saves keys and peerBook in the database
   */
  public async stop(): Promise<void> {
    if (!this.running) {
      return Promise.resolve()
    }
    clearTimeout(this.crawlTimeout)
    this.running = false
    await Promise.all([this._network.stop(), this.paymentChannels?.stop().then(() => log(`Connector stopped.`))])

    await Promise.all([this.db?.close().then(() => log(`Database closed.`)), this._libp2p.stop()])

    // Give the operating system some extra time to close the sockets
    await new Promise((resolve) => setTimeout(resolve, 100))
  }

  public isRunning(): boolean {
    return this.running
  }

  public getId(): PeerId {
    return this._libp2p.peerId // Not a documented API, but in the sourceu
  }

  /*
   * List the addresses the node is available on
   */
  public getAddresses(): Multiaddr[] {
    return this._libp2p.multiaddrs
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
          let path: PeerId[]
          if (getIntermediateNodesManually != undefined) {
            verbose('manually creating path')
            path = await getIntermediateNodesManually()
          } else {
            path = await this.getIntermediateNodes(destination)
          }

          path.push(destination)

          let packet: Packet<Chain>
          verbose('creating packet with path', path.join(', \n'))
          try {
            packet = await Packet.create(
              this,
              this._libp2p,
              msg.slice(n * PACKET_SIZE, Math.min(msg.length, (n + 1) * PACKET_SIZE)),
              await Promise.all(path.map(addPubKey))
            )
          } catch (err) {
            return reject(err)
          }

          const unAcknowledgedDBKey = this._dbKeys.UnAcknowledgedTickets(packet.challenge.hash)

          await this.db.put(Buffer.from(unAcknowledgedDBKey), Buffer.from(''))

          this._interactions.packet.acknowledgment.once(u8aToHex(unAcknowledgedDBKey), () => {
            resolve()
          })

          try {
            await this._interactions.packet.forward.interact(path[0], packet)
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
   *
   * @param destination PeerId of the node
   * @returns latency
   */
  public async ping(destination: PeerId): Promise<{ info: string; latency: number }> {
    if (!PeerId.isPeerId(destination)) {
      throw Error(`Expecting a non-empty destination.`)
    }
    let info = ''
    let latency = await this._interactions.network.heartbeat.interact(destination)
    return { latency, info }
  }

  public getConnectedPeers(): PeerId[] {
    return this._network.networkPeers.all()
  }

  public async crawl(filter?: (peer: PeerId) => boolean): Promise<CrawlInfo> {
    return this._network.crawler.crawl(filter)
  }

  private async periodicCrawl() {
    let crawlInfo = await this.crawl()
    this.emit('hopr:crawl:completed', crawlInfo)
    this.crawlTimeout = setTimeout(() => this.periodicCrawl(), CRAWL_TIMEOUT)
  }

  public setChannelStrategy(strategy: ChannelStrategyNames) {
    if (strategy == 'PASSIVE') {
      this.strategy = new PassiveStrategy()
    }
    if (strategy == 'PROMISCUOUS') {
      this.strategy = new PromiscuousStrategy()
    }
  }

  public async getBalance(): Promise<BN> {
    return await this.paymentChannels.account.balance
  }

  /**
   * Open a payment channel
   *
   * @param counterParty the counter party's peerId
   * @param amountToFund the amount to fund in HOPR(wei)
   */
  public async openChannel(
    counterParty: PeerId,
    amountToFund: BN
  ): Promise<{
    channelId: Types.Hash
  }> {
    const { utils, types, account } = this.paymentChannels
    const self = this.getId()

    const channelId = await utils.getId(
      await utils.pubKeyToAccountId(self.pubKey.marshal()),
      await utils.pubKeyToAccountId(counterParty.pubKey.marshal())
    )

    const myAvailableTokens = await account.balance

    // validate 'amountToFund'
    if (amountToFund.lten(0)) {
      throw Error(`Invalid 'amountToFund' provided: ${amountToFund.toString(10)}`)
    } else if (amountToFund.gt(myAvailableTokens)) {
      throw Error(`You don't have enough tokens: ${amountToFund.toString(10)}<${myAvailableTokens.toString(10)}`)
    }

    const amPartyA = utils.isPartyA(
      await utils.pubKeyToAccountId(self.pubKey.marshal()),
      await utils.pubKeyToAccountId(counterParty.pubKey.marshal())
    )

    const channelBalance = types.ChannelBalance.create(
      undefined,
      amPartyA
        ? {
            balance: amountToFund,
            balance_a: amountToFund
          }
        : {
            balance: amountToFund,
            balance_a: new BN(0)
          }
    )

    await this.paymentChannels.channel.create(
      counterParty.pubKey.marshal(),
      async () => this._interactions.payments.onChainKey.interact(counterParty),
      channelBalance,
      (balance: Types.ChannelBalance): Promise<Types.SignedChannel> =>
        this._interactions.payments.open.interact(counterParty, balance)
    )

    return {
      channelId
    }
  }

  public async closeChannel(peerId: PeerId): Promise<{ receipt: string; status: string }> {
    const channel = await this.paymentChannels.channel.create(
      peerId.pubKey.marshal(),
      async (counterparty: Uint8Array) =>
        this._interactions.payments.onChainKey.interact(await pubKeyToPeerId(counterparty))
    )

    const status = await channel.status

    if (!(status === 'OPEN' || status === 'PENDING')) {
      throw new Error('To close a channel, it must be open or pending for closure')
    }

    const receipt = await channel.initiateSettlement()
    return { receipt, status }
  }

  public async getAcknowledgedTickets() {
    return getAcknowledgedTickets(this)
  }

  public async submitAcknowledgedTicket(ackTicket: Types.AcknowledgedTicket, index: Uint8Array) {
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
      this._network.networkPeers,
      this.paymentChannels.indexer,
      PATH_RANDOMNESS
    )
  }

  private static openDatabase(options: HoprOptions, chainName: string, network: string): LevelUp {
    if (options.db) {
      return options.db
    }

    let dbPath: string
    if (options.dbPath) {
      dbPath = options.dbPath
    } else {
      dbPath = `${process.cwd()}/db/${chainName}/${network}/`
      if (options.bootstrapNode) {
        dbPath += `bootstrap`
      } else if (options.id != null && Number.isInteger(options.id)) {
        dbPath += `node_${options.id}`
      } else {
        dbPath += `node`
      }
    }
    dbPath = path.resolve(dbPath)

    verbose('using db at ', dbPath)
    if (!existsSync(dbPath)) {
      verbose('db does not exist, creating?:', options.createDbIfNotExist)
      if (options.createDbIfNotExist) {
        createDirectoryIfNotExists(dbPath)
      } else {
        throw new Error('Database does not exist: ' + dbPath)
      }
    }
    // @ts-ignore
    return levelup(leveldown(dbPath))
  }
}

export { Hopr as default, LibP2P }
