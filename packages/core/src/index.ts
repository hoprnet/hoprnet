/// <reference path="./@types/libp2p.d.ts" />
import LibP2P from 'libp2p' // @ts-ignore
import MPLEX = require('libp2p-mplex')
// @ts-ignore
import KadDHT = require('libp2p-kad-dht')
// @ts-ignore
import SECIO = require('libp2p-secio')

import TCP from './network/transport'

import { Packet } from './messages/packet'
import { PACKET_SIZE, MAX_HOPS, VERSION } from './constants'

import { Network } from './network'

import { addPubKey, getPeerInfo, pubKeyToPeerId } from './utils'
import { createDirectoryIfNotExists, u8aToHex } from '@hoprnet/hopr-utils'
import { existsSync } from 'fs'

import levelup, { LevelUp } from 'levelup'
import leveldown from 'leveldown'
import Multiaddr from 'multiaddr'
import chalk from 'chalk'

import Debug from 'debug'
const log = Debug(`hopr-core`)

import PeerId from 'peer-id'
import PeerInfo from 'peer-info'

import { Handler } from './network/transport/types'

import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type { HoprCoreConnectorStatic, Types } from '@hoprnet/hopr-core-connector-interface'
import HoprCoreEthereum from '@hoprnet/hopr-core-ethereum'

import { Interactions } from './interactions'
import * as DbKeys from './dbKeys'

const verbose = Debug('hopr-core:verbose')

interface NetOptions {
  ip: string
  port: number
}

export type HoprOptions = {
  debug: boolean
  db?: LevelUp
  dbPath?: string
  peerId?: PeerId
  peerInfo?: PeerInfo
  password?: string
  id?: number
  bootstrapNode?: boolean
  network: string
  connector?: HoprCoreConnectorStatic
  bootstrapServers?: PeerInfo[]
  provider: string
  output?: (encoded: Uint8Array) => void
  hosts?: {
    ip4?: NetOptions
    ip6?: NetOptions
  }
}

const MAX_ITERATIONS_PATH_SELECTION = 2000

class Hopr<Chain extends HoprCoreConnector> extends LibP2P {
  public interactions: Interactions<Chain>
  public network: Network<Chain>
  public dbKeys = DbKeys
  public output: (arr: Uint8Array) => void
  public isBootstrapNode: boolean
  public bootstrapServers: PeerInfo[]
  public initializedWithOptions: HoprOptions

  // Allows us to construct HOPR with falsy options
  public _debug: boolean

  /**
   * @constructor
   *
   * @param _options
   * @param provider
   */
  constructor(options: HoprOptions, public db: LevelUp, public paymentChannels: Chain) {
    super({
      peerInfo: options.peerInfo,

      // Disable libp2p-switch protections for the moment
      switch: {
        denyTTL: 1,
        denyAttempts: Infinity,
      },
      // The libp2p modules for this libp2p bundle
      modules: {
        transport: [TCP],
        streamMuxer: [MPLEX],
        connEncryption: [SECIO],
        dht: KadDHT,
      },
      config: {
        transport: {
          TCP: {
            bootstrapServers: options.bootstrapServers,
          },
        },
        dht: {
          enabled: true,
        },
        relay: {
          enabled: false,
        },
      },
    })

    this.initializedWithOptions = options
    this.output = options.output || console.log
    this.bootstrapServers = options.bootstrapServers || []
    this.isBootstrapNode = options.bootstrapNode || false

    this.interactions = new Interactions(this)
    this.network = new Network(this, options)

    verbose('# STARTED NODE')
    verbose('ID', this.peerInfo.id.toB58String())
    verbose('Protocol version', VERSION)
    this._debug = options.debug
  }

  /**
   * Creates a new node.
   *
   * @param options the parameters
   */
  static async create<CoreConnector extends HoprCoreConnector>(options: HoprOptions): Promise<Hopr<CoreConnector>> {
    const Connector = options.connector ?? HoprCoreEthereum
    const db = Hopr.openDatabase(options, Connector.constants.CHAIN_NAME, Connector.constants.NETWORK)

    options.peerInfo = options.peerInfo || (await getPeerInfo(options, db))

    if (
      !options.debug &&
      !options.bootstrapNode &&
      (options.bootstrapServers == null || options.bootstrapServers.length == 0)
    ) {
      throw Error(`Cannot start node without a bootstrap server`)
    }

    let connector = (await Connector.create(db, options.peerInfo.id.privKey.marshal(), {
      provider: options.provider,
      debug: options.debug,
    })) as CoreConnector

    return await new Hopr<CoreConnector>(options, db, connector).start()
  }

  /**
   * Parses the bootstrap servers given in `.env` and tries to connect to each of them.
   *
   * @throws an error if none of the bootstrapservers is online
   */
  async connectToBootstrapServers(): Promise<void> {
    const potentialBootstrapServers = this.bootstrapServers.filter(
      (addr: PeerInfo) => !addr.id.equals(this.peerInfo.id)
    )

    if (potentialBootstrapServers.length == 0) {
      if (this._debug != true && !this.isBootstrapNode) {
        throw Error(
          `Can't start HOPR without any known bootstrap server. You might want to start this node as a bootstrap server.`
        )
      }

      return
    }

    const results = await Promise.all(
      potentialBootstrapServers.map((addr: PeerInfo) =>
        this.dial(addr).then(
          () => true,
          () => false
        )
      )
    )

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
  async start(): Promise<Hopr<Chain>> {
    await Promise.all([
      super.start().then(() =>
        Promise.all([
          // prettier-ignore
          this.connectToBootstrapServers(),
          this.network.start(),
        ])
      ),
      this.paymentChannels?.start(),
    ])

    log(`Available under the following addresses:`)

    this.peerInfo.multiaddrs.forEach((ma: Multiaddr) => log(ma.toString()))

    return this
  }

  async down(): Promise<void> {
    log('DEPRECATED use stop() not down()')
    return this.stop()
  }

  /**
   * Shuts down the node and saves keys and peerBook in the database
   */
  async stop(): Promise<void> {
    await Promise.all([
      // prettier-ignore
      this.network.stop(),
      this.paymentChannels?.stop().then(() => log(`Connector stopped.`)),
    ])

    await Promise.all([
      // prettier-ignore
      this.db?.close().then(() => log(`Database closed.`)),
      super.stop(),
    ])

    // Give the operating system some extra time to close the sockets
    await new Promise((resolve) => setTimeout(resolve, 100))
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
  async sendMessage(
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
              /* prettier-ignore */
              this,
              msg.slice(n * PACKET_SIZE, Math.min(msg.length, (n + 1) * PACKET_SIZE)),
              await Promise.all(path.map(addPubKey))
            )
          } catch (err) {
            return reject(err)
          }

          const unAcknowledgedDBKey = this.dbKeys.UnAcknowledgedTickets(packet.challenge.hash)

          await this.db.put(Buffer.from(unAcknowledgedDBKey), Buffer.from(''))

          this.interactions.packet.acknowledgment.once(u8aToHex(unAcknowledgedDBKey), () => {
            resolve()
          })

          try {
            await this.interactions.packet.forward.interact(path[0], packet)
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
      console.trace(err)
      throw err
    }
  }

  /**
   * Ping a node.
   *
   * @param destination PeerId of the node
   * @returns latency
   */
  async ping(destination: PeerId): Promise<number> {
    if (!PeerId.isPeerId(destination)) {
      throw Error(`Expecting a non-empty destination.`)
    }

    const start = Date.now()
    try {
      await this.interactions.network.heartbeat.interact(destination)
      return Date.now() - start
    } catch (err) {
      throw Error(`node unreachable`)
    }
  }

  /**
   * Takes a destination and samples randomly intermediate nodes
   * that will relay that message before it reaches its destination.
   *
   * @param destination instance of peerInfo that contains the peerId of the destination
   */
  async getIntermediateNodes(destination: PeerId): Promise<PeerId[]> {
    const start = new this.paymentChannels.types.Public(this.peerInfo.id.pubKey.marshal())
    const exclude = [
      destination.pubKey.marshal(),
      ...this.bootstrapServers.map((pInfo) => pInfo.id.pubKey.marshal()),
    ].map((pubKey) => new this.paymentChannels.types.Public(pubKey))

    return await Promise.all(
      (
        await this.paymentChannels.path.findPath(
          start,
          MAX_HOPS - 1, // Need a hop for destination node
          MAX_ITERATIONS_PATH_SELECTION,
          (node) => !exclude.includes(node)
        )
      ).map((pubKey) => pubKeyToPeerId(pubKey))
    )
  }

  static openDatabase(options: HoprOptions, chainName: string, network: string): LevelUp {
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

    verbose('using db at ', dbPath)
    if (!existsSync(dbPath)) {
      verbose('db does not exist, creating')
    }
    createDirectoryIfNotExists(dbPath)
    return levelup(leveldown(dbPath))
  }

  /**
   * Get all acknowledged tickets
   * @returns an array of all acknowledged tickets
   */
  public async getAcknowledgedTickets(): Promise<
    {
      ackTicket: Types.AcknowledgedTicket
      index: Uint8Array
    }[]
  > {
    const { AcknowledgedTicket } = this.paymentChannels.types
    const acknowledgedTicketSize = AcknowledgedTicket.SIZE(this.paymentChannels)
    let promises: {
      ackTicket: Types.AcknowledgedTicket
      index: Uint8Array
    }[] = []

    return new Promise((resolve, reject) => {
      this.db
        .createReadStream({
          gte: Buffer.from(this.dbKeys.AcknowledgedTickets(new Uint8Array(0x00))),
        })
        .on('error', (err) => reject(err))
        .on('data', ({ key, value }: { key: Buffer; value: Buffer }) => {
          if (value.buffer.byteLength !== acknowledgedTicketSize) return

          const index = this.dbKeys.AcknowledgedTicketsParse(key)
          const ackTicket = AcknowledgedTicket.create(this.paymentChannels, {
            bytes: value.buffer,
            offset: value.byteOffset,
          })

          promises.push({
            ackTicket,
            index,
          })
        })
        .on('end', () => resolve(Promise.all(promises)))
    })
  }

  /**
   * Update Acknowledged Ticket in database
   * @param ackTicket Uint8Array
   * @param index Uint8Array
   */
  public async updateAcknowledgedTicket(ackTicket: Types.AcknowledgedTicket, index: Uint8Array): Promise<void> {
    await this.db.put(Buffer.from(this.dbKeys.AcknowledgedTickets(index)), Buffer.from(ackTicket))
  }

  /**
   * Delete Acknowledged Ticket in database
   * @param index Uint8Array
   */
  public async deleteAcknowledgedTicket(index: Uint8Array): Promise<void> {
    await this.db.del(Buffer.from(this.dbKeys.AcknowledgedTickets(index)))
  }

  /**
   * Submit Acknowledged Ticket and update database
   * @param ackTicket Uint8Array
   * @param index Uint8Array
   */
  public async submitAcknowledgedTicket(
    ackTicket: Types.AcknowledgedTicket,
    index: Uint8Array
  ): Promise<
    | {
        status: 'SUCCESS'
        receipt: string
      }
    | {
        status: 'FAILURE'
        message: string
      }
    | {
        status: 'ERROR'
        error: Error | string
      }
  > {
    try {
      const result = await this.paymentChannels.channel.tickets.submit(ackTicket, index)

      if (result.status === 'SUCCESS') {
        ackTicket.redeemed = true
        await this.updateAcknowledgedTicket(ackTicket, index)
      } else if (result.status === 'FAILURE') {
        await this.deleteAcknowledgedTicket(index)
      } else if (result.status === 'ERROR') {
        // await this.deleteAcknowledgedTicket(index)
        // @TODO: better handle this
      }

      return result
    } catch (err) {
      return {
        status: 'ERROR',
        error: err,
      }
    }
  }
}

export { Hopr as default, LibP2P }
