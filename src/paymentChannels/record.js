'use strict'

const Transaction = require('../transaction')
const { numberToBuffer } = require('../utils')

module.exports = class Record {
    constructor(buf = Buffer.alloc(Record.SIZE)) {
        this.buf = buf
    }

    get tx() {
        return Transaction.fromBuffer(this.buf.slice(0, Transaction.SIZE))
    }

    set tx(newTx) {
        this.buf.slice(0, Transaction.SIZE).fill(newTx.toBuffer(), 0, Transaction.SIZE)
    }

    get restoreTx() {
        return Transaction.fromBuffer(this.buf.slice(Transaction.SIZE, Transaction.SIZE + Transaction.SIZE))
    }

    get index() {
        return this.buf.slice(Transaction.SIZE + Transaction.SIZE, Transaction.SIZE + Transaction.SIZE + Transaction.INDEX_LENGTH)
    }

    set index(newIndex) {
        if (typeof newIndex === 'number')
            newIndex = numberToBuffer(newIndex, Transaction.INDEX_LENGTH)

        if (!Buffer.isBuffer(newIndex) || newIndex.length !== Transaction.INDEX_LENGTH)
            throw Error(`Invalid argument. Unable to convert input to Buffer.`)

        this.index.fill(newIndex, 0, Transaction.INDEX_LENGTH)
    }

    get currentValue() {
        return this.buf.slice(Transaction.SIZE + Transaction.SIZE + Transaction.INDEX_LENGTH, Transaction.SIZE + Transaction.SIZE + Transaction.INDEX_LENGTH + Transaction.VALUE_LENGTH)
    }

    set currentValue(newValue) {
        if (typeof newValue === 'number')
            newValue = numberToBuffer(newValue, Transaction.VALUE_LENGTH)

        if (!Buffer.isBuffer(newValue) || newValue.length !== Transaction.VALUE_LENGTH)
            throw Error(`Invalid argument. Unable to convert input to Buffer.`)

        this.currentValue.fill(newValue, 0, Transaction.VALUE_LENGTH)
    }

    get totalBalance() {
        return this.buf.slice(Transaction.SIZE + Transaction.SIZE + Transaction.INDEX_LENGTH + Transaction.VALUE_LENGTH, Transaction.SIZE + Transaction.SIZE + Transaction.INDEX_LENGTH + Transaction.VALUE_LENGTH + Transaction.VALUE_LENGTH)
    }

    static get SIZE() {
        return Transaction.SIZE + Transaction.SIZE + Transaction.INDEX_LENGTH + Transaction.VALUE_LENGTH + Transaction.VALUE_LENGTH
    }

    static create(restoreTx, tx, index, currentValue, totalBalance) {
        const buf = Buffer.alloc(Record.SIZE)

        buf.fill(restoreTx.toBuffer(), Transaction.SIZE, Transaction.SIZE + Transaction.SIZE)
        buf.fill(totalBalance, Transaction.SIZE + Transaction.SIZE + Transaction.INDEX_LENGTH + Transaction.VALUE_LENGTH, Transaction.SIZE + Transaction.SIZE + Transaction.INDEX_LENGTH + Transaction.VALUE_LENGTH + Transaction.VALUE_LENGTH)

        const record = new Record(buf)

        if (tx)
            record.tx = tx

        if (index)
            record.index = index

        if (currentValue)
            record.currentValue = currentValue

        return record
    }

    
    toBuffer() {
        return this.buf
    }

    static fromBuffer(buf) {
        if (!Buffer.isBuffer(buf) || buf.length != Record.SIZE)
            throw Error(`Expected Buffer of size ${Record.SIZE}. Got ${typeof buf}${Buffer.isBuffer(buf) ? ` of size ${buf.length}` : ''}.`)

        return new Record(buf)
    }
}