'use strict'

const PeerId = require('peer-id')
const PeerInfo = require('peer-info')

const pull = require('pull-stream')
const lp = require('pull-length-prefixed')

const flatten = require('lodash.flatten');
const uniqWith = require('lodash.uniqwith')
const remove = require('lodash.remove')
const { doWhilst, map, waterfall } = require('neo-async')

const { randomSubset, log } = require('./utils')

const { MAX_HOPS, PROTOCOL_CRAWLING, MARSHALLED_PUBLIC_KEY_SIZE } = require('./constants')

module.exports = (node) =>
    (cb, comparator = _ => true) => {
        let nodes = [], selected, currentPeerInfo

        node.peerBook.getAllArray().forEach((peerInfo) => {
            nodes.push(peerInfo.id.toB58String())
        })

        doWhilst(
            (cbWhilst) => {
                if (nodes.size === 0)
                    throw Error('Unable to find enough other nodes in the network.')

                selected = randomSubset(nodes, Math.min(nodes.length, MAX_HOPS))
                nodes = remove(nodes, selected)

                map(selected, (currentNode, cb) => waterfall([
                    (cb) => {
                        currentPeerInfo = node.peerBook.get(currentNode)
                        if (currentPeerInfo.multiaddrs.size === 0) {
                            node.peerRouting.findPeer(currentPeerInfo.id, cb)
                        } else {
                            cb(null, currentPeerInfo)
                        }
                    },
                    (peerInfo, cb) => node.dialProtocol(peerInfo, PROTOCOL_CRAWLING, cb),
                    (conn, cb) => pull(
                        conn,
                        lp.decode(),
                        pull.filter(data =>
                            data.length > 0 &&
                            data.length % MARSHALLED_PUBLIC_KEY_SIZE === 0),
                        pull.asyncMap((pubKey, cb) => waterfall([
                            (cb) => PeerId.createFromPubKey(pubKey, cb),
                            (peerId, cb) => PeerInfo.create(peerId, cb)
                        ], cb)),
                        pull.filter(peerInfo =>
                            // received node != known nodes
                            !node.peerBook.has(peerInfo.id.toB58String()) &&
                            // received node != self
                            node.peerInfo.id.toBytes().compare(peerInfo.id.toBytes()) !== 0
                        ),
                        pull.collect(cb))
                ], cb), (err, newNodes) => {
                    if (err) { throw err }

                    newNodes = uniqWith(flatten(newNodes), (a, b) => 
                        a.id.toBytes().compare(b.id.toBytes())
                    )

                    newNodes.forEach(peerInfo => {
                        node.peerBook.put(peerInfo)
                        nodes.push(peerInfo.id.toB58String())
                    })

                    log(node.peerInfo.id, `Received ${newNodes.length} new node${newNodes.length === 1 ? '' : 's'}.`)
                    log(node.peerInfo.id, `Now holding peer information of ${node.peerBook.getAllArray().length} node${node.peerBook.getAllArray().length === 1 ? '' : 's'} in the network.`)

                    cbWhilst()
                })
            }, () => node.peerBook.getAllArray().filter(comparator).length < MAX_HOPS - 1, cb)
    }