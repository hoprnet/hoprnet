'use strict'

const { waterfall } = require('neo-async')
const { generateKeyPair } = require('libp2p-crypto').keys
const { deserializeKeyPair, serializeKeyPair, privKeyToPeerId } = require('./utils')

const PeerInfo = require('peer-info')
const PeerId = require('peer-id')
const Multiaddr = require('multiaddr')
const { NAME } = require('./constants')

module.exports = (options, db) => new Promise(async (resolve, reject) => {
    const getFromDatabase = (cb) =>
        db.get('key-pair', (err, serializedKeyPair) => {
            if (err && !err.notFound)
                return cb(err)

            if (!err)
                return deserializeKeyPair(serializedKeyPair, cb)

            let key, peerId
            waterfall([
                (cb) => generateKeyPair('secp256k1', 256, cb),
                (_key, cb) => {
                    key = _key
                    key.public.hash(cb)
                },
                (id, cb) => {
                    peerId = new PeerId(id, key, key.public)

                    serializeKeyPair(peerId, cb)
                },
                (cb) => db.put('key-pair', serializedKeyPair, cb)
            ], (err) => {
                if (err)
                    throw err

                cb(null, peerId)
            })
        })

    if (!db) {
        if (!options.peerInfo)
            return reject(Error('Invalid input parameter. Please set a valid peerInfo.'))
    }

    if (PeerInfo.isPeerInfo(options.peerInfo))
        return resolve(options.peerInfo)

    if (!process.env.PORT || !process.env.HOST)
        return reject(Error('Unable to start node without an address. Please provide at least one.'))

    let port = process.env.PORT
    let host = process.env.HOST

    if (Number.isInteger(options.id)) 
        port = (Number.parseInt(port) + 2 * options.id).toString()

    options.addrs = [Multiaddr.fromNodeAddress({
        address: host,
        port: port
    }, 'tcp')]

    let peerId
    if (Number.isInteger(options.id)) {
        if (Number.parseInt(process.env.DEMO_ACCOUNTS) < options.id)
            return reject(Error(`Failed while trying to access demo account number ${options.id}. Please ensure that there are enough demo account specified in '.env'.`))

        peerId = privKeyToPeerId(process.env[`DEMO_ACCOUNT_${options.id}_PRIVATE_KEY`])
    } else if (PeerId.isPeerId(options.peerId)) {
        peerId = options.peerId
    } else if (options['bootstrap-node']) {
        peerId = privKeyToPeerId(process.env.FUND_ACCOUNT_PRIVATE_KEY)
    } else {
        peerId = await new Promise((resolve, reject) => getFromDatabase((err, peerId) => {
            if (err)
                return reject(err)

            resolve(peerId)
        }))
    }

    const peerInfo = new PeerInfo(peerId)
    options.addrs.forEach((addr) =>
        peerInfo.multiaddrs.add(addr.encapsulate(`/${NAME}/${peerInfo.id.toB58String()}`))
    )

    return resolve(peerInfo)
})