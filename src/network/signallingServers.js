'use strict'

const { series } = require('async')
const { randomNumber } = require('../utils')
const { PROTOCOL_NAME } = require('../constants')
const mafmt = require('mafmt')

module.exports = (node, options, WebRTC) => (newPeerInfo) => {
    if (!newPeerInfo.isConnected())
        return

    if (mafmt.WebRTCStar.matches(newPeerInfo.isConnected()))
        return

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

            const connectedMultiaddr = newPeerInfo.isConnected()

            const options = connectedMultiaddr.toOptions()

            console.log(`Signalling: Trying to connect to: ${connectedMultiaddr
                .decapsulate(`${options.transport}`)
                .encapsulate(`/${options.transport}/${parseInt(options.port) + 1}/ws/p2p-webrtc-star/${PROTOCOL_NAME}/${node.peerInfo.id.toB58String()}`).toString()}`)

            node.peerInfo.multiaddrs.add(
                connectedMultiaddr
                    .decapsulate(`${options.transport}`)
                    .encapsulate(`/${options.transport}/${parseInt(options.port) + 1}/ws/p2p-webrtc-star/${PROTOCOL_NAME}/${node.peerInfo.id.toB58String()}`)
            )

            node._switch.transport.listen('WebRTCStar', {}, null, cb)
        }
    ], (err) => {
        if (err)
            console.log(`SignallingServers: ${err.message}. ${err.stack}. ${err.toString()}`)
    })
}