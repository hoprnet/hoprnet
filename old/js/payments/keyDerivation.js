'use strict'

const secp256k1 = require('secp256k1')
const withIs = require('class-is')


const { hash, bufferXOR } = require('../messageDelivery/utils')
const Header = require('../messageDelivery/packet/header')

const SIGNATURE_LENGTH = 64
const HASH_LENGTH = 32

class KeyDerivation {
    constructor(buf) {
        this.buffer = buf
    }

    static get SIZE() {
        return 3 * HASH_LENGTH + SIGNATURE_LENGTH
    }

    get hashedKeyHalfA() {
        return this.buffer.slice(0, HASH_LENGTH)
    }

    get hashedKeyHalfB() {
        return this.buffer.slice(HASH_LENGTH, 2 * HASH_LENGTH)
    }

    get hashedKey() {
        return this.buffer.slice(2 * HASH_LENGTH, 3 * HASH_LENGTH)
    }

    get signature() {
        return this.buffer.slice(3 * HASH_LENGTH, 3 * HASH_LENGTH + SIGNATURE_LENGTH)
    }

    set signature(buf) {
        if (!Buffer.isBuffer(buf) || buf.length != SIGNATURE_LENGTH)
            throw Error('Invalid input arguments. Got \"' + typeof buf + '"\.')

        buf.copy(this.buffer, 3 * HASH_LENGTH, 0, SIGNATURE_LENGTH)
    }

    toBuffer() {
        return this.buffer
    }

    static create(secrets) {
        const keyDerivation = new KeyDerivation(
            Buffer.concat([
                hash(bufferXOR(Header.deriveTransactionKey(secrets[0]), Header.deriveTransactionKeyBlinding(secrets[0]))),
                hash(bufferXOR(Header.deriveTransactionKey(secrets[1]), Header.deriveTransactionKeyBlinding(secrets[0]))),
                hash(KeyDerivation.deriveKey(secrets)),
                Buffer.alloc(SIGNATURE_LENGTH)
            ])
        )

        keyDerivation.signature = Buffer.from(secp256k1.sign(hash(this.buffer.slice(0, 3 * HASH_LENGTH))))

        return keyDerivation
    }

    static deriveKey(Header, secrets) {
        const k_A = Header.deriveTransactionKey(secrets[0])
        const k_B = Header.deriveTransactionKey(secrets[1])

        return hash(bufferXOR(k_A, k_B))
    }
}

module.exports = withIs(KeyDerivation, { className: 'KeyDerivation', symbolName: '@validitylabs/hopper/KeyDerivation' })