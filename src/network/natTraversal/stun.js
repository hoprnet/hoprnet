'use strict'

const pull = require('pull-stream')
const lp = require('pull-length-prefixed')
const Multiaddr = require('multiaddr')

const dgram = require('dgram')
const stun = require('webrtc-stun')

// module.exports = (node, options) => new Promise((resolve, reject) => {
//     const socket = dgram.createSocket({ type: 'udp4' })
//     const tid = stun.generateTransactionId()

//     socket.on('message', msg => {
//         const res = stun.createBlank()

//         // if msg is valid STUN message
//         if (res.loadBuffer(msg)) {
//             // if msg is BINDING_RESPONSE_SUCCESS and valid content
//             if (res.isBindingResponseSuccess({ transactionId: tid })) {
//                 const attr = res.getXorMappedAddressAttribute()
//                 // if msg includes attr
//                 if (attr) {
//                     console.log('RESPONSE', res)
//                 }
//             }
//         }

//         socket.close()
//     })

//     const req = stun
//         .createBindingRequest(tid)
//         .setSoftwareAttribute(`${pkg.name}@${pkg.version}`)
//         .setFingerprintAttribute()

//     console.log('REQUEST', req)
//     // socket.send(req.toBuffer(), 3478, 'stun.webrtc.ecl.ntt.com');
//     socket.send(req.toBuffer(), 19302, 'stun.l.google.com')
//     // socket.send(req.toBuffer(), 55555);
// })

// // addr => tcp addrs
// node.dialProtocol(node.bootstrapServers[0], PROTOCOL_STUN, (err, conn) => {
//     if (err) return cb(err)

//     pull(conn, lp.decode(), pull.map(data => Multiaddr(data)), pull.collect(cb))
// })
