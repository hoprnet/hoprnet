'use strict'

const Header = require('./header')
const Transaction = require('../transaction')
const Challenge = require('./challenge')
const Message = require('./message')

const secp256k1 = require('secp256k1')

const { RELAY_FEE } = require('../constants')
const { hash, bufferXOR, log, pubKeyToEthereumAddress, getId, bufferToNumber, pubKeyToPeerId } = require('../utils')
const BN = require('bn.js')

const PRIVATE_KEY_LENGTH = 32

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

        log(node.peerInfo.id, `Encrypting with ${hash(bufferXOR(Header.deriveTransactionKey(secrets[0]), Header.deriveTransactionKey(secrets[1]))).toString('base64')}.`)

        node.paymentChannels.transfer({
            amount: fee,
            to: path[0],
            channelId: getId(
                pubKeyToEthereumAddress(node.peerInfo.id.pubKey.marshal()),
                pubKeyToEthereumAddress(path[0].pubKey.marshal())
            ),
            key: secp256k1.privateKeyTweakAdd(Header.deriveTransactionKey(secrets[0]), Header.deriveTransactionKey(secrets[1]))
        })
            .then((tx) =>
                cb(null, new Packet(header, tx, challenge, message))
            )
            .catch(cb)
    }

    /**
     * Checks the packet and transforms it such that it can be send to the next node.
     * 
     * @param {Hopr} node the node itself
     */
    async forwardTransform(node) {
        this.header.deriveSecret(node.peerInfo.id.privKey.marshal())

        if (await this.hasTag(node.db))
            throw Error('General error.')

        if (!this.header.verify())
            throw Error('General error.')

        this.header.extractHeaderInformation()

        const sender = await this.getSenderPeerId()
        const target = await this.getTargetPeerId()

        const channelId = getId(
            pubKeyToEthereumAddress(node.peerInfo.id.pubKey.marshal()),
            pubKeyToEthereumAddress(sender.pubKey.marshal())
        )

        try {
            await node.db.get(node.paymentChannels.RestoreTransaction(channelId))
        } catch (err) {
            if (err.notFound)
                throw Error('General error.')
            
            throw err
        }

        let channelKey
        try {
            channelKey = await node.db.get(node.paymentChannels.ChannelKey(channelId))
        } catch (err) {
            if (err.notFound) {
                channelKey = Buffer.alloc(PRIVATE_KEY_LENGTH, 0)
            } else {
                throw err
            }
        }

        // TODO:
        // What about reordering of incoming messages?
        const index = await node.db.get(node.paymentChannels.Index(channelId))
        if (bufferToNumber(index) + 1 != bufferToNumber(this.transaction.index))
            throw Error('General error.')

        const nextChannelId = getId(
            pubKeyToEthereumAddress(node.peerInfo.id.pubKey.marshal()),
            pubKeyToEthereumAddress(target.pubKey.marshal())
        )

        log(node.peerInfo.id, `Payment channel exists. Requested SHA256 pre-image of '${hash(this.header.derivedSecret).toString('base64')}' is derivable.`)

        console.log(`channelId ${channelId.toString('hex')} nextChannelId ${nextChannelId.toString('hex')}`)

        this.message.decrypt(this.header.derivedSecret)
        this.oldChallenge = this.challenge

        try {
            if (this.header.address.equals(node.peerInfo.id.pubKey.marshal())) {
                await this.prepareDelivery(node, channelId, channelKey)
            } else {
                await this.prepareForward(node, channelId, channelKey, nextChannelId, target)
            }
        } catch (err) {
            throw err
        }

        return this
    }

    /**
     * 
     */
    async prepareDelivery(node, channelId, channelKey) {
        console.log(Header.deriveTransactionKey(this.header.derivedSecret).toString('hex'))
        if (!this.transaction.curvePoint.equals(secp256k1.publicKeyCreate(secp256k1.privateKeyTweakAdd(Header.deriveTransactionKey(this.header.derivedSecret), channelKey)))) {
            console.log(Header.deriveTransactionKey(this.header.derivedSecret).toString('hex'))

            throw Error('General error.')
        }

        node.db.batch()
            .put(node.paymentChannels.Transaction(channelId), this.transaction.toBuffer())
            .put(node.paymentChannels.Index(channelId), this.transaction.index)
            .put(node.paymentChannels.CurrentValue(channelId), this.transaction.value)
            .put(node.paymentChannels.ChannelKey(channelId), secp256k1.privateKeyTweakAdd(channelKey, Header.deriveTransactionKey(this.header.derivedSecret)))
            .write()
    }

    /**
     * 
     */
    async prepareForward(node, channelId, channelKey, nextChannelId, target) {
        if (!this.transaction.curvePoint.equals(secp256k1.publicKeyCombine([secp256k1.publicKeyCreate(secp256k1.privateKeyTweakAdd(channelKey, Header.deriveTransactionKey(this.header.derivedSecret))), this.header.hashedKeyHalf])))
            throw Error('General error.')

        const receivedMoney = node.paymentChannels.getEmbeddedMoney(this.transaction, this._senderPeerId, await node.db.get(node.paymentChannels.CurrentValue(channelId)))

        log(node.peerInfo.id, `Received \x1b[35m${receivedMoney.toString()} wei\x1b[0m in channel \x1b[33m${channelId.toString('hex')}\x1b[0m.`)

        if (receivedMoney.lt(RELAY_FEE))
            return cb(Error('Bad transaction.'))

        this.header.transformForNextNode()

        const forwardedFunds = receivedMoney.isub(new BN(RELAY_FEE, 10))

        this.challenge = Challenge
            .create(this.header.hashedKeyHalf, forwardedFunds)
            .sign(node.peerInfo.id)

        node.db.batch()
            .put(node.paymentChannels.Transaction(channelId), this.transaction.toBuffer())
            .put(node.paymentChannels.Index(channelId), this.transaction.index)
            .put(node.paymentChannels.CurrentValue(channelId), this.transaction.value)
            .put(node.paymentChannels.Challenge(channelId, this.header.hashedKeyHalf), Header.deriveTransactionKey(this.header.derivedSecret))
            .write()

        this.transaction = await node.paymentChannels.transfer({
            amount: forwardedFunds,
            to: target,
            channelId: nextChannelId,
            key: this.header.encryptionKey
        })
    }

    /**
     * Computes the peerId of the next downstream node and caches it for later use.
     */
    getTargetPeerId() {
        if (this._targetPeerId)
            return this._targetPeerId

        return new Promise((resolve, reject) => pubKeyToPeerId(this.header.address, (err, peerId) => {
            if (err)
                return reject(err)

            this._targetPeerId = peerId

            resolve(peerId)
        }))
    }

    /**
     * Computes the peerId if the preceeding node and caches it for later use.
     */
    getSenderPeerId() {
        if (!this.header.derivedSecret)
            throw Error('Unable to compute the senders public key.')

        if (this._previousPeerId)
            return this._previousPeerId

        return new Promise((resolve, reject) => pubKeyToPeerId(this.transaction.counterparty, (err, peerId) => {
            if (err)
                return reject(err)

            this._senderPeerId = peerId

            resolve(peerId)
        }))
    }

    /**
     * Checks whether the packet has already been seen.
     */
    async hasTag(db) {
        const tag = Header.deriveTagParameters(this.header.derivedSecret)
        const key = Buffer.concat([Buffer.from('packet-tag-'), tag], 11 + 16)

        try {
            await db.get(key)
        } catch (err) {
            if (err.notFound) {
                return false
            }
            throw err
        }

        return true
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