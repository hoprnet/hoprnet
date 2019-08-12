'use strict'

const secp256k1 = require('secp256k1')
const BN = require('bn.js')

const { RELAY_FEE } = require('../constants')
const { hash, bufferXOR, log, pubKeyToEthereumAddress, getId, bufferToNumber, pubKeyToPeerId } = require('../utils')

const Header = require('./header')
const Transaction = require('../transaction')
const Challenge = require('./challenge')
const Message = require('./message')

const PRIVATE_KEY_LENGTH = 32
const OPENING_TIMEOUT = 86400 * 1000

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
     */
    async static createPacket(node, msg, path) {
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

        const tx = await node.paymentChannels.transfer({
            amount: fee,
            to: path[0],
            channelId: getId(
                pubKeyToEthereumAddress(node.peerInfo.id.pubKey.marshal()),
                pubKeyToEthereumAddress(path[0].pubKey.marshal())
            ),
            key: secp256k1.privateKeyTweakAdd(Header.deriveTransactionKey(secrets[0]), Header.deriveTransactionKey(secrets[1]))
        })

        return new Packet(header, tx, challenge, message)
    }

    /**
    * Tries to get a previous transaction from the database. If there's no such one,
    * listen to the channel opening event for some time and throw an error if the
    * was not opened within `OPENING_TIMEOUT` ms.
    *
    * @param {Buffer} channelId ID of the channel
    */
    getPreviousTransaction(channelId, node) {
        return new Promise((resolve, reject) =>
            node.db.get(node.paymentChannels.RestoreTransaction(channelId))
                .then(resolve)
                .catch((err) => {
                    if (!err.notFound)
                        return reject(err)

                    const eventListener = node.paymentChannels.once(`opened ${channelId.toString('base64')}`, resolve)

                    setTimeout(() => {
                        eventListener.removeListener(resolve)
                        reject(Error(`Sender didn't send payment channel opening request for channel ${channelId.toString('hex')} in time.`))
                    }, OPENING_TIMEOUT)
                })
        )
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

        const [sender, target] = await Promise.all([
            this.getSenderPeerId(),
            this.getTargetPeerId()
        ])

        const channelId = getId(
            pubKeyToEthereumAddress(node.peerInfo.id.pubKey.marshal()),
            pubKeyToEthereumAddress(sender.pubKey.marshal())
        )

        await this.getPreviousTransaction(channelId, node);

        const [index, currentChallenge] = await Promise.all([
            node.db.get(node.paymentChannels.Index(channelId)),
            node.paymentChannels.getPreviousChallenges(channelId)
        ])

        log(node.peerInfo.id, `Database index ${index.toString('hex')} on channnel ${channelId.toString('hex')}.`)
        log(node.peerInfo.id, `Transaction index ${this.transaction.index.toString('hex')} on channnel ${channelId.toString('hex')}.`)

        if (bufferToNumber(index) + 1 != bufferToNumber(this.transaction.index))
            throw Error('General error.')

        const nextChannelId = getId(
            pubKeyToEthereumAddress(node.peerInfo.id.pubKey.marshal()),
            pubKeyToEthereumAddress(target.pubKey.marshal())
        )

        log(node.peerInfo.id, `Payment channel exists. Requested SHA256 pre-image of '${hash(this.header.derivedSecret).toString('base64')}' is derivable.`)

        this.message.decrypt(this.header.derivedSecret)
        this.oldChallenge = this.challenge

        if (this.header.address.equals(node.peerInfo.id.pubKey.marshal())) {
            await this.prepareDelivery(node, channelId, currentChallenge)
        } else {
            await this.prepareForward(node, channelId, currentChallenge, nextChannelId, target)
        }

        console.log('Finished transformation')
        return this
    }


    /**
     * Prepares the packet to deliver it.
     *
     * @param {Hopr} node the node itself
     * @param {Buffer} channelId the ID of the payment channel
     * @param {Buffer} curvePoint sum of the previous channel keys and pending challenges
     */
    async prepareDelivery(node, channelId, curvePoint) {
        const pubKeys = [
            secp256k1.publicKeyCreate(Header.deriveTransactionKey(this.header.derivedSecret))
        ]

        if (curvePoint)
            pubKeys.push(curvePoint)

        if (!this.transaction.curvePoint.equals(secp256k1.publicKeyCombine(pubKeys)))
            throw Error('General error.')

        let channelKey
        try {
            channelKey = await node.db.get(node.paymentChannels.ChannelKey(channelId))
        } catch (err) {
            if (!err.notFound)
                throw err

            channelKey = Buffer.alloc(PRIVATE_KEY_LENGTH, 0)
        }

        await node.db.batch()
            .put(node.paymentChannels.Transaction(channelId), this.transaction.toBuffer())
            .put(node.paymentChannels.Index(channelId), this.transaction.index)
            .put(node.paymentChannels.CurrentValue(channelId), this.transaction.value)
            .put(node.paymentChannels.ChannelKey(channelId), secp256k1.privateKeyTweakAdd(channelKey, Header.deriveTransactionKey(this.header.derivedSecret)))
            .write({ sync: true })
    }

    /**
     * Prepares the packet in order to forward it to the next node.
     *
     * @param {Hopr} node the node itself
     * @param {Buffer} channelId the ID of the payment channel
     * @param {Buffer} curvePoint sum of the previous channel keys and the pending challenges
     * @param {Buffer} nextChannelId the ID of the next downstream payment channel
     * @param {PeerId} target peer Id of the next node
     */
    async prepareForward(node, channelId, curvePoint, nextChannelId, target) {
        const pubKeys = [
            secp256k1.publicKeyCreate(Header.deriveTransactionKey(this.header.derivedSecret)),
            this.header.hashedKeyHalf
        ]
        if (curvePoint)
            pubKeys.push(curvePoint)

        if (!this.transaction.curvePoint.equals(secp256k1.publicKeyCombine(pubKeys)))
            throw Error('General error.')

        const receivedMoney = node.paymentChannels.getEmbeddedMoney(this.transaction, this._senderPeerId, await node.db.get(node.paymentChannels.CurrentValue(channelId)))

        log(node.peerInfo.id, `Received \x1b[35m${receivedMoney.toString()} wei\x1b[0m in channel \x1b[33m${channelId.toString('hex')}\x1b[0m.`)

        if (receivedMoney.lt(RELAY_FEE))
            throw Error('Bad transaction.')

        this.header.transformForNextNode()

        const forwardedFunds = receivedMoney.isub(new BN(RELAY_FEE, 10))

        this.challenge = Challenge
            .create(this.header.hashedKeyHalf, forwardedFunds)
            .sign(node.peerInfo.id)

        const tx = await node.paymentChannels.transfer({
            amount: forwardedFunds,
            to: target,
            channelId: nextChannelId,
            key: this.header.encryptionKey
        })

        await node.db.batch()
            .put(node.paymentChannels.Transaction(channelId), this.transaction.toBuffer())
            .put(node.paymentChannels.Index(channelId), this.transaction.index)
            .put(node.paymentChannels.CurrentValue(channelId), this.transaction.value)
            .put(node.paymentChannels.ChannelId(this.challenge.signatureHash), channelId)
            .put(node.paymentChannels.Challenge(channelId, this.header.hashedKeyHalf), Header.deriveTransactionKey(this.header.derivedSecret))
            .write({ sync: true })

        this.transaction = tx
    }

    /**
     * Computes the peerId of the next downstream node and caches it for later use.
     */
    async getTargetPeerId() {
        if (this._targetPeerId)
            return this._targetPeerId

        this._targetPeerId = await pubKeyToPeerId(this.header.address)

        return this._targetPeerId
    }

    /**
     * Computes the peerId if the preceeding node and caches it for later use.
     */
    async getSenderPeerId() {
        if (this._previousPeerId)
            return this._previousPeerId

        this._senderPeerId = await pubKeyToPeerId(this.transaction.counterparty)

        return this._senderPeerId
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

    /**
     * @returns the binary representation of the packet
     */
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