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

module.exports = (node) => (cb, comparator = _ => true) => {
    let nodes = [], selected

    node.peerBook.getAllArray().forEach((peerInfo) => {
        nodes.push(peerInfo.id.toB58String())
    })

    function queryNode(peerId, cb) {
        waterfall([
            (cb) => {
                const connectedMultiaddr = node.peerBook.get(peerId).isConnected()

                if (connectedMultiaddr) {
                    return cb(null, connectedMultiaddr)
                }

                node.peerRouting.findPeer(PeerId.createFromB58String(peerId), cb)
            },
            (peerInfo, cb) => node.dialProtocol(peerInfo, PROTOCOL_CRAWLING, cb),
            (conn, cb) => pull(
                conn,
                lp.decode(),
                pull.filter(data =>
                    data.length > 0 &&
                    data.length === MARSHALLED_PUBLIC_KEY_SIZE),
                pull.asyncMap((pubKey, cb) =>
                    PeerId.createFromPubKey(pubKey, (err, peerId) => {
                        if (err)
                            return cb(err)

                        cb(null, new PeerInfo(peerId))
                    })),
                pull.filter(peerInfo =>
                    // received node != known nodes
                    !node.peerBook.has(peerInfo.id.toB58String()) &&
                    // received node != self
                    node.peerInfo.id.toBytes().compare(peerInfo.id.toBytes()) !== 0
                ),
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

            newNodes = uniqWith(flatten(newNodes), (a, b) =>
                a.id.toBytes().compare(b.id.toBytes())
            )

            newNodes.forEach(peerInfo => {
                node.peerBook.put(peerInfo)
                nodes.push(peerInfo.id.toB58String())
            })

            log(node.peerInfo.id, `Received ${newNodes.length} new node${newNodes.length === 1 ? '' : 's'}.`)
            log(node.peerInfo.id, `Now holding peer information of ${node.peerBook.getAllArray().length} node${node.peerBook.getAllArray().length === 1 ? '' : 's'} in the network.`)
            // log(node.peerInfo.id, node.peerBook.getAllArray().reduce((acc, peerInfo) => {
            //     return acc.concat(`PeerId ${peerInfo.id.toB58String()}, available under ${peerInfo.multiaddrs.toArray().join(', ')}`)
            // }, ''))

            return cb()
        })
    }, () => {
        const length = node.peerBook.getAllArray().filter(comparator).length

        return length < MAX_HOPS
    }, cb)
}