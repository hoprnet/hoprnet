'use strict'

const { waterfall } = require('neo-async')
const { generateKeyPair } = require('libp2p-crypto').keys
const { deserializeKeyPair, serializeKeyPair, privKeyToPeerId } = require('./utils')

const PeerInfo = require('peer-info')
const PeerId = require('peer-id')
const Multiaddr = require('multiaddr')
const { NAME } = require('./constants')

module.exports = (options, db) =>
    new Promise(async (resolve, reject) => {
        const checkConfig = () => {
            if (!process.env.HOST_IPV4 && !process.env.HOST_IPV6) return reject(Error('Unable to start node without an address. Please provide at least one.'))

            if (!process.env.PORT) return reject(Error('Got no port to listen on. Please specify one.'))
        }

        const getFromDatabase = () =>
            new Promise(async (resolve, reject) => {
                let serializedKeyPair
                try {
                    serializedKeyPair = await db.get('key-pair')

                    resolve(
                        new Promise((resolve, reject) =>
                            deserializeKeyPair(serializedKeyPair, (err, peerId) => {
                                if (err) return reject(err)

                                resolve(peerId)
                            })
                        )
                    )
                } catch (err) {
                    if (!err.notFound) return reject(err)

                    let key, peerId
                    waterfall(
                        [
                            cb => generateKeyPair('secp256k1', 256, cb),
                            (_key, cb) => {
                                key = _key
                                key.public.hash(cb)
                            },
                            (id, cb) => {
                                peerId = new PeerId(id, key, key.public)

                                serializeKeyPair(peerId, cb)
                            },
                            (serializedKeyPair, cb) => db.put('key-pair', serializedKeyPair, cb)
                        ],
                        err => {
                            if (err) return reject(err)

                            resolve(peerId)
                        }
                    )
                }
            })

        const getAddrs = () => {
            const addrs = []

            let port = process.env.PORT

            // ============================= Only for testing ================================================
            if (Number.isInteger(options.id)) port = (Number.parseInt(port) + 2 * (options.id + 1)).toString()
            // ===============================================================================================

            if (process.env.HOST_IPV4) {
                addrs.push(Multiaddr(`/ip4/${process.env.HOST_IPV4}/udp/${port}`))
            }

            if (process.env.HOST_IPV6) {
                addrs.push(Multiaddr(`/ip6/${process.env.HOST_IPV6}/udp/${port}`))
            }

            return addrs
        }

        const getPeerId = async () => {
            let peerId
            if (Number.isInteger(options.id)) {
                if (Number.parseInt(process.env.DEMO_ACCOUNTS) < options.id)
                    return reject(
                        Error(
                            `Failed while trying to access demo account number ${
                                options.id
                            }. Please ensure that there are enough demo account specified in '.env'.`
                        )
                    )

                peerId = await privKeyToPeerId(process.env[`DEMO_ACCOUNT_${options.id}_PRIVATE_KEY`])
            } else if (await PeerId.isPeerId(options.peerId)) {
                peerId = options.peerId
            } else if (options['bootstrap-node']) {
                peerId = await privKeyToPeerId(process.env.FUND_ACCOUNT_PRIVATE_KEY)
            } else {
                peerId = await getFromDatabase()
            }

            return peerId
        }

        if (!db) {
            if (!options.peerInfo) return reject(Error('Invalid input parameter. Please set a valid peerInfo.'))
        }

        checkConfig()

        options.addrs = getAddrs()

        const peerInfo = new PeerInfo(await getPeerId())
        options.addrs.forEach(addr => peerInfo.multiaddrs.add(addr.encapsulate(`/${NAME}/${peerInfo.id.toB58String()}`)))

        return resolve(peerInfo)
    })
