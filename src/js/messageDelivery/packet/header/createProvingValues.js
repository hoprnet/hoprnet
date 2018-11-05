'use strict'

const { randomBytes } = require('crypto')

const { deriveKey } = require('../../../payments/keyDerivation')
const { hash } = require('../../../utils')
const p = require('./parameters')

module.exports = (Header, buffer, secrets, index) => {
    function createWithTransaction(secrets) {
        buffer
            .fill(hash(Header.deriveTransactionKey(secrets[1])), 0, p.HASH_LENGTH)
            .fill(hash(deriveKey(Header,secrets.slice(0, 2))), p.HASH_LENGTH, 2 * p.HASH_LENGTH)
            .fill(deriveKey(Header, secrets.slice(1, 3)), 2 * p.HASH_LENGTH, p.PROVING_VALUES_SIZE)
    }

    function createWithoutTransaction(secret) {
        buffer
            .fill(hash(Header.deriveTransactionKey(secret)), 0, p.HASH_LENGTH)
            .fill(randomBytes(p.HASH_LENGTH + p.KEY_LENGTH), p.HASH_LENGTH, p.PROVING_VALUES_SIZE)
    }

    if (secrets.length <= 0)
        throw Error('Invalid number of hops. Got \"' + secrets.length + '\".')

    if (0 <= index && index < secrets.length - 2) {
        createWithTransaction(secrets.slice(index, index + 3))
    } else if (index == secrets.length - 2) {
        createWithoutTransaction(secrets[index + 1])
    } else {
        throw Error('Invalid index and/or invalid number of hops.')
    }
}