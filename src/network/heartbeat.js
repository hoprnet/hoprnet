'use strict'

const lp = require('pull-length-prefixed')
const pull = require('pull-stream')

const chalk = require('chalk')

const { randomBytes, createHash } = require('crypto')
const { waterfall, series, tryEach } = require('neo-async')

const { PROTOCOL_HEARTBEAT } = require('../constants')
const { log } = require('../utils')

const THIRTY_ONE_SECONDS = 31 * 1000
const HASH_SIZE = 32

module.exports = (node) => {
    let timers

    const queryNode = (peerInfo, cb) =>
        tryEach([
            (cb) => node.dialProtocol(peerInfo, PROTOCOL_HEARTBEAT, cb),
            (cb) => waterfall([
                (cb) => node.peerRouting.findPeer(peerInfo.id, cb),
                (peerInfo, cb) => node.dialProtocol(peerInfo, PROTOCOL_HEARTBEAT, cb)
            ], cb)
        ], (err, conn) => {
            if (err || !conn)
                return cb(false, err ? err.message : `Unable to connect to ${chalk.blue(peerInfo.id.toB58String())}.`)

            const challenge = randomBytes(16)

            try {
                pull(
                    pull.once(challenge),
                    lp.encode(),
                    conn,
                    lp.decode({
                        maxLength: HASH_SIZE
                    }),
                    pull.collect((err, response) => {
                        if (err || response.length != 1)
                            return cb(false, `foo .Invalid response from ${chalk.blue(peerInfo.id.toB58String())}.`)
    
                        const correctResponse = createHash('sha256').update(challenge).digest().slice(0, 16)
                        if (!response[0].equals(correctResponse))
                            return cb(false, `Invalid response from ${chalk.blue(peerInfo.id.toB58String())}.`)
    
                        if (!node.peerBook.has(peerInfo.id.toB58String()))
                            node.peerBook.put(peerInfo)
    
                        cb(true)
                    })
                )
            } catch (err) {
                cb(false, `Connection to ${chalk.blue(peerInfo.id.toB58String())} broke up.`)
            }
            
        })

    const startTimer = (peerInfo) => {
        const timer = timers.get(peerInfo.id.toB58String())
        
        if (!timer) 
            console.log('here')
            timers.set(peerInfo.id.toB58String(), setInterval(() => queryNode(peerInfo, (available, reason) => {
                if (!available) {
                    log(node.peerInfo.id, `Removing ${peerInfo.id.toB58String()} from peerBook due to "${reason}".`)

                    clearInterval(timers.get(peerInfo.id.toB58String()))
                    timers.delete(peerInfo.id.toB58String())

                    series([
                        (cb) => node.hangUp(peerInfo, cb),
                        (cb) => node._dht.routingTable.remove(peerInfo.id, cb),
                    ], (err) => {
                        node.peerBook.remove(peerInfo.id)
                    })
                }
            }), THIRTY_ONE_SECONDS))
    }

    const start = () => {
        timers = new Map()

        node.on('peer:connect', startTimer)
    }

    const stop = () => {
        node.off('peer:connect', startTimer)
        timers.forEach((timer) => clearInterval(timer))
        timers.clear()
    }

    return {
        start,
        stop
    }
}

