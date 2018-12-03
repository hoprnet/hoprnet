'use strict'

const withIs = require('class-is')
const PeerId = require('peer-id')

const Header = require('./header')
const Transaction = require('./transaction')
const Challenge = require('./challenge')
const Message = require('./message')

const { parallel, series } = require('async')

const { deriveKey } = require('./transaction/keyDerivation')
const { RELAY_FEE } = require('../constants')
const { hash, deepCopy } = require('../utils')

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
        const { header, secrets, identifier } = Header.createHeader(intermediateNodes.concat([destination]).map(peerInfo => peerInfo.id))

        console.log('\n\n[\'' + node.peerInfo.id.toB58String() + '\']: ---------- New Packet ----------')
        intermediateNodes.forEach((peerInfo, index) => console.log('[\'' + node.peerInfo.id.toB58String() + '\']: Intermediate ' + index + ' : ' + peerInfo.id.toB58String()))
        console.log('[\'' + node.peerInfo.id.toB58String() + '\']: Destination ' + destination.id.toB58String())

        parallel({
            challenge: (cb) => cb(null, Challenge.createChallenge(Header.deriveTransactionKey(secrets[0]), node.peerInfo.id.privKey.marshal())),
            message: (cb) => cb(null, Message.createMessage(msg).onionEncrypt(secrets)),
            transaction: (cb) => Transaction.createTransaction((secrets.length - 1) * RELAY_FEE, intermediateNodes[0].id, node, cb)
        }, (err, results) => {
            if (err) {
                cb(err)
            } else {
                const encryptedTx = results.transaction.encrypt(deriveKey(Header, secrets.slice(0,2)))

                node.pendingTransactions.set(hash(Header.deriveTransactionKey(secrets[0])).toString('base64'), encryptedTx)

                cb(null, new Packet(
                    header, 
                    encryptedTx, 
                    results.challenge, 
                    results.message))
            }
        })
    }

    forwardTransform(node, previousPeerId, cb) {
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
            (cb) => cb(!this.challenge.verify(previousPeerId.pubKey.marshal(), Header.deriveTransactionKey(this.header.derivedSecret))),
            (cb) => {
                // save transaction
                node.pendingTransactions.set(this.header.hashedKeyHalf.toString('base64'), deepCopy(this.transaction, Transaction))
                cb()
            },
            (cb) => parallel([
                (cb) => {
                    this.header.transformForNextNode()
                    cb()
                },
                (cb) => {
                    // Challenge Backup
                    this.oldChallenge = deepCopy(this.challenge, Challenge)

                    this.challenge.updateChallenge(this.header.hashedKeyHalf, node.peerInfo.id.privKey.marshal())
                    cb()
                },
                (cb) => {
                    this.message.decrypt(this.header.derivedSecret)
                    cb()
                }
            ], cb)
        ], (err) => err ? cb(err) : cb(null, this))
    }

    getTargetPeerId() {
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

    addTransaction(targetPeerId, node, cb) {
        if (this.transaction.value < RELAY_FEE)
            throw Error('Insufficient funds.')

        this.transaction.forwardTransaction(RELAY_FEE, targetPeerId, node, cb)
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