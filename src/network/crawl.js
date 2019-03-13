'use strict'

const PeerId = require('peer-id')
const PeerInfo = require('peer-info')

const pull = require('pull-stream')
const lp = require('pull-length-prefixed')

const flatten = require('lodash.flatten');
const uniqWith = require('lodash.uniqwith')
const remove = require('lodash.remove')
const { doWhilst, map, waterfall } = require('neo-async')

const { randomSubset, log } = require('../utils')
const { MAX_HOPS, PROTOCOL_CRAWLING, MARSHALLED_PUBLIC_KEY_SIZE } = require('../constants')

module.exports = (node) => (comparator, cb) => {
    if (!cb) {
        if (!comparator)
            throw Error('Invalid input parameter.')

        cb = comparator
        comparator = () => true
    }

    let nodes = [...node.peerBook.getAllArray().map((peerInfo) => peerInfo.id.toB58String())], selected

    function queryNode(peerId, cb) {
        waterfall([
            (cb) => node.peerRouting.findPeer(PeerId.createFromB58String(peerId), cb),
            (peerInfo, cb) => node.dialProtocol(peerInfo, PROTOCOL_CRAWLING, cb),
            (conn, cb) => pull(
                conn,
                lp.decode({
                    maxLength: MARSHALLED_PUBLIC_KEY_SIZE
                }),
                pull.asyncMap((pubKey, cb) => PeerId.createFromPubKey(pubKey, cb)),
                pull.filter(peerId => {
                    if (peerId.isEqual(node.peerInfo.id))
                        return false

                    const found = node.peerBook.has(peerId.toB58String())
                    node.peerBook.put(new PeerInfo(peerId))

                    return !found
                }),
                pull.collect(cb))
        ], cb)
    }

    doWhilst((cb) => {
        if (nodes.length === 0)
            return cb(Error('Unable to find enough other nodes in the network.'))

        selected = randomSubset(nodes, Math.min(nodes.length, MAX_HOPS))
        nodes = remove(nodes, selected)

        map(selected, queryNode, (err, newNodes) => {
            if (err) {
                console.log(err)
                return cb(err)
            }

            newNodes = uniqWith(flatten(newNodes), (a, b) => a.isEqual(b))

            nodes.push(...newNodes.map((peerId) => peerId.toB58String()))

            log(node.peerInfo.id, `Received ${newNodes.length} new node${newNodes.length === 1 ? '' : 's'}.`)
            log(node.peerInfo.id, `Now holding peer information of ${node.peerBook.getAllArray().length} node${node.peerBook.getAllArray().length === 1 ? '' : 's'} in the network.`)
            // log(node.peerInfo.id, node.peerBook.getAllArray().reduce((acc, peerInfo) => {
            //     return acc.concat(`PeerId ${peerInfo.id.toB58String()}, available under ${peerInfo.multiaddrs.toArray().join(', ')}`)
            // }, ''))

            return cb()
        })
    }, () => node.peerBook.getAllArray().filter(comparator).length < MAX_HOPS, cb)
}