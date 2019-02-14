'use strict'

const { waterfall, each } = require('async')
const { randomSubset } = require('../utils')
const { PROTOCOL_NAME } = require('../constants')
const PeerId = require('peer-id')

const differenceWith = require('lodash.differencewith')


module.exports = (node, options, WebRTC) => (peer) => {
    const current = WebRTC.filter(node.peerInfo.multiaddrs.toArray()).map((multiaddr) => PeerId.createFromB58String(multiaddr.getPeerId()))

    const peers = node.peerBook.getAllArray()
    const amountOfPeers = Math.min(peers.length - options.bootstrapServers.length, parseInt(options.WebRTC.signallingServers))

    if (amountOfPeers <= 0)
        return

    const selected = randomSubset(peers, amountOfPeers, (peerInfo) =>
        !options.bootstrapServers.some((multiaddr) => PeerId.createFromB58String(multiaddr.getPeerId()).isEqual(peerInfo.id))
    ).map((peerInfo) => peerInfo.id)

    if (selected.length <= 0)
        return

    const isEqual = (a, b) => a.isEqual(b)
    const toDelete = differenceWith(current, selected, isEqual)
    const toAdd = differenceWith(selected, current, isEqual)
    console.log(`To add ${toAdd.length}. To remove ${toDelete.length}.`)

    waterfall([
        (cb) => {
            if (!node._switch.transports['WebRTCStar']) {
                node._switch.transport.add('WebRTCStar', WebRTC)
                cb()
            } else {
                node._switch.transport.close('WebRTCStar', (err) => cb(err))
            }
        },
        (cb) => {
            toDelete.forEach((peerId) => {
                const deleteCandidates = node.peerInfo.multiaddrs.toArray().filter((multiaddr) => PeerId.createFromB58String(multiaddr.getPeerId()).isEqual(peerId))

                deleteCandidates.forEach((multiaddr) => node.peerInfo.multiaddrs.delete(multiaddr))
            })

            each(toAdd, (peerId, cb) => waterfall([
                (cb) => {
                    const peerInfo = node.peerBook.get(peerId)
                    if (!peerInfo.isConnected()) {
                        waterfall([
                            (cb) => node.peerRouting.findPeer(peerId, cb),
                            (peerInfo, cb) => node.dial(peerInfo, (err, conn) => {
                                if (err)
                                    return cb(err)

                                return cb(null, peerInfo._connectedMultiaddr)
                            })
                        ], cb)
                    } else {
                        cb(null, peerInfo._connectedMultiaddr)
                    }
                },
                (multiaddr, cb) => {
                    const peerOptions = multiaddr.toOptions()

                    let addr

                    if (WebRTC.filter([multiaddr]).length > 0) {
                        // already connected via WebRTCStar
                        addr = peer._connectedMultiaddr
                            .decapsulate(`${PROTOCOL_NAME}`)
                            .encapsulate(`/${PROTOCOL_NAME}/${node.peerInfo.id.toB58String()}`)
                    } else {
                        // connected via TCP
                        addr = peer._connectedMultiaddr
                            .decapsulate(`${PROTOCOL_NAME}`)
                            .decapsulate('tcp')
                            .encapsulate(`/tcp/${parseInt(peerOptions.port) + 1}/ws/p2p-webrtc-star`)
                            .encapsulate(`/${PROTOCOL_NAME}/${node.peerInfo.id.toB58String()}`)
                    }

                    console.log(`now available under ${addr.toString()}\n`)

                    node.peerInfo.multiaddrs.add(addr)
                    cb()
                }
            ], cb), (err) => {
                if (err)
                    return cb(err)

                node._switch.transport.listen('WebRTCStar', {}, null, cb)
            })
        }
    ], (err) => {
        if (err)
            console.log(err)

        // this.dial('/dns4/hopr.validity.io/tcp/9092/ws/p2p-webrtc-star/ipfs/QmS7Wtck9aFHUu2zEzqzEdnzaG5jvp9wxjGYL7JJ15WBJD', (err, conn) => {
        //     console.log(err, conn)
        // })
    })

}