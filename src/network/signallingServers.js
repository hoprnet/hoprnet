'use strict'

const { series } = require('async')
const { randomNumber } = require('../utils')
const { PROTOCOL_NAME } = require('../constants')
const mafmt = require('mafmt')
const Multiaddr = require('multiaddr')

module.exports = (node, options, WebRTC) => (newPeerInfo) => {
    const peerIdStrings = WebRTC.filter(node.peerInfo.multiaddrs.toArray())
        .map((multiaddr) => multiaddr.getPeerId())
        .concat(newPeerInfo.id.toB58String())

    let toDelete = -1
    if (peerIdStrings.length > options.WebRTC.signallingServers) {
        toDelete = randomNumber(0, peerIdStrings.length)
    }

    if (toDelete == peerIdStrings.length - 1)
        return

    series([
        (cb) => {
            if (!node._switch.transports['WebRTCStar']) {
                node._switch.transport.add('WebRTCStar', WebRTC)
                return cb()
            }

            return node._switch.transport.close('WebRTCStar', (err) => cb(err))
        },
        (cb) => {
            if (toDelete >= 0) {
                node.peerInfo.multiaddrs.toArray()
                    .filter((multiaddr) =>
                        mafmt.WebRTCStar.matches(multiaddr) &&
                        multiaddr.getPeerId() === peerIdStrings[toDelete])
                    .forEach((multiaddr) =>
                        node.peerInfo.multiaddrs.delete(multiaddr))
            }

            const addrs = newPeerInfo.multiaddrs.toArray().filter((multiaddr) =>
                (
                    mafmt.WebSockets.matches(multiaddr.decapsulate(`/${PROTOCOL_NAME}`)) ||
                    mafmt.WebSocketsSecure.matches(multiaddr.decapsulate(`/${PROTOCOL_NAME}`))
                ) &&
                !['0.0.0.0', '127.0.0.1'].includes(multiaddr.toOptions().host)
            )

            if (addrs.length == 0)
                return cb(Error(`Unable to detect address of signalling server. Given multiaddress are ${newPeerInfo.multiaddrs.toArray().join(', ')}.`))

            addrs.forEach((multiaddr) => {
                const options = multiaddr.toOptions()

                node.peerInfo.multiaddrs.add(
                    multiaddr
                        .decapsulate('tcp')
                        .encapsulate(`/${options.transport}/${parseInt(options.port) + 1}/ws/p2p-webrtc-star/${PROTOCOL_NAME}/${node.peerInfo.id.toB58String()}`)
                )
            })

            node._switch.transport.listen('WebRTCStar', {}, null, cb)
        }
    ], (err) => {
        if (err)
            console.log(`SignallingServers: ${err.message}. ${err.stack}. ${err.toString()}`)
    })
}