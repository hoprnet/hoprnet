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

module.exports = self => async to => {
    to = await addPubKey(to)

    const channelId = getId(
        /* prettier-ignore */
        pubKeyToEthereumAddress(to.pubKey.marshal()),
        pubKeyToEthereumAddress(self.node.peerInfo.id.pubKey.marshal())
    )

    const prepareOpening = async () => {
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

    const getSignatureFromCounterparty = (conn, restoreTx) =>
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

    const open = async () => {
        let conn, restoreTx
        try {
            conn = await self.node.peerRouting.findPeer(to).then(peerInfo => self.node.dialProtocol(peerInfo, PROTOCOL_PAYMENT_CHANNEL))
        } catch (err) {
            throw Error(`Could not connect to peer ${chalk.blue(to.toB58String())} due to '${err.message}'.`)
        }

        try {
            restoreTx = await prepareOpening()
        } catch (err) {
            throw Error(
                `Could not open payment channel ${chalk.yellow(channelId.toString('hex'))} to peer ${chalk.blue(to.toB58String())} due to '${err.message}'.`
            )
        }

        const timeout = setTimeout(() => {
            throw Error(`Unable to open a payment channel because counterparty ${chalk.blue(to.toB58String())} is not answering with an appropriate response.`)
        }, OPENING_TIMEOUT)

        try {
            restoreTx = await getSignatureFromCounterparty(conn, restoreTx)
        } catch (err) {
            throw Error(`Unable to open a payment channel because counterparty ${chalk.blue(to.toB58String())} because '${err.message}'.`)
        }

        self.registerSettlementListener(channelId)
        self.registerOpeningListener(channelId)

        await self.setState(channelId, {
            state: self.TransactionRecordState.OPENING
        })

        return new Promise(resolve =>
            self.once(
                `opened ${channelId.toString('base64')}`,
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

    return open()
}
