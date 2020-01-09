'use strict'

const libp2p = require('libp2p')
const MPLEX = require('libp2p-mplex')
const KadDHT = require('libp2p-kad-dht')
// const SECIO = require('libp2p-secio')
const { WebRTCv4, WebRTCv6 } = require('./network/natTraversal')
const TCP = require('libp2p-tcp')

const defaultsDeep = require('@nodeutils/defaults-deep')

const { createPacket } = require('./packet')
const registerHandlers = require('./handlers')
const { NAME, PACKET_SIZE, PROTOCOL_STRING, MAX_HOPS } = require('./constants')
const crawler = require('./network/crawler')
const heartbeat = require('./network/heartbeat')
const getPeerInfo = require('./getPeerInfo')
const { randomSubset, serializePeerBook, deserializePeerBook, log, addPubKey, createDirectoryIfNotExists } = require('./utils')

import { default as levelup, LevelUp } from 'levelup'
import leveldown from 'leveldown'
import Multiaddr from 'multiaddr'

import PeerId from 'peer-id'
import PeerInfo from 'peer-info'
import PeerBook from 'peer-book'

import PaymentChannels from '@hoprnet/hopr-core-connector-interface'

import pull from 'pull-stream'

const lp = require('pull-length-prefixed')
const Acknowledgement = require('./acknowledgement')

type HoprOptions = {
  peerInfo: PeerInfo
  output: (str: string) => void
  id?: number
}

export default class Hopr<ChainConnector extends PaymentChannels> extends libp2p {
  /**
   * @constructor
   *
   * @param _options
   * @param provider
   */
  private constructor(_options: HoprOptions, public db: LevelUp, public bootstrapServers: PeerInfo[], public paymentChannels: PaymentChannels) {
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
          connEncryption: [], //  [SECIO],
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

    this.heartbeat = heartbeat(this)
  }

  /**
   * Creates a new node.
   *
   * @param options the parameters
   */
  static async createNode<ChainConnector extends PaymentChannels>(
    ChainConnector: ChainConnector,
    options?: HoprOptions & {
      'bootstrapServers'?: PeerInfo[]
      'bootstrap-node'?: boolean
      'provider'?: string
    }
  ): Promise<Hopr<ChainConnector>> {
    const db = Hopr.openDatabase(`db`, options)

    if (options == null) {
      options = {
        // config: {
        //     // WebRTC: options.WebRTC
        // },
        // peerBook: peerBook,
        peerInfo: await getPeerInfo(options, db),
        output: console.log
      }
    }

    // peerBook: (cb) => {
    //     if (options.peerBook) {
    //         cb(null, options.peerBook)
    //     } else {
    //         Hopr.importPeerBook(db, cb)
    //     }
    // }

    return new Hopr<ChainConnector>(
      options,
      db,
      options['bootstrap-node'] ? null : options.bootstrapServers,
      options['bootstrap-node'] ? null : await ChainConnector.create(db, new Uint8Array(), options['provider'])
    ).up(options)
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
  async up(options: HoprOptions): Promise<Hopr<ChainConnector>> {
    await new Promise((resolve, reject) =>
      super.start((err: Error) => {
        if (err) return reject(err)

        resolve()
      })
    )

    registerHandlers(this, options)

    if (!options['bootstrap-node']) {
      await this.connectToBootstrapServers()
    } else {
      log(this.peerInfo.id, `Available under the following addresses:`)
      this.peerInfo.multiaddrs.forEach((ma: Multiaddr) => {
        log(this.peerInfo.id, ma.toString())
      })
    }

    // this.heartbeat.start()

    this.crawler = new crawler({ libp2p: this })

    // this.peerInfo.multiaddrs.forEach(addr => {
    //     if (match.LOCALHOST(addr)) {
    //         this.peerInfo.multiaddrs.delete(addr)
    //     }
    // })

    if (!options['bootstrap-node']) {
      this.paymentChannels = await PaymentChannels.create(this)
    }

    // if (publicAddrs) publicAddrs.forEach(addr => this.peerInfo.multiaddrs.add(addr.encapsulate(`/${NAME}/${this.peerInfo.id.toB58String()}`)))

    return this
  }

  /**
   * Shuts down the node and saves keys and peerBook in the database
   */
  async down(): Promise<void> {
    if (this.db) await this.db.close()

    log(this.peerInfo.id, `Database closed.`)

    await new Promise((resolve, reject) =>
      super.stop((err: Error) => {
        if (err) return reject(err)

        log(this.peerInfo.id, `Node shut down.`)

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
  async sendMessage(msg: string | Buffer, destination: PeerId): Promise<void> {
    if (!destination) throw Error(`Expecting a non-empty destination.`)

    if (!Buffer.isBuffer(msg)) {
      msg = Buffer.from(msg)
    }

    const promises = []

    for (let n = 0; n < msg.length / PACKET_SIZE; n++) {
      promises.push(
        new Promise(async (resolve, reject) => {
          let intermediateNodes = await this.getIntermediateNodes(destination)

          let path = intermediateNodes.concat(destination)

          await Promise.all(path.map(addPubKey))

          const peerInfo = await this.peerRouting.findPeer(path[0])

          const conn = await this.dialProtocol(peerInfo, PROTOCOL_STRING)

          const packet = await createPacket(
            /* prettier-ignore */
            this,
            msg.slice(n * PACKET_SIZE, Math.min(msg.length, (n + 1) * PACKET_SIZE)),
            path
          )

          pull(
            pull.once(packet.toBuffer()),
            lp.encode(),
            conn,
            lp.decode({
              maxLength: Acknowledgement.SIZE
            }),
            pull.drain(data => {
              log(this.peerInfo.id, `Received acknowledgement.`)
              // return cb()
              // if (!cb.called) {
              //     return cb()
              // }
            }, resolve)
          )
        })
      )
    }

    try {
      await Promise.all(promises)
    } catch (err) {
      console.log(err)
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
    await this.crawler.crawl()

    return randomSubset(this.peerBook.getAllArray(), MAX_HOPS - 1, filter).map((peerInfo: PeerInfo) => peerInfo.id)
  }

  static async importPeerBook(db: LevelUp) {
    const key = 'peer-book'

    const peerBook = new PeerBook()

    let serializedPeerbook: Buffer
    try {
      serializedPeerbook = await db.get(key)
    } catch (err) {
      if (err.notFound) {
        return peerBook
      } else {
        throw err
      }
    }

    return deserializePeerBook(serializedPeerbook, peerBook)
  }

  async exportPeerBook() {
    const key = 'peer-book'

    await this.db.put(key, serializePeerBook(this.peerBook))
  }

  static openDatabase(db_dir: string, options?: { id?: number }) {
    if (options != null) {
      db_dir += `/${process.env['NETWORK']}/`
      if (Number.isInteger(options.id) && options['bootstrap-node'] == false) {
        // For testing ...
        db_dir += `node_${options.id}`
      } else if (!Number.isInteger(options.id) && options['bootstrap-node'] == false) {
        db_dir += `node`
      } else if (!Number.isInteger(options.id) && options['bootstrap-node'] == true) {
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
