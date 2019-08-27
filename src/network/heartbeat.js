'use strict'

const lp = require('pull-length-prefixed')
const pull = require('pull-stream')

const chalk = require('chalk')

const { randomBytes, createHash } = require('crypto')

const { PROTOCOL_HEARTBEAT } = require('../constants')
const { log } = require('../utils')

const THIRTY_ONE_SECONDS = 31 * 1000
const HASH_SIZE = 32

module.exports = (node) => {
    let timers

    const queryNode = (peerInfo) => new Promise(async (resolve, reject) => {
        const conn = await Promise.race([
            node.dialProtocol(peerInfo, PROTOCOL_HEARTBEAT),
            node.peerRouting.findPeer(peerInfo.id).then(peerInfo => node.dialProtocol(peerInfo, PROTOCOL_HEARTBEAT))
        ])

        if (!conn) {
            return reject(Error(`Unable to connect to ${chalk.blue(peerInfo.id.toB58String())}.`))
        }

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
                        return reject(Error(`Invalid response from ${chalk.blue(peerInfo.id.toB58String())}.`))

                    const correctResponse = createHash('sha256').update(challenge).digest().slice(0, 16)

                    if (!response[0].equals(correctResponse))
                        return reject(`Invalid response from ${chalk.blue(peerInfo.id.toB58String())}.`)

                    if (!node.peerBook.has(peerInfo.id.toB58String()))
                        node.peerBook.put(peerInfo)

                    resolve()
                })
            )
        } catch (err) {
            return reject(Error(`Connection to ${chalk.blue(peerInfo.id.toB58String())} broke up due to '${err.message}'`))
        }
    })

    const startTimer = (peerInfo) => {
        const timer = timers.get(peerInfo.id.toB58String())

        if (!timer)
            timers.set(peerInfo.id.toB58String(), setInterval(async () => {
                try {
                    availabe = await queryNode(peerInfo)
                } catch (err) {
                    log(node.peerInfo.id, `Removing ${peerInfo.id.toB58String()} from peerBook due to "${reason}".`)

                    clearInterval(timers.get(peerInfo.id.toB58String()))
                    timers.delete(peerInfo.id.toB58String())

                    try {
                        await node.hangUp(peerInfo)
                        await new Promise((resolve, reject) => node._dht.routingTable.remove(peerInfo.id, (err) => {
                            if (err) return reject(err)

                            resolve()
                        }))
                    } catch (err) {
                        node.peerBook.remove(peerInfo.id)
                    }
                }
            }, THIRTY_ONE_SECONDS))
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

