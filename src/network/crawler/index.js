'use strict'
const chalk = require('chalk')
const PeerId = require('peer-id')
const PeerInfo = require('peer-info')

const pull = require('pull-stream')
const lp = require('pull-length-prefixed')

const Queue = require('promise-queue')

const fs = require('fs')
const protons = require('protons')

const { CrawlResponse, Status } = protons(fs.readFileSync(`${__dirname}/protos/response.proto`))

const crawlHandler = require('./handler')

const { randomSubset, log, pubKeyToPeerId } = require('../../utils')
const { MAX_HOPS, PROTOCOL_CRAWLING } = require('../../constants')

const MAX_PARALLEL_REQUESTS = 4
const QUEUE_MAX_SIZE = Infinity

module.exports = class Crawler {
    constructor(opts) {
        this.node = opts.libp2p

        this.node.handle(PROTOCOL_CRAWLING, crawlHandler(this.node))
    }

    crawl(comparator) {
        return new Promise((resolve, reject) => {
            if (!comparator) comparator = () => true

            const queue = new Queue(MAX_PARALLEL_REQUESTS, QUEUE_MAX_SIZE)
            let finished = false,
                first = true

            const errors = []

            /**
             * Connect to another peer and returns a promise that resolves to all received nodes
             * that were previously unknown.
             *
             * @param {PeerId} peerId PeerId of the peer that is queried
             */
            const queryNode = peerId =>
                new Promise(async (resolve, reject) => {
                    let conn
                    try {
                        conn = await this.node.dialProtocol(peerId, PROTOCOL_CRAWLING)
                    } catch (err) {
                        try {
                            conn = await this.node.peerRouting.findPeer(peerId).then(peerInfo => this.node.dialProtocol(peerInfo, PROTOCOL_CRAWLING))
                        } catch (err) {
                            reject(err)
                        }
                    }

                    pull(
                        conn,
                        lp.decode(),
                        pull.map(CrawlResponse.decode),
                        pull.filter(response => response.status === Status.OK),
                        pull.collect(async (err, responses) => {
                            if (err) return reject(err)

                            if (responses.length < 1) reject(Error('Empty response'))

                            resolve(
                                Promise.all(responses[0].pubKeys.map(pubKey => pubKeyToPeerId(pubKey))).then(peerIds =>
                                    peerIds.filter(peerId => {
                                        if (peerId.isEqual(this.node.peerInfo.id)) return false

                                        const found = this.node.peerBook.has(peerId.toB58String())
                                        this.node.peerBook.put(new PeerInfo(peerId))

                                        return !found
                                    })
                                )
                            )
                        })
                    )
                })

            /**
             * Decides whether we have enough peers to build a path and initiates some queries
             * if that's not the case.
             *
             * @param {PeerId[]} peerIds array of peerIds
             */
            const processResults = peerIds => {
                const now = this.node.peerBook.getAllArray().filter(comparator).length
                const enoughPeers = now >= MAX_HOPS

                if (finished) return

                if (!first && enoughPeers) {
                    if (errors.length > 0) log(this.node.peerInfo.id, `Errors while crawling:${errors.reduce((acc, err) => `\n${chalk.red(err.message)}`, '')}`)

                    finished = true

                    log(this.node.peerInfo.id, `Received ${now - before} new node${now - before == 1 ? '' : 's'}.`)
                    log(this.node.peerInfo.id, `Now holding peer information of ${now} node${now == 1 ? '' : 's'} in the network.`)

                    return resolve()
                }

                first = false

                if (peerIds.length > 0)
                    return randomSubset(peerIds, Math.min(peerIds.length, MAX_PARALLEL_REQUESTS)).forEach(peerId =>
                        queue
                            .add(() => queryNode(peerId))
                            .then(peerIds => {
                                processResults(peerIds)
                            })
                            .catch(err => {
                                errors.push(err)
                                return processResults([])
                            })
                    )

                if (queue.getPendingLength() == 0 && queue.getQueueLength() == 0) {
                    if (errors.length > 0) log(this.node.peerInfo.id, `Errors while crawling:${errors.reduce((acc, err) => `\n${chalk.red(err.message)}`, '')}`)

                    log(this.node.peerInfo.id, `Received ${now - before} new node${now - before > 1 ? '' : 's'}.`)
                    log(this.node.peerInfo.id, `Now holding peer information of ${now} node${now == 1 ? '' : 's'} in the network.`)

                    return reject(Error('Unable to find enough other nodes in the network.'))
                }
            }

            const before = this.node.peerBook.getAllArray().filter(comparator).length

            let nodes = this.node.peerBook.getAllArray().map(peerInfo => peerInfo.id)

            processResults(nodes)
        })
    }
}
