'use strict'

const { PROTOCOL_PAYMENT_CHANNEL } = require('../constants')
const pull = require('pull-stream')
const utils = require('../../utils')
const pullJson = require('pull-json-doubleline')
const waterfall = require('async/waterfall')
const paymentChannel = require('../paymentChannel')

// module.exports.registerFunctionality = (node, registerChannel) => {
//     node.handle(constants.paymentChannelProtocol, (protocol, conn) => {
//         conn.getPeerInfo((err, peerId) => {
//             if (err) { throw err }

//             pull(
//                 conn,
//                 pull.map(str => utils.parseJSON(str)),
//                 processMessage(node, peerId, registerChannel),
//                 pullJson.stringify(),
//                 conn
//             )
//         })
//     })
// }

// function processMessage(node, peerId, registerChannel) {
//     return function (read) {
//         return function (end, reply) {
//             read(end, function next(end, tx) {
//                 if (!end) {
//                     waterfall([
//                         (cb) => paymentChannel.establish(node, peerId, tx, cb),
//                         // registerChannel() ...
//                     ], (err, tx, channel) => {
//                         if (err) { throw err }
//                         reply(end, tx)
//                     })
//                 } else {
//                     reply(end, null)
//                 }
//             })
//         }
//     }
// }

module.exports = (node) => node.handle(PROTOCOL_PAYMENT_CHANNEL, (protocol, conn) => pull(
    conn,
    (read) => {
        return function (end, reply) {
            read(end, function next(end, data) {
            })
        }
    },
    conn
))