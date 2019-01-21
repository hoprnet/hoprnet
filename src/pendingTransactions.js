'use strict'

const { decode } = require('multihashes')

const Transaction = require('./transaction')
const HASH_LENGTH = 32

class PendingTransactions {

    constructor(db) {
        this.db = db
    }

    addEncryptedTransaction(hashedKeyHalf, ownKeyHalf, tx, nextPeerId) {
        const key = `pending-${hashedKeyHalf.toString('base64')}`

        const record = arguments.length === 4 ?
            Buffer.concat(
                [
                    tx.toBuffer(),
                    ownKeyHalf,
                    decode(nextPeerId.toBytes()).digest
                ], Transaction.SIZE + Transaction.KEY_LENGTH + HASH_LENGTH) : ''

        this.db.put(key, record)
    }

    getEncryptedTransaction(hashedKeyHalf, cb) {
        const key = `pending-${hashedKeyHalf.toString('base64')}`

        this.db.get(key, (err, record) => {
            if (err && !err.notFound)
                throw err

            // Got an acknowledgement that fits to a self-signed transaction,
            // so we aren't interested in any decryption key or similar.
            if (record.length === 0)
                return

            const tx = Transaction.fromBuffer(record.slice(0, Transaction.SIZE), true)
            const ownKeyHalf = record.slice(Transaction.SIZE, Transaction.SIZE + Transaction.KEY_LENGTH)
            const hashedPubKey = record.slice(Transaction.SIZE + Transaction.KEY_LENGTH, Transaction.SIZE + Transaction.KEY_LENGTH + HASH_LENGTH)

            if (err && err.notFound) {
                cb(null)
            } else {
                cb(null, {
                    tx: tx,
                    ownKeyHalf: ownKeyHalf,
                    hashedPubKey: hashedPubKey
                })
            }
        })
    }
}

module.exports = PendingTransactions