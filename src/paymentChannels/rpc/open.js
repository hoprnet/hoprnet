'use strict'

const pull = require('pull-stream')
const lp = require('pull-length-prefixed')

const chalk = require('chalk')

const { randomBytes } = require('crypto')
const { toWei } = require('web3-utils')
const BN = require('bn.js')
const secp256k1 = require('secp256k1')

const { bufferToNumber, numberToBuffer, getId, pubKeyToEthereumAddress, addPubKey } = require('../../utils')
const { PROTOCOL_PAYMENT_CHANNEL } = require('../../constants')
const Transaction = require('../../transaction')

const OPENING_TIMEOUT = 6 * 60 * 1000

module.exports = self => {
    /**
     * Creates the restore transaction and stores it in the database.
     *
     * @param {Buffer} channelId ID of the payment channel
     */
    const prepareOpening = async channelId => {
        const restoreTx = Transaction.create(
            randomBytes(Transaction.NONCE_LENGTH),
            numberToBuffer(1, Transaction.INDEX_LENGTH),
            new BN(toWei('1', 'shannon')).toBuffer('be', Transaction.VALUE_LENGTH),

            // 0 is considered as infinity point / neutral element
            Buffer.alloc(33, 0)
        ).sign(self.node.peerInfo.id)

        self.setState(channelId, {
            state: self.TransactionRecordState.INITIALIZED,
            initialBalance: restoreTx.value,
            restoreTransaction: restoreTx
        })

        return restoreTx
    }

    /**
     * Sends the signed restore transaction to the counterparty and wait for
     * a signature from that party.
     *
     * @param {PeerId} to peerId of the counterparty
     * @param {Connection} conn an open stream to the counterparty
     * @param {Transaction} restoreTx the backup transaction
     */
    const getSignatureFromCounterparty = (to, conn, restoreTx) =>
        new Promise((resolve, reject) => {
            let resolved = false
            pull(
                pull.once(restoreTx.toBuffer()),
                lp.encode(),
                conn,
                lp.decode({
                    maxLength: Transaction.SIGNATURE_LENGTH + Transaction.RECOVERY_LENGTH
                }),
                pull.drain(data => {
                    if (resolved) return

                    if (!Buffer.isBuffer(data) || data.length != Transaction.SIGNATURE_LENGTH + Transaction.RECOVERY_LENGTH)
                        return reject(Error(`Counterparty ${chalk.blue(to.toB58String())} answered with an invalid message. Dropping message.`))

                    restoreTx.signature = data.slice(0, Transaction.SIGNATURE_LENGTH)
                    restoreTx.recovery = data.slice(Transaction.SIGNATURE_LENGTH)

                    if (
                        !secp256k1
                            .recover(restoreTx.hash, data.slice(0, Transaction.SIGNATURE_LENGTH), bufferToNumber(data.slice(Transaction.SIGNATURE_LENGTH)))
                            .equals(to.pubKey.marshal())
                    )
                        return reject(Error(`Counterparty ${chalk.blue(to.toB58String())} answered with an invalid signature. Dropping message.`))

                    resolve(restoreTx)
                    resolved = true

                    // Closes the stream
                    return false
                })
            )
        })

    /**
     * Opens a payment channel with the given party.
     *
     * @notice throws an exception in case the other party is not responding
     *
     * @param {PeerId | string} to peerId of multiaddr of the counterparty
     * @param {Transaction} [restoreTx] (optional) use that restore transaction instead
     * of creating a new one
     */
    const open = async (to, restoreTx) => {
        to = await addPubKey(to)

        const channelId = getId(
            /* prettier-ignore */
            pubKeyToEthereumAddress(to.pubKey.marshal()),
            pubKeyToEthereumAddress(self.node.peerInfo.id.pubKey.marshal())
        )

        let conn
        if (!restoreTx) {
            try {
                conn = await self.node.peerRouting.findPeer(to).then(peerInfo => self.node.dialProtocol(peerInfo, PROTOCOL_PAYMENT_CHANNEL))
            } catch (err) {
                throw Error(`Could not connect to peer ${chalk.blue(to.toB58String())} due to '${err.message}'.`)
            }

            try {
                restoreTx = await prepareOpening(channelId)
            } catch (err) {
                throw Error(
                    `Could not open payment channel ${chalk.yellow(channelId.toString('hex'))} to peer ${chalk.blue(to.toB58String())} due to '${err.message}'.`
                )
            }

            try {
                restoreTx = await getSignatureFromCounterparty(to, conn, restoreTx)
            } catch (err) {
                throw Error(`Unable to open a payment channel because counterparty ${chalk.blue(to.toB58String())} because '${err.message}'.`)
            }
        }

        const timeout = setTimeout(() => {
            throw Error(`Unable to open a payment channel because counterparty ${chalk.blue(to.toB58String())} is not answering with an appropriate response.`)
        }, OPENING_TIMEOUT)

        self.registerSettlementListener(channelId)
        self.registerOpeningListener(channelId)

        await self.setState(channelId, {
            restoreTransaction: restoreTx,
            state: self.TransactionRecordState.OPENING
        })

        return new Promise(resolve =>
            self.onceOpened(
                channelId,
                (() => {
                    const promise = self.contractCall(
                        self.contract.methods.createFunded(
                            restoreTx.nonce,
                            new BN(restoreTx.value).toString(),
                            restoreTx.signature.slice(0, 32),
                            restoreTx.signature.slice(32, 64),
                            bufferToNumber(restoreTx.recovery) + 27
                        )
                    )

                    return () => {
                        clearTimeout(timeout)
                        resolve(promise)
                    }
                })()
            )
        )
    }

    return open
}
