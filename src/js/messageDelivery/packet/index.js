'use strict'

const withIs = require('class-is')
const PeerId = require('peer-id')

const Header = require('./header')
const Transaction = require('./transaction')
const Challenge = require('./challenge')
const Message = require('./message')

const parallel = require('async/parallel')
const series = require('async/series')

const c = require('../constants')
const { hash } = require('../../utils')

class Packet {
    constructor(_header, _transaction, _challenge, _message) {
        this.header = _header
        this.transaction = _transaction
        this.challenge = _challenge
        this.message = _message
    }

    static get SIZE() {
        return Header.SIZE + Transaction.SIZE + Challenge.SIZE + Message.SIZE
    }

    static createPacket(node, msg, intermediateNodes, destination, cb) {
        const { header, secrets, identifier } = Header.createHeader(intermediateNodes.concat([destination]))

        intermediateNodes.forEach((node, index) => console.log('Intermediate ' + index + ' : ' + node.toB58String()))
        console.log('Destination ' + destination.toB58String())

        parallel({
            challenge: (cb) => cb(null, Challenge.createChallenge(hash(Header.deriveTransactionKey(secrets[0])), node.peerInfo.id.privKey.marshal())),
            message: (cb) => cb(null, Message.createMessage(msg).onionEncrypt(secrets)),
            transaction: (cb) => {
                if (intermediateNodes.length > 0) {
                    cb(null, Transaction.createTransaction((secrets.length - 1) * c.RELAY_FEE, intermediateNodes[0].pubKey.marshal(), node.peerInfo.id.privKey.marshal()))
                } else {
                    throw Error('TODO: implement direct message transfer')
                }
            }
        }, (err, results) => {
            if (err) {
                cb(err) 
            } else {
                cb(null, new Packet(header, results.transaction, results.challenge, results.message))
            }
        })
    }

    forwardTransform(node, pubKeyPrevious, cb) {
        if (!this.transaction.verify())
            throw Error('TODO: No transaction')

        this.header.deriveSecret(node.peerInfo.id.privKey.marshal())

        series([
            (cb) => parallel([
                (cb) => {
                    const tag = Header.deriveTagParameters(this.header.derivedSecret)
                    if (node.seenTags.has(tag))
                        cb(Error('General error.'))

                    node.seenTags.add(tag)
                    cb()
                },
                (cb) => cb(!this.header.verify()),
            ], cb),
            (cb) => {
                this.header.extractHeaderInformation()
                cb()
            },
            (cb) => cb(!this.challenge.verify(pubKeyPrevious, hash(Header.deriveTransactionKey(this.header.derivedSecret)))),
            (cb) => parallel([
                (cb) => {
                    this.header.transformForNextNode()
                    cb()
                },
                (cb) => {
                    this.challenge.updateChallenge(this.header.hashedKeyHalf, node.peerInfo.id.privKey.marshal())
                    cb()
                },
                (cb) => {
                    this.message.decrypt(this.header.derivedSecret)
                    cb()
                }
            ], cb)
        ], (err) => {
            if (err) {
                cb(err)
            } else {
                cb(null, this)
            }
        })
    }

    getTargetPeerId() {
        console.log('Next Hop ' + this.header.address.toString('hex'))
        return PeerId.createFromBytes(this.header.address)
    }

    toBuffer() {
        return Buffer.concat([
            this.header.toBuffer(),
            this.transaction.toBuffer(),
            this.challenge.toBuffer(),
            this.message.toBuffer(),
        ], Packet.SIZE)
    }

    addTransaction(to, node, cb) {
        if (this.transaction.value < c.RELAY_FEE)
            throw Error('Insufficient funds.')

        this.transaction.forwardTransaction(this.transaction.value - c.RELAY_FEE, to.pubKey.marshal(), node.peerInfo.id.privKey.marshal())
        cb()
    }

    static fromBuffer(buf) {
        return new Packet(
            Header.fromBuffer(buf.slice(0, Header.SIZE)),
            Transaction.fromBuffer(buf.slice(Header.SIZE, Header.SIZE + Transaction.SIZE), true),
            Challenge.fromBuffer(buf.slice(Header.SIZE + Transaction.SIZE, Header.SIZE + Transaction.SIZE + Challenge.SIZE)),
            Message.fromBuffer(buf.slice(Header.SIZE + Transaction.SIZE + Challenge.SIZE, Header.SIZE + Transaction.SIZE + Challenge.SIZE + Message.SIZE))
        )
    }
}

module.exports = withIs(Packet, { className: 'Packet', symbolName: '@validitylabs/hopper/Packet' })