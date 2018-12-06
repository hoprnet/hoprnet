'use strict'

const withIs = require('class-is')
const PeerId = require('peer-id')

const Header = require('./header')
const Transaction = require('../transaction')
const Challenge = require('./challenge')
const Message = require('./message')

const { parallel } = require('async')

const { deriveKey } = require('../paymentChannels/keyDerivation')
const { RELAY_FEE } = require('../constants')
const { hash, deepCopy } = require('../utils')

class Packet {
    constructor(header, transaction, challenge, message) {
        this.header = header
        this.transaction = transaction
        this.challenge = challenge
        this.message = message
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
            transaction: (cb) => node.paymentChannels.transfer((secrets.length - 1) * RELAY_FEE, intermediateNodes[0].id, cb)
        }, (err, results) => {
            if (err) { throw err }

            const encryptedTx = results.transaction.encrypt(deriveKey(Header, secrets.slice(0, 2)))

            node.pendingTransactions.set(hash(Header.deriveTransactionKey(secrets[0])).toString('base64'), encryptedTx)


            cb(null, new Packet(
                header,
                encryptedTx,
                results.challenge,
                results.message))
        })
    }

    forwardTransform(node, previousPeerId, cb) {
        const receivedMoney = node.paymentChannels.getEmbeddedMoney(previousPeerId, this.transaction)

        if (receivedMoney < RELAY_FEE)
            throw Error('Bad transaction.')

        this.header.deriveSecret(node.peerInfo.id.privKey.marshal())

        const tag = Header.deriveTagParameters(this.header.derivedSecret)
        if (node.seenTags.has(tag))
            throw Error('General error.')

        node.seenTags.add(tag)

        if (!this.header.verify)
            throw Error('General error.')

        this.header.extractHeaderInformation()

        if (!this.challenge.verify(previousPeerId.pubKey.marshal(), Header.deriveTransactionKey(this.header.derivedSecret)))
            throw Error('General error.')

        this.oldChallenge = deepCopy(this.challenge, Challenge)
        node.pendingTransactions.set(this.header.hashedKeyHalf.toString('base64'), deepCopy(this.transaction, Transaction))

        if (this.header.address.equals(node.peerInfo.id.toBytes())) {
            this.message.decrypt(this.header.derivedSecret, (err, message) => {
                this.message = message

                cb(null, this)
            })
        } else {
            parallel({
                transaction: (cb) => node.paymentChannels.transfer(receivedMoney - RELAY_FEE, this.getTargetPeerId(), cb),
                message: (cb) => this.message.decrypt(this.header.derivedSecret, cb),
                challenge: (cb) => this.challenge.updateChallenge(this.header.hashedKeyHalf, node.peerInfo.id.privKey.marshal(), cb),
                header: (cb) => this.header.transformForNextNode(cb)
            }, (err, results) => {
                if (err) { throw err }

                this.header = results.header
                this.transaction = results.transaction
                this.challenge = results.challenge
                this.message = results.message

                cb(null, this)
            })
        }




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