'use strict'

const PeerId = require('peer-id')
const PeerInfo = require('peer-info')

const pull = require('pull-stream')

const doWhilst = require('async/doWhilst')
const times = require('async/times')
const waterfall = require('async/waterfall')
const filter = require('async/filter')
const eachSeries = require('async/eachSeries')

const { MAX_HOPS, PROTOCOL_CRAWLING, MARSHALLED_PUBLIC_KEY_SIZE } = require('./constants')

module.exports = (node) =>
    (cb, comparator = _ => true) => {

        let peers = node.peerBook.getAllArray()
        let newNodes = []

        doWhilst(
            (cbWhilst) => {
                if (peers.length === 0)
                    throw Error('Unable to find enough other nodes in the network.')

                times(Math.min(peers.length, MAX_HOPS), (_, next) => waterfall([
                    (cb) => {
                        const currentPeerInfo = peers.pop()
                        if (currentPeerInfo.multiaddrs.size === 0) {
                            node.peerRouting.findPeer(currentPeerInfo.id, cb)
                        } else {
                            cb(null, currentPeerInfo)
                        }
                    },
                    (peerInfo, cb) => node.dialProtocol(peerInfo, PROTOCOL_CRAWLING, (err, conn) => cb(err, conn, peerInfo)),

                    (conn, currentPeerInfo, cb) => pull(
                            conn,
                            pull.filter(data => data.length > 0 && data.length % MARSHALLED_PUBLIC_KEY_SIZE === 0),
                            pull.drain(buf => waterfall([
                                // TODO: Add multiaddrs to response to decrease amount of rountrips
                                (cb) => times(buf.length / MARSHALLED_PUBLIC_KEY_SIZE,
                                    (n, next) => waterfall([
                                        (cb) => PeerId.createFromPubKey(buf.slice(n * MARSHALLED_PUBLIC_KEY_SIZE, (n + 1) * MARSHALLED_PUBLIC_KEY_SIZE), cb),
                                        (peerId, cb) => PeerInfo.create(peerId, cb)
                                    ], next),
                                    cb),
                                (peerInfos, cb) => filter(peerInfos,
                                    (peerInfo, cb) =>
                                        cb(null, currentPeerInfo.id.toBytes().compare(peerInfo.id.toBytes()) !== 0)
                                    , cb),
                                (peerInfos, cb) => eachSeries(peerInfos,
                                    (peerInfo, cb) => {
                                        if (!node.peerBook.has(peerInfo.id.toB58String()))
                                            newNodes.push(peerInfo)

                                        cb(null, node.peerBook.put(peerInfo))
                                    }, cb)
                            ], cb))
                        )

                ], next), (err) => {
                    if (err) { throw err }

                    console.log('[\'' + node.peerInfo.id.toB58String() + '\']: Received ' + newNodes.length + ' new node' + (newNodes.length === 1 ? '' : 's') + '.')
                    console.log('[\'' + node.peerInfo.id.toB58String() + '\']: Now holding peer information of ' + node.peerBook.getAllArray().length + ' node' + (node.peerBook.getAllArray().length === 1 ? '' : 's') + ' in the network.')

                    peers = peers.concat(newNodes)
                    newNodes = []

                    cbWhilst()
                })
            }, () => node.peerBook.getAllArray().filter(comparator).length < MAX_HOPS - 1, cb)
    }