'use strict'

const dgram = require('dgram')
const stun = require('webrtc-stun')

const Multiaddr = require('multiaddr')
const PeerInfo = require('peer-info')

module.exports.getPublicIp = (peerInfos, ownPeerId) => {
    const multiaddrs = peerInfos.reduce((acc, peerInfo) => {
        acc.push(...peerInfo.multiaddrs.toArray())

        return acc
    }, [])

    const doSTUNRequest = ma =>
        new Promise((resolve, reject) => {
            const cOpts = ma.toOptions()

            let socket
            switch (cOpts.family.toLowerCase()) {
                case 'ipv4':
                    socket = dgram.createSocket('udp4')
                    break
                case 'ipv6':
                    socket = dgram.createSocket('udp6')
                    break
                default:
                    throw Error('Invalid input')
            }

            const tid = stun.generateTransactionId()

            socket.on('message', msg => {
                const res = stun.createBlank()

                if (res.loadBuffer(msg)) {
                    if (res.isBindingResponseSuccess({ transactionId: tid })) {
                        const attr = res.getXorMappedAddressAttribute()
                        if (attr) {
                            resolve(Multiaddr.fromNodeAddress(attr, 'udp').encapsulate(`/ipfs/${ownPeerId.toB58String()}`))
                        }
                    }
                }

                socket.close()
            })

            const req = stun
                .createBindingRequest(tid)
                //.setSoftwareAttribute(`${pkg.name}@${pkg.version}`)
                .setFingerprintAttribute()

            socket.send(req.toBuffer(), cOpts.port, cOpts.host)
        })
    return Promise.race(multiaddrs.map(doSTUNRequest))
}

module.exports.getSTUNServers = peerInfos => {
    const result = []

    /**
     * If there are no known STUN server, use the ones from Google and Twilio (default configuration for simple-peer)
     */
    if (!peerInfos) {
        return [{ urls: 'stun:stun.l.google.com:19302' }, { urls: 'stun:global.stun.twilio.com:3478?transport=udp' }]
    }

    if (!Array.isArray(peerInfos) && PeerInfo.isPeerInfo(peerInfos)) peerInfos = [peerInfos]

    peerInfos.forEach(peerInfo => {
        peerInfo.multiaddrs.forEach(ma => {
            const opts = ma.toOptions()

            if (opts.family.toLowerCase() === 'ipv4') {
                result.push({
                    urls: `stun:${opts.host}:${opts.port}`
                })
            } else if (opts.family.toLowerCase() === 'ipv6') {
                result.push({
                    urls: `stun:[${opts.host}]:${opts.port}`
                })
            }
        })
    })

    return result
}

module.exports.answerStunRequest = (msg, rinfo, send) => {
    const req = stun.createBlank()

    // if msg is valid STUN message
    if (req.loadBuffer(msg)) {
        // if STUN message is BINDING_REQUEST and valid content
        if (req.isBindingRequest({ fingerprint: true })) {
            // console.log('REQUEST', req)

            const res = req
                .createBindingResponse(true)
                .setXorMappedAddressAttribute(rinfo)
                .setFingerprintAttribute()

            // console.log('RESPONSE', res)
            send(res.toBuffer())
        }
    }
}
