'use strict'

const PeerId = require('peer-id')

const Header = require('./header')
const Transaction = require('../transaction')
const Challenge = require('./challenge')
const Message = require('./message')

const { parallel, waterfall } = require('neo-async')

const { RELAY_FEE } = require('../constants')
const { hash, bufferXOR, deepCopy, log, pubKeyToEthereumAddress, getId } = require('../utils')
const { BN } = require('web3').utils

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

    static createPacket(node, msg, path, cb) {
        const { header, secrets, identifier } = Header.createHeader(path)

        log(node.peerInfo.id, '---------- New Packet ----------')
        path
            .slice(0, Math.max(0, path.length - 1))
            .forEach((peerId, index) => log(node.peerInfo.id, `Intermediate ${index} : \x1b[34m${peerId.toB58String()}\x1b[0m`))
        log(node.peerInfo.id, `Destination    : \x1b[34m${path[path.length - 1].toB58String()}\x1b[0m`)
        log(node.peerInfo.id, '--------------------------------')


        const fee = (new BN(secrets.length - 1, 10)).imul(new BN(RELAY_FEE, 10))

        parallel({
            challenge: (cb) => cb(null, Challenge.createChallenge(Header.deriveTransactionKey(secrets[0]), node.peerInfo.id)),
            message: (cb) => cb(null, Message.createMessage(msg).onionEncrypt(secrets)),
            transaction: (cb) => node.paymentChannels.transfer(fee, path[0], cb)
        }, (err, results) => {
            if (err) { throw err }

            log(node.peerInfo.id, `Encrypting with ${hash(bufferXOR(Header.deriveTransactionKey(secrets[0]), Header.deriveTransactionKey(secrets[1]))).toString('base64')}.`)
            const encryptedTx = results.transaction.encrypt(hash(bufferXOR(Header.deriveTransactionKey(secrets[0]), Header.deriveTransactionKey(secrets[1]))))

            node.pendingTransactions.addEncryptedTransaction(
                hash(Header.deriveTransactionKey(secrets[0]))
            )

            cb(null, new Packet(
                header,
                encryptedTx,
                results.challenge,
                results.message))
        })
    }

    forwardTransform(node, previousPeerId, cb) {
        let receivedMoney

        const channelId = getId(
            pubKeyToEthereumAddress(node.peerInfo.id.pubKey.marshal()),
            pubKeyToEthereumAddress(previousPeerId.pubKey.marshal())
        )

        waterfall([
            (cb) => node.paymentChannels.getEmbeddedMoney(channelId, this.transaction, cb),
            (_receivedMoney, cb) => {
                receivedMoney = _receivedMoney
                log(node.peerInfo.id, `Received \x1b[35m${receivedMoney.toString()} wei\x1b[0m.`)

                this.header.deriveSecret(node.peerInfo.id.privKey.marshal())

                this.hasTag(node, cb)
            },
            (alreadyReceived, cb) => {
                if (alreadyReceived) {
                    throw Error('General error.')
                }

                if (!this.header.verify())
                    throw Error('General error.')

                this.header.extractHeaderInformation()

                if (!this.challenge.verify(previousPeerId, Header.deriveTransactionKey(this.header.derivedSecret)))
                    throw Error('General error.')

                log(node.peerInfo.id, `Challenge is valid. Requested SHA256 pre-image of '${Challenge.deriveHashedKey(this.header.derivedSecret).toString('base64')}' is derivable.`)

                this.oldChallenge = deepCopy(this.challenge, Challenge)

                if (this.header.address.equals(node.peerInfo.id.toBytes())) {
                    this.message.decrypt(this.header.derivedSecret, (err, message) => {
                        this.message = message

                        cb(null, this)
                    })
                } else {
                    if (receivedMoney.lt(RELAY_FEE))
                        throw Error('Bad transaction.')

                    node.pendingTransactions.addEncryptedTransaction(
                        this.header.hashedKeyHalf,
                        Header.deriveTransactionKey(this.header.derivedSecret),
                        deepCopy(this.transaction, Transaction),
                        this.getTargetPeerId()
                    )

                    node.paymentChannels.setChannel({
                        index: this.transaction.index,
                        currentValue: this.transaction.value
                    }, channelId, (err) => {
                        if (err)
                            throw err

                        const forwardedFee = receivedMoney.isub(new BN(RELAY_FEE, 10))

                        parallel({
                            transaction: (cb) => node.paymentChannels.transfer(forwardedFee, this.getTargetPeerId(), (err, tx) => {
                                if (err)
                                    throw err

                                log(node.peerInfo.id, `Encrypting with  '${this.header.encryptionKey.toString('base64')}'.`)
                                cb(null, tx.encrypt(this.header.encryptionKey))
                            }),
                            message: (cb) => this.message.decrypt(this.header.derivedSecret, cb),
                            challenge: (cb) => this.challenge.updateChallenge(this.header.hashedKeyHalf, node.peerInfo.id, cb),
                            header: (cb) => this.header.transformForNextNode(cb)
                        }, (err, results) => {
                            if (err) { throw err }

                            cb(null, Object.assign(this, results))
                        })
                    })
                }
            }
        ], cb)
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

    hasTag(node, cb) {
        const tag = Header.deriveTagParameters(this.header.derivedSecret)

        const key = `packet-tag-${tag.toString('base64')}`

        node.db.get(key, (err, result) => {
            if (err) {
                if (err.notFound) {
                    node.db.put(key, '', (err) => cb(null, false))
                } else {
                    throw err
                }
            } else {
                cb(null, true)
            }
        })
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

module.exports = Packet