'use strict'

const secp256k1 = require('secp256k1')
const BN = require('bn.js')

const { RELAY_FEE } = require('../constants')
const { fromWei } = require('web3-utils')
const { hash, bufferXOR, log, pubKeyToEthereumAddress, getId, bufferToNumber, pubKeyToPeerId } = require('../utils')

const Header = require('./header')
const Transaction = require('../transaction')
const Challenge = require('./challenge')
const Message = require('./message')

const chalk = require('chalk')

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
    static async createPacket(node, msg, path) {
        const { header, secrets, identifier } = Header.createHeader(path)

        log(node.peerInfo.id, '---------- New Packet ----------')
        path.slice(0, Math.max(0, path.length - 1)).forEach((peerId, index) =>
            log(node.peerInfo.id, `Intermediate ${index} : ${chalk.blue(peerId.toB58String())}`)
        )
        log(node.peerInfo.id, `Destination    : ${chalk.blue(path[path.length - 1].toB58String())}`)
        log(node.peerInfo.id, '--------------------------------')

        const fee = new BN(secrets.length - 1, 10).imul(new BN(RELAY_FEE, 10))

        const challenge = Challenge.create(hash(Header.deriveTransactionKey(secrets[0])), fee).sign(node.peerInfo.id)

        const message = Message.createMessage(msg).onionEncrypt(secrets)

        log(
            node.peerInfo.id,
            `Encrypting with ${hash(bufferXOR(Header.deriveTransactionKey(secrets[0]), Header.deriveTransactionKey(secrets[1]))).toString('base64')}.`
        )

        const tx = await node.paymentChannels.transfer(
            path[0],
            fee,
            secp256k1.privateKeyTweakAdd(Header.deriveTransactionKey(secrets[0]), Header.deriveTransactionKey(secrets[1]))
        )

        return new Packet(header, tx, challenge, message)
    }

    /**
     * Tries to get a previous transaction from the database. If there's no such one,
     * listen to the channel opening event for some time and throw an error if the
     * was not opened within `OPENING_TIMEOUT` ms.
     *
     * @param {Buffer} channelId ID of the channel
     */
    async getPreviousTransaction(node, channelId, state) {
        const recordState = node.paymentChannels.TransactionRecordState
        switch (state.state) {
            case recordState.OPENING:
                state = await new Promise((resolve, reject) =>
                    setTimeout(
                        (() => {
                            const eventListener = node.paymentChannels.onceOpened(channelId, resolve)

                            return () => {
                                eventListener.removeListener(resolve)
                                reject(
                                    Error(`Sender didn't send payment channel opening request for channel ${chalk.yellow(channelId.toString('hex'))} in time.`)
                                )
                            }
                        })(),
                        OPENING_TIMEOUT
                    )
                )
            case recordState.OPEN:
                return state.lastTransaction

            default:
                throw Error(`Invalid state of payment channel ${chalk.yellow(channelId.toString('hex'))}. Got '${state.state}'`)
        }
    }

    /**
     * Checks the packet and transforms it such that it can be send to the next node.
     *
     * @param {Hopr} node the node itself
     */
    async forwardTransform(node) {
        this.header.deriveSecret(node.peerInfo.id.privKey.marshal())

        if (await this.hasTag(node.db)) throw Error('General error.')

        if (!this.header.verify()) throw Error('General error.')

        this.header.extractHeaderInformation()

        const [sender, target] = await Promise.all([this.getSenderPeerId(), this.getTargetPeerId()])

        const channelId = getId(pubKeyToEthereumAddress(node.peerInfo.id.pubKey.marshal()), pubKeyToEthereumAddress(sender.pubKey.marshal()))

        const currentState = await node.paymentChannels.state(channelId)

        // Update incoming payment channel
        const newState = {
            currentOffchainBalance: this.transaction.value,
            currentIndex: this.transaction.index,
            lastTransaction: this.transaction
        }

        if (currentState.state == node.paymentChannels.TransactionRecordState.PRE_OPENED) {
            // @TODO da fehlt noch was
            log(node.peerInfo.id, `incoming payment over pre-opened channel ${chalk.yellow(channelId.toString('hex'))}`)
            newState.state = node.paymentChannels.TransactionRecordState.OPEN
            newState.nonce = this.transaction.nonce
            newState.counterparty = sender.pubKey.marshal()
        } else {
            // Check whether we have an open channel in our database
            await this.getPreviousTransaction(node, channelId, currentState)
        }

        log(node.peerInfo.id, `Database index ${chalk.cyan(currentState.currentIndex.toString('hex'))} on channnel ${chalk.yellow(channelId.toString('hex'))}.`)
        log(node.peerInfo.id, `Transaction index ${chalk.cyan(this.transaction.index.toString('hex'))} on channnel ${chalk.yellow(channelId.toString('hex'))}.`)

        if (bufferToNumber(currentState.currentIndex) + 1 != bufferToNumber(this.transaction.index)) throw Error('General error.')

        log(
            node.peerInfo.id,
            `Payment channel exists. Requested SHA256 pre-image of ${chalk.green(hash(this.header.derivedSecret).toString('hex'))} is derivable.`
        )

        this.message.decrypt(this.header.derivedSecret)
        this.oldChallenge = this.challenge

        if (this.header.address.equals(node.peerInfo.id.pubKey.marshal())) {
            await this.prepareDelivery(node, currentState, newState, channelId)
        } else {
            await this.prepareForward(node, currentState, newState, channelId, target)
        }

        return this
    }

    /**
     * Prepares the delivery of the packet.
     *
     * @param {Hopr} node the node itself
     * @param {Object} state current off-chain state
     * @param {Object} newState future off-chain state
     * @param {Buffer} channelId the ID of the payment channel
     */
    async prepareDelivery(node, state, newState, channelId) {
        const challenges = [secp256k1.publicKeyCreate(Header.deriveTransactionKey(this.header.derivedSecret))]

        const previousChallenges = await node.paymentChannels.getPreviousChallenges(channelId)

        if (previousChallenges) challenges.push(previousChallenges)
        if (state.channelKey) challenges.push(secp256k1.publicKeyCreate(state.channelKey))

        if (!this.transaction.curvePoint.equals(secp256k1.publicKeyCombine(challenges))) throw Error('General error.')

        newState.channelKey = secp256k1.privateKeyTweakAdd(
            state.channelKey || Buffer.alloc(PRIVATE_KEY_LENGTH, 0),
            Header.deriveTransactionKey(this.header.derivedSecret)
        )

        await node.paymentChannels.setState(channelId, newState)
    }

    /**
     * Prepares the packet in order to forward it to the next node.
     *
     * @param {Hopr} node the node itself
     * @param {Object} state current off-chain state
     * @param {Object} newState future off-chain state
     * @param {Buffer} channelId the ID of the payment channel
     * @param {PeerId} target peer Id of the next node
     */
    async prepareForward(node, state, newState, channelId, target) {
        const challenges = [secp256k1.publicKeyCreate(Header.deriveTransactionKey(this.header.derivedSecret)), this.header.hashedKeyHalf]
        let previousChallenges = await node.paymentChannels.getPreviousChallenges(channelId)

        if (previousChallenges) challenges.push(previousChallenges)
        if (state.channelKey) challenges.push(secp256k1.publicKeyCreate(state.channelKey))

        if (!this.transaction.curvePoint.equals(secp256k1.publicKeyCombine(challenges))) throw Error('General error.')

        const receivedMoney = node.paymentChannels.getEmbeddedMoney(this.transaction.value, state.currentOffchainBalance, await this.getSenderPeerId())

        log(
            node.peerInfo.id,
            `Received ${chalk.magenta(`${fromWei(receivedMoney, 'ether').toString()} ETH`)} on channel ${chalk.yellow(channelId.toString('hex'))}.`
        )

        if (receivedMoney.lt(RELAY_FEE)) throw Error('Bad transaction.')

        this.header.transformForNextNode()

        const forwardedFunds = receivedMoney.isub(new BN(RELAY_FEE, 10))

        this.challenge = Challenge.create(this.header.hashedKeyHalf, forwardedFunds).sign(node.peerInfo.id)

        const [tx] = await Promise.all([
            node.paymentChannels.transfer(target, forwardedFunds, this.header.encryptionKey),
            node.paymentChannels.setState(channelId, newState),
            node.db
                .batch()
                .put(node.paymentChannels.ChannelId(this.challenge.signatureHash), channelId)
                .put(node.paymentChannels.Challenge(channelId, this.header.hashedKeyHalf), Header.deriveTransactionKey(this.header.derivedSecret))
                .write()
        ])

        this.transaction = tx
    }

    /**
     * Computes the peerId of the next downstream node and caches it for later use.
     */
    async getTargetPeerId() {
        if (this._targetPeerId) return this._targetPeerId

        return (this._targetPeerId = await pubKeyToPeerId(this.header.address))
    }

    /**
     * Computes the peerId if the preceeding node and caches it for later use.
     */
    async getSenderPeerId() {
        if (this._previousPeerId) return this._previousPeerId

        return (this._senderPeerId = await pubKeyToPeerId(this.transaction.counterparty))
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
        return Buffer.concat([this.header.toBuffer(), this.transaction.toBuffer(), this.challenge.toBuffer(), this.message.toBuffer()], Packet.SIZE)
    }

    static fromBuffer(buf) {
        if (!Buffer.isBuffer(buf) || buf.length != Packet.SIZE)
            throw Error(
                `Invalid input parameter. Expected a Buffer of size ${Packet.SIZE}. Got instead ${typeof buf}${
                    Buffer.isBuffer(buf) ? ` of length ${buf.length} but expected length ${Packet.SIZE}` : ''
                }.`
            )

        return new Packet(
            Header.fromBuffer(buf.slice(0, Header.SIZE)),
            Transaction.fromBuffer(buf.slice(Header.SIZE, Header.SIZE + Transaction.SIZE)),
            Challenge.fromBuffer(buf.slice(Header.SIZE + Transaction.SIZE, Header.SIZE + Transaction.SIZE + Challenge.SIZE)),
            Message.fromBuffer(buf.slice(Header.SIZE + Transaction.SIZE + Challenge.SIZE, Header.SIZE + Transaction.SIZE + Challenge.SIZE + Message.SIZE))
        )
    }
}

module.exports = Packet
