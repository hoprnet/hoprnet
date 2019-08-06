'use strict'

const fs = require('fs')
const protons = require('protons')

const pull = require('pull-stream')
const lp = require('pull-length-prefixed')
const Pushable = require('pull-pushable')

const { Message } = protons(fs.readFileSync(`${__dirname}/protos/message.proto`))
const { Request } = protons(fs.readFileSync(`${__dirname}/protos/request.proto`))
const { Response, Status } = protons(fs.readFileSync(`${__dirname}/protos/response.proto`))

const { PROTOCOL_WEBRTC_TURN, PROTOCOL_WEBRTC_TURN_REQUEST } = require('../../../constants')

const MAX_PROVIDERS = 2

module.exports = class Signalling {
    /**
     *
     * @param {object} opts
     */
    constructor(opts) {
        this.relayedConnections = new Map()

        this.node = opts.libp2p

        this.node.handle(PROTOCOL_WEBRTC_TURN, this.handleRelay)
        this.node.handle(PROTOCOL_WEBRTC_TURN_REQUEST, this.handRelayRequest)
    }

    handleRelay(err, conn) {
        const p = Pushable()

        pull(
            conn,
            lp.decode(),
            pull.map(buf => Message.decode(buf)),
            pull.drain(message => {
                let relayedConnection = this.relayedConnections.get(message.destination)

                if (!relayedConnection)
                    return true

                if (relayedConnection.conn) {
                    relayedConnection.lastUsed = Date.now()

                    this.relayedConnections.set(message.destination, relayedConnection)

                    pull(
                        conn,
                        pull.colle
                    )
                    // @TODO
                    this.node.dialProtocol(message.getDestination(), (err, conn) => {
                        if (err) throw err
                    })
                }
            })
        )

        // prettier-ignore
        pull(
            p,
            lp.encode(),
            conn
        )
    }

    handRelayRequest(err, conn) {
        if (err) throw err

        const p = Pushable()

        // prettier-ignore
        pull(
            p,
            lp.encode(),
            conn
        )

        pull(
            conn,
            lp.decode(),
            pull.map(buf => Request.decode(buf)),
            pull.drain(async request => {
                let relayedConnection = this.relayedConnections.get(request.peerId)

                if (!relayedConnection) {
                    this.node.contentRouting.provide(request.peerId, err => {
                        if (err) {
                            p.push(Response.encode({
                                status: Status.FAIL
                            }))
                            p.end()
                            throw err
                        }

                        this.relayedConnections.set(request.getPeerId(), {
                            lastUsed: Date.now()
                        })

                        p.push(Response.encode({
                            status: Status.OK
                        }))
                        p.end()
                    })
                } else {
                    this.relayedConnections.set(request.getPeerId(), {
                        lastUsed: Date.now()
                    })

                    p.push(Response.encode({
                        status: Status.OK
                    }))
                    p.end()
                }
            })
        )
    }

    /**
     * @TODO
     * @param {PeerInfo} peerInfo
     */
    requestRelaying(peerInfo) {
        this.node.dialProtocol(peerInfo, PROTOCOL_WEBRTC_TURN_REQUEST, (err, conn) => {
            if (err) throw err

            pull(
                pull.once(new RelayRequest(this.node.peerInfo.id)),
                lp.encode(),
                conn,
                lp.decode(),
                pull.map(buf => Response.decode(buf)),
                pull.collect((err, responses) => {
                    if (err) throw err

                    if (responses.length == 0 || responses[0].status !== Status.OK) throw Error('Got no response from node.')
                })
            )
        })
    }

    /**
     * @TODO
     * @param {PeerId} destination
     */
    relay(msgStream, destination) {
        this.node.contentRouting.findProviders(destination.toBytes(), { maxNumProviders: MAX_PROVIDERS }, (err, peerInfos) => {
            this.node.dialProtocol(peerInfos[0], PROTOCOL_WEBRTC_TURN, (err, conn) =>
                pull(
                    msgStream,
                    pull.map(payload => Message.encode({
                        destination,
                        payload
                    })),
                    lp.encode(),
                    conn,
                    lp.decode(),
                    pull.map(buf => Message.decode(buf).payload),
                    msgStream
                )
            )
        })
    }
}
