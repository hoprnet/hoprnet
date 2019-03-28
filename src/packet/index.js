'use strict'

const Header = require('./header')
const Transaction = require('../transaction')
const Challenge = require('./challenge')
const Message = require('./message')

const { waterfall } = require('neo-async')
const secp256k1 = require('secp256k1')

const { RELAY_FEE } = require('../constants')
const { hash, bufferXOR, log, pubKeyToEthereumAddress, getId, bufferToNumber, pubKeyToPeerId } = require('../utils')
const BN = require('bn.js')

/**
 * Encapsulates the internal representation of a packet
 */
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

    /**
     * Creates a new packet.
     * 
     * @param {Hop} node the node itself
     * @param {Buffer} msg the message that is sent through the network
     * @param {PeerId[]} path array of peerId that determines the route that
     * the packet takes
     * @param {Function(Error, Packet)} cb called afterward with `(err, packet)` if execution was successful,
     * otherwise with `(err)`
     */
    static createPacket(node, msg, path, cb) {
        const { header, secrets, identifier } = Header.createHeader(path)

        log(node.peerInfo.id, '---------- New Packet ----------')
        path
            .slice(0, Math.max(0, path.length - 1))
            .forEach((peerId, index) => log(node.peerInfo.id, `Intermediate ${index} : \x1b[34m${peerId.toB58String()}\x1b[0m`))
        log(node.peerInfo.id, `Destination    : \x1b[34m${path[path.length - 1].toB58String()}\x1b[0m`)
        log(node.peerInfo.id, '--------------------------------')


        const fee = (new BN(secrets.length - 1, 10)).imul(new BN(RELAY_FEE, 10))

        const challenge = Challenge
            .create(hash(Header.deriveTransactionKey(secrets[0])), fee)
            .sign(node.peerInfo.id)

        const message = Message.createMessage(msg).onionEncrypt(secrets)

        return node.paymentChannels.transfer(fee, path[0], (err, tx) => {
            if (err)
                return cb(err)

            log(node.peerInfo.id, `Encrypting with ${hash(bufferXOR(Header.deriveTransactionKey(secrets[0]), Header.deriveTransactionKey(secrets[1]))).toString('base64')}.`)

            const key = secp256k1.privateKeyTweakAdd(Header.deriveTransactionKey(secrets[0]), Header.deriveTransactionKey(secrets[1]))

            const channelId = getId(
                pubKeyToEthereumAddress(node.peerInfo.id.pubKey.marshal()),
                pubKeyToEthereumAddress(path[0].pubKey.marshal())
            )

            node.paymentChannels.addKey(channelId, key, (err) => {
                if (err) {
                    console.log(err)
                    return
                }

                return cb(null, new Packet(
                    header,
                    tx,
                    challenge,
                    message)
                )
            })
        })
    }

    /**
     * Checks the packet and transforms it such that it can be send to the next node.
     * 
     * @param {Hopr} node the node itself
     * @param {Function(Error)} callback called with `(err)` afterwards
     */
    forwardTransform(node, callback) {
        this.header.deriveSecret(node.peerInfo.id.privKey.marshal())

        let receivedMoney, channelId, forwardedFunds, nextHop

        waterfall([
            (cb) => this.hasTag(node.db, cb),
            (alreadyReceived, cb) => {
                if (alreadyReceived)
                    return cb(Error('General error.'))

                if (!this.header.verify())
                    return cb(Error('General error.'))

                this.header.extractHeaderInformation()

                return this.getSenderPeerId(cb)
            },
            (sender, cb) => {
                channelId = getId(
                    pubKeyToEthereumAddress(node.peerInfo.id.pubKey.marshal()),
                    pubKeyToEthereumAddress(sender.pubKey.marshal())
                )

                if (this.header.address.equals(node.peerInfo.id.pubKey.marshal())) {
                    node.paymentChannels.addKey(channelId, Header.deriveTransactionKey(this.header.derivedSecret), cb)
                } else {
                    node.paymentChannels.setOwnKeyHalf(channelId, this.header.hashedKeyHalf, Header.deriveTransactionKey(this.header.derivedSecret), cb)
                }
            },
            (cb) => node.paymentChannels.getChannel(channelId, cb),
            (record, cb) => {
                if (typeof record === 'function') {
                    // No record => no payment channel => something went wrong
                    cb = record

                    return cb(Error('General error.'))
                }

                receivedMoney = node.paymentChannels.getEmbeddedMoney(this.transaction, this._senderPeerId, record.currentValue)
                log(node.peerInfo.id, `Received \x1b[35m${receivedMoney.toString()} wei\x1b[0m.`)

                if (receivedMoney.lt(RELAY_FEE))
                    return cb(Error('Bad transaction.'))

                if (bufferToNumber(record.index) + 1 != bufferToNumber(this.transaction.index))
                    return cb(Error('General error.'))

                record.currentValue = this.transaction.value
                log(node.peerInfo.id, `currentValue ${(new BN(record.currentValue)).toString()}`)

                record.index = this.transaction.index

                return node.paymentChannels.setChannel(record, { channelId: channelId }, cb)
            },
            (cb) => {
                log(node.peerInfo.id, `Payment channel exists. Requested SHA256 pre-image of '${hash(this.header.derivedSecret).toString('base64')}' is derivable.`)

                this.oldChallenge = this.challenge
                this.message.decrypt(this.header.derivedSecret)

                this.getTargetPeerId(cb)
            },
            (_nextHop, cb) => {
                nextHop = _nextHop

                const nextChannelId = getId(
                    pubKeyToEthereumAddress(node.peerInfo.id.pubKey.marshal()),
                    pubKeyToEthereumAddress(nextHop.pubKey.marshal())
                )

                console.log(`channelId ${channelId.toString('hex')} nextChannelId ${nextChannelId.toString('hex')}`)

                if (this.header.address.equals(node.peerInfo.id.pubKey.marshal()))
                    return callback()

                forwardedFunds = receivedMoney.isub(new BN(RELAY_FEE, 10))

                this.challenge = Challenge.create(this.header.hashedKeyHalf, forwardedFunds).sign(node.peerInfo.id)

                this.header.transformForNextNode()

                node.paymentChannels.addKey(nextChannelId, this.header.encryptionKey, cb)
            },
            (cb) => node.paymentChannels.transfer(forwardedFunds, nextHop, cb),
            (tx, cb) => {
                log(node.peerInfo.id, `Encrypting with ${this.header.encryptionKey.toString('base64')}.`)

                this.transaction = tx
                console.log(`signature hash ${this.challenge.signatureHash.toString('base64')}`)
                node.paymentChannels.setChannelIdFromSignatureHash(this.challenge.signatureHash, channelId, cb)
            }
        ], callback)
    }

    /**
     * Computes the peerId of the next downstream node and caches it for later use.
     * 
     * @param {Function(Error, PeerId)} cb called when finished with `(err, peerId)` 
     */
    getTargetPeerId(cb) {
        if (this._targetPeerId)
            return cb(null, this._targetPeerId)

        pubKeyToPeerId(this.header.address, (err, peerId) => {
            if (err)
                return cb(err)

            this._targetPeerId = peerId

            cb(null, peerId)
        })
    }

    /**
     * Computes the peerId if the preceeding node and caches it for later use.
     * 
     * @param {Function(Error, PeerId)} cb called when finished with `(err, peerId)` 
     */
    getSenderPeerId(cb) {
        if (!this.header.derivedSecret)
            return cb(Error('Unable to compute the senders public key.'))

        if (this._previousPeerId)
            return cb(null, this._previousPeerId)

        pubKeyToPeerId(this.transaction.counterparty, (err, peerId) => {
            if (err)
                return cb(err)

            this._senderPeerId = peerId

            cb(null, peerId)
        })
    }

    toBuffer() {
        return Buffer.concat([
            this.header.toBuffer(),
            this.transaction.toBuffer(),
            this.challenge.toBuffer(),
            this.message.toBuffer(),
        ], Packet.SIZE)
    }

    /**
     * Checks whether the packet has already been seen.
     * 
     * @param {LevelDown} db a database instance
     * @param {Function(Error, Boolean)} cb called with `(err, found)` afterwards where `found` is true when
     * there is a record.
     */
    hasTag(db, cb) {
        const tag = Header.deriveTagParameters(this.header.derivedSecret)
        const key = Buffer.concat([Buffer.from('packet-tag-'), tag], 11 + 16)

        db.get(key, (err) => {
            if (err && !err.notFound)
                return cb(err)

            if (err.notFound)
                return db.put(key, '', (err) => cb(err, false))

            cb(null, true)
        })
    }

    static fromBuffer(buf) {
        if (!Buffer.isBuffer(buf) || buf.length != Packet.SIZE)
            throw Error(`Invalid input parameter. Expected a Buffer of size ${Packet.SIZE}. Got instead ${typeof buf}${Buffer.isBuffer(buf) ? ` of length ${buf.length} but expected length ${Packet.SIZE}` : ''}.`)

        return new Packet(
            Header.fromBuffer(buf.slice(0, Header.SIZE)),
            Transaction.fromBuffer(buf.slice(Header.SIZE, Header.SIZE + Transaction.SIZE)),
            Challenge.fromBuffer(buf.slice(Header.SIZE + Transaction.SIZE, Header.SIZE + Transaction.SIZE + Challenge.SIZE)),
            Message.fromBuffer(buf.slice(Header.SIZE + Transaction.SIZE + Challenge.SIZE, Header.SIZE + Transaction.SIZE + Challenge.SIZE + Message.SIZE))
        )
    }
}

module.exports = Packet