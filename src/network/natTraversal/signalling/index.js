'use strict'

const EventEmitter = require('events')

const fs = require('fs')
const protons = require('protons')

const pull = require('pull-stream')
const lp = require('pull-length-prefixed')
const Pushable = require('pull-pushable')

const CID = require('cids')
const Multihash = require('multihashes')
const Connection = require('interface-connection').Connection
const PeerId = require('peer-id')

const libp2pCrypto = require('libp2p-crypto')

const path = require('path')
const { Type, Status, Message } = protons(fs.readFileSync(path.resolve(__dirname, './messages.proto')))

const { PROTOCOL_WEBRTC_TURN } = require('../../../constants')

const MAX_PROVIDERS = 2

module.exports = class Signalling extends EventEmitter {
    /**
     * @param {object} opts
     * @param {libp2p} opts.libp2p
     */
    constructor(opts) {
        super()

        this.relayedConnections = new Map()
        this.destinations = new Map()

        this.node = opts.libp2p
    }

    async requestRelaying(peerInfo) {
        const conn = await this.node.dialProtocol(peerInfo, PROTOCOL_WEBRTC_TURN)

        const send = Pushable()

        this.initialiseStream(send, conn, peerInfo.id.toBytes())

        send.push(
            Message.encode({
                type: Type.ALLOCATION_REQUEST,
                origin: this.node.peerInfo.id.toBytes()
            })
        )

        this.handleRequest(null, conn)
    }

    initialiseStream(send, conn, to) {
        this.destinations.set(to.toString('base64'), {
            send
        })

        pull(
            send,
            // pull.map(message => {
            //     const decoded = Message.decode(message)
            //     if (decoded.destination)
            //         console.log(
            //             `self ${this.node.peerInfo.id.toB58String()} to ${PeerId.createFromBytes(
            //                 decoded.destination
            //             ).toB58String()} content ${decoded.payload.toString()}`
            //         )
            //     return message
            // }),
            lp.encode(),
            conn
        )
    }

    async processMessage(conn, message) {
        let send

        switch (message.type) {
            case Type.ALLOCATION_REQUEST:
                // try {
                //     await this.node.contentRouting.provide(await peerIdBytesToCID(message.origin))
                // } catch (err) {
                //     return this.returnFail(send)
                // }

                let found = this.destinations.get(message.origin.toString('base64'))

                if (!found) {
                    found = {
                        send: Pushable()
                    }

                    this.initialiseStream(found.send, conn, message.origin)
                }

                found.send.push(
                    Message.encode({
                        type: Type.RESPONSE,
                        status: Status.OK
                    })
                )

                break
            case Type.PACKET:
                if (message.destination.equals(this.node.peerInfo.id.toBytes())) {
                    let found = this.relayedConnections.get(getId(message.origin, message.destination))

                    if (!found) {
                        let destFound = this.destinations.get(message.relayer.toString('base64'))

                        if (!destFound) {
                            destFound = {
                                send: Pushable()
                            }

                            this.initialiseStream(destFound.send, conn, message.relayer)
                        }

                        found = {
                            receive: Pushable()
                        }

                        const newConn = new Connection({
                            sink: pull.drain(msg => {
                                destFound.send.push(
                                    Message.encode({
                                        type: Type.PACKET,
                                        destination: message.origin,
                                        origin: message.destination,
                                        payload: msg
                                    })
                                )
                            }),
                            source: found.receive
                        })
                        found.receive.push(message.payload)

                        this.relayedConnections.set(getId(message.origin, message.destination), found)
                        this.emit('connection', newConn)
                    } else {
                        found.receive.push(message.payload)
                    }
                } else {
                    let found = this.destinations.get(message.destination.toString('base64'))

                    if (found) {
                        message.relayer = this.node.peerInfo.id.toBytes()
                        found.send.push(Message.encode(message))
                    } else {
                        pull(
                            pull.once(
                                Message.encode({
                                    type: Type.RESPONSE,
                                    status: Status.FAIL
                                })
                            ),
                            lp.encode(),
                            conn
                        )

                        return false
                    }
                }
                break
            case Type.CLOSING_REQUEST:
                if (found) found.send.end()

                this.destinations.delete(message.origin.toString('base64'))
                break
            case Type.RESPONSE:
                switch (message.status) {
                    case Status.OK:
                        // console.log('OK')
                        break
                    case Status.FAIL:
                        // console.log('FAIL!!!')
                        break
                }
                break
            default:
                if (message.origin) {
                    const found = this.destinations.get(message.origin.toString('base64'))

                    if (found) {
                        found.send.push(
                            Message.encode({
                                type: Type.RESPONSE,
                                status: Status.FAIL
                            })
                        )
                        return false
                    }
                }

                pull(
                    pull.once(
                        Message.encode({
                            type: Type.RESPONSE,
                            status: Status.FAIL
                        })
                    ),
                    lp.encode(),
                    conn
                )

                // Cancel stream by returning false
                return false
        }
    }

    handleRequest(protocol, conn) {
        pull(
            /* prettier-ignore */
            conn,
            lp.decode(),
            pull.map(Message.decode),
            pull.drain(message => this.processMessage(conn, message))
        )
    }

    /**
     * @TODO
     * @param {PeerId} destination
     */
    async relay(destination) {
        //const cid = await peerIdToCID(destination)
        //const peerInfos = await this.node.contentRouting.findProviders(cid, { maxNumProviders: MAX_PROVIDERS })

        const peerInfos = this.node.bootstrapServers
        const innerConn = await Promise.race(
            /* prettier-ignore */
            peerInfos.map(peerInfo =>
                this.node.dialProtocol(peerInfo, PROTOCOL_WEBRTC_TURN))
        )

        const receive = Pushable()

        this.relayedConnections.set(getId(this.node.peerInfo.id.toBytes(), destination.toBytes()), {
            receive
        })

        return new Connection({
            sink: pull(
                pull.map(payload =>
                    Message.encode({
                        type: Type.PACKET,
                        payload,
                        destination: destination.toBytes(),
                        origin: this.node.peerInfo.id.toBytes()
                    })
                ),
                lp.encode(),
                innerConn
            ),
            source: receive
        })
    }
}

async function peerIdBytesToCID(peerIdBytes) {
    const hash = await libp2pCrypto.keys.unmarshalPublicKey(Multihash.decode(peerIdBytes).digest).hash()

    return new CID(1, 'raw', hash)
}

/**
 *
 * @param {PeerId} sender
 * @param {PeerId} receiver
 */
function getId(sender, receiver) {
    if (sender.equals(receiver)) throw Error('Sender and receiver must not be the same.')

    let result
    if (Buffer.compare(sender, receiver) < 0) {
        result = Buffer.concat([sender, receiver])
    } else {
        result = Buffer.concat([receiver, sender])
    }

    return result.toString('base64')
}
