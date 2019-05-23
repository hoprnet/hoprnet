'use strict'
const chalk = require('chalk')
const PeerId = require('peer-id')
const PeerInfo = require('peer-info')

const pull = require('pull-stream')
const lp = require('pull-length-prefixed')

const Queue = require('promise-queue')
const { waterfall, tryEach } = require('neo-async')

const { randomSubset, log, pubKeyToPeerId } = require('../utils')
const { MAX_HOPS, PROTOCOL_CRAWLING } = require('../constants')
const COMPRESSED_PUBLIC_KEY_SIZE = 33

const MAX_PARALLEL_REQUESTS = 4
const QUEUE_MAX_SIZE = Infinity

module.exports = (node) => (comparator) => new Promise((resolve, reject) => {
    if (!comparator)
        comparator = () => true

    const queue = new Queue(MAX_PARALLEL_REQUESTS, QUEUE_MAX_SIZE)
    let finished = false, first = true

    const errors = []

    /**
     * Connect to another peer and returns a promise that resolves to all received nodes
     * that were previously unknown.
     * 
     * @param {PeerId} peerId PeerId of the peer that is queried
     */
    const queryNode = (peerId) => new Promise((resolve, reject) =>
        tryEach([
            (cb) => node.dialProtocol(peerId, PROTOCOL_CRAWLING, cb),
            (cb) => waterfall([
                (cb) => node.peerRouting.findPeer(peerId, cb),
                (peerInfo, cb) => node.dialProtocol(peerInfo, PROTOCOL_CRAWLING, cb)
            ], cb)
        ], (err, conn) => {
            if (err)
                return reject(err)

            pull(
                conn,
                lp.decode({
                    maxLength: COMPRESSED_PUBLIC_KEY_SIZE
                }),
                pull.collect((err, pubKeys) => {
                    if (err)
                        return cb(err)

                    let peerIds = pubKeys
                        .filter((pubKey) => pubKey.length == COMPRESSED_PUBLIC_KEY_SIZE)
                        .map((pubKey) => pubKeyToPeerId(pubKey))
                        .filter((peerId) => {
                            if (peerId.isEqual(node.peerInfo.id))
                                return false

                            const found = node.peerBook.has(peerId.toB58String())
                            node.peerBook.put(new PeerInfo(peerId))

                            return !found
                        })

                    resolve(peerIds)
                })
            )
        })
    )

    /**
     * Decides whether we have enough peers to build a path and initiates some queries
     * if that's not the case.
     * 
     * @param {PeerId[]} peerIds array of peerIds
     */
    const processResults = (peerIds) => {
        const now = node.peerBook.getAllArray().filter(comparator).length
        const enoughPeers = now >= MAX_HOPS

        if (finished)
            return

        if (!first && enoughPeers) {
            if (errors.length > 0)
                log(node.peerInfo.id, `Errors while crawling:${errors.reduce((acc, err) => `\n${chalk.red(err.message)}`, '')}`)

            finished = true

            log(node.peerInfo.id, `Received ${now - before} new node${now - before == 1 ? '' : 's'}.`)
            log(node.peerInfo.id, `Now holding peer information of ${now} node${now == 1 ? '' : 's'} in the network.`)

            return resolve()
        }

        first = false

        if (peerIds.length > 0)
            return randomSubset(peerIds, Math.min(peerIds.length, MAX_PARALLEL_REQUESTS)).forEach((peerId) =>
                queue.add(() => queryNode(peerId))
                    .then((peerIds) => {
                        processResults(peerIds)
                    })
                    .catch((err) => {
                        errors.push(err)
                        return processResults([])
                    })
            )

        if (queue.getPendingLength() == 0) {
            if (errors.length > 0)
                log(node.peerInfo.id, `Errors while crawling:${errors.reduce((acc, err) => `\n${chalk.red(err.message)}`, '')}`)

            log(node.peerInfo.id, `Received ${now - before} new node${now - before > 1 ? '' : 's'}.`)
            log(node.peerInfo.id, `Now holding peer information of ${now} node${now == 1 ? '' : 's'} in the network.`)

            reject(Error('Unable to find enough other nodes in the network.'))
        }
    }

    const before = node.peerBook.getAllArray().filter(comparator).length

    let nodes = node.peerBook.getAllArray().map((peerInfo) => peerInfo.id)

    processResults(nodes)
})