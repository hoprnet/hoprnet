import libp2p = require('libp2p')
import MPLEX = require('libp2p-mplex')
import KadDHT = require('libp2p-kad-dht')
import SECIO = require('libp2p-secio')
// import { WebRTCv4, WebRTCv6 } = require('./network/natTraversal')
import TCP = require('libp2p-tcp')

import defaultsDeep = require('@nodeutils/defaults-deep')

import { Packet } from './messages/packet'
import { PACKET_SIZE, PROTOCOL_STRING, MAX_HOPS } from './constants'

import { Network } from './network'

import { randomSubset, addPubKey, createDirectoryIfNotExists, getPeerInfo } from './utils'

import levelup, { LevelUp } from 'levelup'
import leveldown from 'leveldown'
import Multiaddr from 'multiaddr'
import chalk from 'chalk'
import Debug, { Debugger } from 'debug'
import Stream from 'stream'
import pipe from 'it-pipe'

import PeerId from 'peer-id'
import PeerInfo from 'peer-info'

import HoprCoreConnector, { HoprCoreConnectorInstance } from '@hoprnet/hopr-core-connector-interface'
import { Interactions, Duplex } from './interactions'
import { DbKeys } from './db_keys'

type HoprOptions = {
  peerInfo: PeerInfo
  output: (msg: Uint8Array) => void
  id?: number
  bootstrapServers?: PeerInfo[]
  bootstrapNode: boolean
}

export default class Hopr<Chain extends HoprCoreConnectorInstance> extends libp2p {
  public interactions: Interactions<Chain>
  public network: Network<Chain>
  public log: Debugger
  public dbKeys: DbKeys = new DbKeys()
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
  }
  declare peerRouting: {
    findPeer: (addr: PeerId) => Promise<PeerInfo>
  }
  declare handle: (protocol: string[], handler: (struct: { stream: Stream.Duplex }) => void) => void
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

    // this.heartbeat = heartbeat(this)

    this.log = Debug(`${chalk.blue(_options.peerInfo.id.toB58String())}: `)
  }

  /**
   * Creates a new node.
   *
   * @param options the parameters
   */
  static async createNode<Constructor extends HoprCoreConnector, Instance extends HoprCoreConnectorInstance>(
    HoprCoreConnector: Constructor,
    options?: Partial<HoprOptions> & {
      provider?: string
    }
  ): Promise<Hopr<Instance>> {
    const db = Hopr.openDatabase(`db`, options)

    if (options == null) {
      options = {}
    }

    if (options.bootstrapNode == null) {
      options.bootstrapNode = false
    }

    if (options.output == null) {
      options.output = console.log
    }

    if (options.peerInfo == null) {
      options.peerInfo = await getPeerInfo(options, db)
    }

    return new Hopr<Instance>(
      options as HoprOptions,
      db,
      options.bootstrapNode ? null : options.bootstrapServers,
      (await HoprCoreConnector.create(
        db,
        {
          publicKey: options.peerInfo.id.pubKey.marshal(),
          privateKey: options.peerInfo.id.privKey.marshal()
        },
        options['provider']
      )) as Instance
    ).up(options as HoprOptions)
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

    // this.heartbeat.start()

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
    if (this.db) await this.db.close()

    this.log(`Database closed.`)

    await new Promise((resolve, reject) =>
      super.stop((err: Error) => {
        if (err) return reject(err)

        this.log(`Node shut down.`)

        resolve()
      })
    )
    // this.heartbeat.stop()
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

          const peerInfo = await this.peerRouting.findPeer(path[0])

          const { stream } = await this.dialProtocol(peerInfo, PROTOCOL_STRING)

          const packet = await Packet.create(
            /* prettier-ignore */
            this,
            msg.slice(n * PACKET_SIZE, Math.min(msg.length, (n + 1) * PACKET_SIZE)),
            path
          )

          pipe(
            /* prettier-ignore */
            packet,
            stream,
            async function(source: AsyncIterable<Uint8Array>) {
              for await (const msg of source) {
                this.log(this.peerInfo.id, `Received acknowledgement.`)
                // return cb()
                // if (!cb.called) {
                //     return cb()
                // }
              }
            }
          )
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
   * Takes a destination and samples randomly intermediate nodes
   * that will relay that message before it reaches its destination.
   *
   * @param destination instance of peerInfo that contains the peerId of the destination
   */
  async getIntermediateNodes(destination: PeerId) {
    const filter = (peerInfo: PeerInfo) =>
      !peerInfo.id.isEqual(this.peerInfo.id) &&
      !peerInfo.id.isEqual(destination) &&
      !this.bootstrapServers.some((pInfo: PeerInfo) => pInfo.id.isEqual(peerInfo.id))

    // @TODO exclude bootstrap server(s) from crawling results
    this.network.crawler.crawl()

    const array = []
    for (const peerInfo of this.peerStore.peers.values()) {
      array.push(peerInfo)
    }
    return randomSubset(array, MAX_HOPS - 1, filter).map((peerInfo: PeerInfo) => peerInfo.id)
  }

  static openDatabase(db_dir: string, options?: { id?: number; bootstrapNode?: boolean }) {
    if (options != null) {
      db_dir += `/${process.env['NETWORK']}/`
      if (Number.isInteger(options.id) && options.bootstrapNode == false) {
        // For testing ...
        db_dir += `node_${options.id}`
      } else if (!Number.isInteger(options.id) && options.bootstrapNode != true) {
        db_dir += `node`
      } else if (!Number.isInteger(options.id) && options.bootstrapNode == true) {
        db_dir += `bootstrap`
      } else {
        throw Error(`Cannot run hopr with index ${options.id} as bootstrap node.`)
      }
    }

    createDirectoryIfNotExists(db_dir)

    //     clearDirectory(db_dir)
    //     fs.mkdirSync(db_dir, {
    //         mode: 0o777
    //     })
    // --------------------------

    return levelup(leveldown(db_dir))
  }
}
