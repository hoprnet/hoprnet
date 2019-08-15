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

const libp2pCrypto = require('libp2p-crypto')

const { Type, Status, Message } = protons(fs.readFileSync(`${__dirname}/messages.proto`))

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

        this.relayers = new Map()
        this.destinations = new Map()

        this.node = opts.libp2p
    }

    async requestRelaying(peerInfo) {
        const conn = await this.node.dialProtocol(peerInfo, PROTOCOL_WEBRTC_TURN)

        const send = Pushable()
        pull(send, lp.encode(), conn)

        send.push(
            Message.encode({
                type: Type.ALLOCATION_REQUEST,
                origin: this.node.peerInfo.id.toBytes()
            })
        )

        this.relayers.set(peerInfo.id.toBytes().toString('base64'), {
            send
        })

        this.handleRequest(null, conn)
    }

    handleRequest(protocol, conn) {
        pull(
            conn,
            lp.decode(),
            pull.map(Message.decode),
            pull.drain(async message => {
                switch (message.type) {
                    case Type.ALLOCATION_REQUEST:
                        try {
                            await this.node.contentRouting.provide(await peerIdBytesToCID(message.origin))
                        } catch (err) {
                            return this.returnFail(conn)
                        }

                        let send = Pushable()

                        // prettier-ignore
                        pull(
                            send,
                            lp.encode(),
                            conn
                        )

                        this.destinations.set(message.origin.toString('base64'), {
                            send
                        })

                        send.push(
                            Message.encode({
                                type: Type.RESPONSE,
                                status: Status.OK
                            })
                        )

                        break
                    case Type.PACKET:
                        if (message.destination.equals(this.node.peerInfo.id.toBytes())) {
                            let found = this.relayedConnections.get(getId(message.origin, message.destination))

                            if (found) {
                                found.send.push(message.payload)
                            } else {
                                let send = Pushable()

                                const newConn = new Connection({
                                    sink: pull(
                                        pull.map(payload =>
                                            Message.encode({
                                                type: Type.Message,
                                                destination: message.origin,
                                                origin: message.destination,
                                                payload
                                            })
                                        ),
                                        lp.encode(),
                                        conn
                                    ),
                                    source: send
                                })

                                this.relayedConnections.set(getId(message.origin, message.destination), {
                                    send
                                })

                                send.push(message.payload)
                                this.emit('connection', newConn)
                            }
                        } else {
                            let found = this.destinations.get(message.destination.toString('base64'))

                            if (found) {
                                found.send.push(message)
                            } else {
                                this.returnFail(conn)
                            }
                        }
                        break
                    case Type.CLOSING_REQUEST:
                        if (found) found.send.end()

                        this.destinations.delete(message.destination.toString('base64'))
                        break
                    case Type.RESPONSE:
                        switch (message.status) {
                            case Status.OK:
                                console.log('OK')
                                break
                            case Status.FAIL:
                                console.log('FAIL!!!')
                                break
                        }
                        break
                    default:
                        this.returnFail(conn)

                        // Cancel stream by returning false
                        return false
                }
            })
        )
    }

    returnFail(conn) {
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
    }

    /**
     * @TODO
     * @param {PeerId} destination
     */
    async relay(destination) {
        //const cid = await peerIdToCID(destination)
        //const peerInfos = await this.node.contentRouting.findProviders(cid, { maxNumProviders: MAX_PROVIDERS })

        const peerInfos = this.node.bootstrapServers
        const innerConn = await Promise.race(peerInfos.map(peerInfo => this.node.dialProtocol(peerInfo, PROTOCOL_WEBRTC_TURN)))

        const send = Pushable()

        this.relayedConnections.set(getId(this.node.peerInfo.id.toBytes(), destination.toBytes()), {
            send
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
            source: send
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
