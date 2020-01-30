'use strict'

const pull = require('pull-stream')
const { createHash } = require('crypto')
const chalk = require('chalk')
const lp = require('pull-length-prefixed')
const { log } = require('../utils')

const CHALLENGE_LENGTH = 16
const { PROTOCOL_HEARTBEAT } = require('../constants')

module.exports = node =>
  node.handle(PROTOCOL_HEARTBEAT, (protocol, conn) =>
    pull(
      conn,
      lp.decode({
        maxLength: CHALLENGE_LENGTH
      }),
      pull.asyncMap((data, cb) =>
        conn.getPeerInfo((err, peerInfo) => {
          if (err) {
            log(node.peerInfo.id, chalk.red(err.message))
          } else {
            if (!node.peerBook.has(peerInfo.id.toB58String())) {
              log(node.peerInfo.id, `Heartbeat handler: Accidentially storing peerId ${peerInfo.id.toB58String()} in peerBook.`)
              node.peerBook.put(peerInfo)
            }

            node.peerBook.getAll()[peerInfo.id.toB58String()].lastSeen = Date.now()
          }

          return cb(
            null,
            createHash('sha256')
              .update(data)
              .digest()
              .slice(0, 16)
          )
        })
      ),
      lp.encode(),
      conn
    )
  )
