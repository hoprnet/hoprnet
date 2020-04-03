// @ts-ignore
import libp2p = require('libp2p')
// @ts-ignore
import MPLEX = require('libp2p-mplex')
// @ts-ignore
import KadDHT = require('libp2p-kad-dht')
// @ts-ignore
import SECIO = require('libp2p-secio')
// import { WebRTCv4, WebRTCv6 } = require('./network/natTraversal')
import TCP = require('libp2p-tcp')

// @ts-ignore
import defaultsDeep = require('@nodeutils/defaults-deep')

import { Packet } from './messages/packet'
import { PACKET_SIZE, MAX_HOPS } from './constants'

import { Network } from './network'

import { randomSubset, addPubKey, createDirectoryIfNotExists, getPeerInfo, privKeyToPeerId, u8aToHex } from './utils'

import levelup, { LevelUp } from 'levelup'
import leveldown from 'leveldown'
import Multiaddr from 'multiaddr'
import chalk from 'chalk'
import Debug, { Debugger } from 'debug'

import PeerId from 'peer-id'
import PeerInfo from 'peer-info'

import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import { Interactions, Duplex } from './interactions'
import * as DbKeys from './db_keys'

type HoprOptions = {
  peerInfo: PeerInfo
  output: (msg: Uint8Array) => void
  id?: number
  bootstrapServers?: PeerInfo[]
  bootstrapNode: boolean
}

export default class Hopr<Chain extends HoprCoreConnector> extends libp2p {
  public interactions: Interactions<Chain>
  public network: Network<Chain>
  public log: Debugger
  public dbKeys = DbKeys
  public output: (arr: Uint8Array) => void

  // @TODO put this in proper namespace
  // public heartbeat: any

  // @TODO add libp2p types
  declare dial: (addr: Multiaddr | PeerInfo | PeerId) => Promise<any>
  declare dialProtocol: (addr: Multiaddr | PeerInfo | PeerId, protocol: string) => Promise<{ stream: Duplex; protocol: string }>
  declare peerInfo: PeerInfo
  declare peerStore: {
    has(peerInfo: PeerId): boolean
    put(peerInfo: PeerInfo, options?: { silent: boolean }): PeerInfo
    peers: Map<string, PeerInfo>
    remove(peer: PeerId): void
  }
  declare peerRouting: {
    findPeer: (addr: PeerId) => Promise<PeerInfo>
  }
  declare handle: (protocol: string[], handler: (struct: { stream: any }) => void) => void
  declare start: () => Promise<void>
  declare stop: () => Promise<void>
  declare on: (str: string, handler: (...props: any[]) => void) => void

  /**
   * @constructor
   *
   * @param _options
   * @param provider
   */
  constructor(_options: HoprOptions, public db: LevelUp, public bootstrapServers: PeerInfo[], public paymentChannels: Chain) {
    super(
      defaultsDeep(_options, {
        // Disable libp2p-switch protections for the moment
        switch: {
          denyTTL: 1,
          denyAttempts: Infinity
        },
        // The libp2p modules for this libp2p bundle
        modules: {
          transport: [
            TCP
            // WebRTCv4,
            // WebRTCv6
          ],

          streamMuxer: [MPLEX],
          connEncryption: [SECIO],
          dht: KadDHT
          // peerDiscovery: [
          //     WebRTC.discovery
          // ]
        },
        config: {
          // peerDiscovery: {
          //     webRTCStar: {
          //         enabled: true
          //     }
          // },
          dht: {
            enabled: true
          },
          relay: {
            enabled: false
          }
        }
      })
    )

    this.output = _options.output
    this.interactions = new Interactions(this)
    this.network = new Network(this)

    this.log = Debug(`${chalk.blue(_options.peerInfo.id.toB58String())}: `)
  }

  /**
   * Creates a new node.
   *
   * @param options the parameters
   */
  static async createNode<Constructor extends typeof HoprCoreConnector>(
    HoprCoreConnector: Constructor,
    options?: Partial<HoprOptions> & {
      provider?: string
      peerId?: PeerId
    }
  ): Promise<Hopr<any>> {
    const db = Hopr.openDatabase(`db`, HoprCoreConnector, options)

    options = options || {}

    options.bootstrapNode = options.bootstrapNode || false

    options.output = options.output || console.log

    // @TODO give bootstrap node a different identity
    if (options.bootstrapNode) {
      options.id = 6
    }

    let connector: HoprCoreConnector

    if (options != null && isFinite(options.id)) {
      connector = await HoprCoreConnector.create(db, undefined, options)
      options.peerId = await privKeyToPeerId(connector.self.privateKey)
      if (options.peerInfo != null && !options.peerId.isEqual(options.peerInfo.id)) {
        throw Error(`PeerId and PeerInfo mismatch.`)
      }

      if (options.peerInfo == null) {
        options.peerInfo = await getPeerInfo(options)
      }
    } else {
      if (options.peerInfo == null) {
        options.peerInfo = await getPeerInfo(options, db)
      }

      connector = await HoprCoreConnector.create(db, options.peerInfo.id.privKey.marshal(), options)
    }

    return new Hopr(options as HoprOptions, db, options.bootstrapNode ? null : options.bootstrapServers, connector).up(options as HoprOptions)
  }

  /**
   * Parses the bootstrap servers given in `.env` and tries to connect to each of them.
   *
   * @throws an error if none of the bootstrapservers is online
   */
  async connectToBootstrapServers(): Promise<void> {
    const results = await Promise.all(
      this.bootstrapServers.map(addr =>
        this.dial(addr).then(
          () => true,
          () => false
        )
      )
    )

    if (!results.some(online => online)) {
      throw Error('Unable to connect to any bootstrap server.')
    }
  }

  /**
   * This method starts the node and registers all necessary handlers. It will
   * also open the database and creates one if it doesn't exists.
   *
   * @param options
   */
  async up(options: HoprOptions): Promise<Hopr<Chain>> {
    await super.start()

    if (!options.bootstrapNode) {
      await this.connectToBootstrapServers()
    } else {
      this.log(`Available under the following addresses:`)
      this.peerInfo.multiaddrs.forEach((ma: Multiaddr) => {
        this.log(ma.toString())
      })
    }

    this.network.heartbeat.start()

    // this.peerInfo.multiaddrs.forEach(addr => {
    //     if (match.LOCALHOST(addr)) {
    //         this.peerInfo.multiaddrs.delete(addr)
    //     }
    // })

    // if (publicAddrs) publicAddrs.forEach(addr => this.peerInfo.multiaddrs.add(addr.encapsulate(`/${NAME}/${this.peerInfo.id.toB58String()}`)))

    return this
  }

  /**
   * Shuts down the node and saves keys and peerBook in the database
   */
  async down(): Promise<void> {
    if (this.db) {
      await this.db.close()
    }

    this.log(`Database closed.`)

    if (this.network.heartbeat) {
      this.network.heartbeat.stop()
    }

    await super.stop()
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
   * the acknowledgement of the first hop
   */
  async sendMessage(msg: Uint8Array, destination: PeerId): Promise<void> {
    if (!destination) throw Error(`Expecting a non-empty destination.`)

    const promises = []

    for (let n = 0; n < msg.length / PACKET_SIZE; n++) {
      promises.push(
        new Promise(async (resolve, reject) => {
          let intermediateNodes = await this.getIntermediateNodes(destination)

          let path = intermediateNodes.concat(destination)

          await Promise.all(path.map(addPubKey))

          let packet: Packet<Chain>
          try {
            packet = await Packet.create(
              /* prettier-ignore */
              this,
              msg.slice(n * PACKET_SIZE, Math.min(msg.length, (n + 1) * PACKET_SIZE)),
              path
            )
          } catch (err) {
            return reject(err)
          }

          this.interactions.packet.acknowledgment.once(
            u8aToHex(this.dbKeys.UnAcknowledgedTickets(path[0].pubKey.marshal(), packet.ticket.ticket.challenge)),
            () => {
              console.log(`received acknowledgement`)
              resolve()
            }
          )

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
      this.log(`Could not send message. Error was: ${chalk.red(err.message)}`)
    }
  }

  /**
   * Ping a node.
   *
   * @param destination PeerId of the node
   * @returns latency
   */
  async ping(destination: PeerId): Promise<number> {
    if (!destination) throw Error(`Expecting a non-empty destination.`)

    const latency = await super.ping(destination)

    if (typeof latency === 'undefined') {
      throw Error('node unreachable')
    }

    return latency
  }

  /**
   * Takes a destination and samples randomly intermediate nodes
   * that will relay that message before it reaches its destination.
   *
   * @param destination instance of peerInfo that contains the peerId of the destination
   */
  async getIntermediateNodes(destination: PeerId) {
    const filter = (peerInfo: PeerInfo) =>
      !peerInfo.id.isEqual(this.peerInfo.id) &&
      !peerInfo.id.isEqual(destination) &&
      // exclude bootstrap server(s) from crawling results
      !this.bootstrapServers.some((pInfo: PeerInfo) => pInfo.id.isEqual(peerInfo.id))

    await this.network.crawler.crawl()

    const array = []
    for (const peerInfo of this.peerStore.peers.values()) {
      array.push(peerInfo)
    }
    return randomSubset(array, MAX_HOPS - 1, filter).map((peerInfo: PeerInfo) => peerInfo.id)
  }

  static openDatabase<Constructor extends typeof HoprCoreConnector>(
    db_dir: string,
    connector: Constructor,
    options?: { id?: number; bootstrapNode?: boolean }
  ) {
    db_dir += `/${connector.constants.CHAIN_NAME}/${connector.constants.NETWORK}/`

    if (options != null && options.bootstrapNode) {
      db_dir += `bootstrap`
    } else if (options != null && options.id != null && Number.isInteger(options.id)) {
      // For testing ...
      db_dir += `node_${options.id}`
    } else {
      db_dir += `node`
    }

    createDirectoryIfNotExists(db_dir)

    return levelup(leveldown(db_dir))
  }
}
