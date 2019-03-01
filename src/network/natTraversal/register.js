'use strict'

const { randomNumber, match } = require('../../utils')
const { NAME } = require('../../constants')
const Multiaddr = require('multiaddr')
const unionWith = require('lodash.unionwith')

module.exports = (self) => (newPeerInfo) => {
    let addrs = self.sw._peerInfo.multiaddrs.toArray()
        .filter((addr) => match.WebRTC(addr))

    const addrsLength = addrs.length
    addrs = unionWith(addrs, [Multiaddr(`/${NAME}/${newPeerInfo.id.toB58String()}/p2p-webrtc-star/${NAME}/${self.sw._peerInfo.id.toB58String()}`)], (a, b) => a.equals(b))

    if (addrs.length == addrsLength)
        // Nothing to do since peer is already registered as a
        // signaling server
        return

    let toDelete = null
    if (addrs.length + 1 > self.options.signallingServers) {
        const choice = randomNumber(0, addrs.length)

        if (choice == addrs.length - 1)
            // The new peer is the peer that was selected
            // to delete, so nothing to do here
            return

        toDelete = addrs[choice]
    }

    const toAdd = Multiaddr(`/${NAME}/${newPeerInfo.id.toB58String()}/p2p-webrtc-star/${NAME}/${self.sw._peerInfo.id.toB58String()}`)
    
    self.sw._peerInfo.multiaddrs.replace(toDelete, toAdd)

    if (!self.sw.transports['WebRTCStar']) {
        self.sw.transport.add('WebRTCStar', self)
        self.sw.transport.listen('WebRTCStar', null, null, () => {})
    } 

    self.listener.listen([toAdd], (err) => {
        if (err)
            console.log(`SignallingServers: ${err.message}. ${err.stack}`, err)
    })
}