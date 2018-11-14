'use strict'

const secp256k1 = require('secp256k1')
const hkdf = require('futoin-hkdf')
const withIs = require('class-is')
const crypto = require('crypto')
const bs58 = require('bs58')
const multihashes = require('multihashes')

const createHeader = require('./createHeader')
const prp = require('../../crypto/prp')
const prg = require('../../crypto/prg')
const { bufferXOR, bufferADD } = require('../../utils')
const constants = require('../../constants')
const p = require('./parameters')


const MAC_KEY_LENGTH = 16
const HASH_KEY_PRG = 'P'
const HASH_KEY_PRP = 'W'
const HASH_KEY_BLINDINGS = 'B'
const HASH_KEY_HMAC = 'H'
const HASH_KEY_TAGGING = 'T'
const HASH_KEY_TX = 'Tx'
const HASH_KEY_TX_BLINDED = 'Tx_'

const TAG_SIZE = 16

class Header {
    constructor(buf) {
        if (!Buffer.isBuffer(buf))
            throw Error('Expected a buffer. Got \"' + typeof buf + '\" instead.')

        if (buf.length != Header.SIZE)
            throw Error('Wrong input. Please provide a Buffer of size ' + Header.SIZE + ' .')

        this.buffer = buf

        this.data = null
    }

    get alpha() {
        return this.buffer.slice(0, p.COMPRESSED_PUBLIC_KEY_LENGTH)
    }

    get beta() {
        return this.buffer.slice(p.COMPRESSED_PUBLIC_KEY_LENGTH, p.COMPRESSED_PUBLIC_KEY_LENGTH + Header.BETA_LENGTH)
    }

    get gamma() {
        return this.buffer.slice(p.COMPRESSED_PUBLIC_KEY_LENGTH + Header.BETA_LENGTH, p.COMPRESSED_PUBLIC_KEY_LENGTH + Header.BETA_LENGTH + p.MAC_SIZE)
    }

    get address() {
        return this.data ? multihashes.encode(this.data.slice(0, p.ADDRESS_SIZE), 'sha2-256') : null
    }

    get hashedKeyHalf() {
        return this.data ? this.data.slice(p.ADDRESS_SIZE, p.ADDRESS_SIZE + p.HASH_LENGTH) : null
    }

    get hashedDecryptionKey() {
        return this.data ? this.data.slice(p.ADDRESS_SIZE + p.HASH_LENGTH, p.ADDRESS_SIZE + p.HASH_LENGTH + p.HASH_LENGTH) : null
    }

    get encryptionKey() {
        return this.data ? this.data.slice(p.ADDRESS_SIZE + p.HASH_LENGTH + p.HASH_LENGTH, p.ADDRESS_SIZE + p.PROVING_VALUES_SIZE) : null
    }

    get derivedSecret() {
        return this.data ? this.data.slice(p.ADDRESS_SIZE + p.PROVING_VALUES_SIZE, p.ADDRESS_SIZE + p.PROVING_VALUES_SIZE + p.COMPRESSED_PUBLIC_KEY_LENGTH) : null
    }

    static get BETA_LENGTH() {
        return p.PER_HOP_SIZE * (constants.MAX_HOPS - 1) + p.LAST_HOP_SIZE
    }

    static get SIZE() {
        return p.COMPRESSED_PUBLIC_KEY_LENGTH + Header.BETA_LENGTH + p.MAC_SIZE
    }

    static fromBuffer(buf) {
        if (!Buffer.isBuffer(buf) || buf.length != Header.SIZE)
            throw Error('Wrong input. Expected a buffer of size ' + Header.SIZE + ' bytes.')

        return new Header(buf)
    }

    toBuffer() {
        return this.buffer
    }

    static createHeader(peerIds) {
        const header = new Header(Buffer.alloc(Header.SIZE))
        header.data = header.beta.slice(p.ADDRESS_SIZE + p.MAC_SIZE, p.PER_HOP_SIZE)

        return createHeader(Header, header, peerIds)
    }

    deriveSecret(secretKey) {
        if (!secp256k1.privateKeyVerify(secretKey))
            throw Error('Invalid private key.')

        this.data = this.data || Buffer.alloc(p.ADDRESS_SIZE + p.PROVING_VALUES_SIZE + p.COMPRESSED_PUBLIC_KEY_LENGTH)

        this.derivedSecret
            .fill(secp256k1.publicKeyTweakMul(this.alpha, secretKey), 0, p.COMPRESSED_PUBLIC_KEY_LENGTH)
    }

    verify() {
        return this.gamma.compare(Header.createMAC(this.derivedSecret, this.beta)) === 0
    }

    extractHeaderInformation() {
        const { key, iv } = Header.derivePRGParameters(this.derivedSecret)
        const tmp = Buffer
            .alloc(Header.BETA_LENGTH + p.PER_HOP_SIZE)
            .fill(this.beta, 0, Header.BETA_LENGTH)
            .fill(0, Header.BETA_LENGTH, Header.BETA_LENGTH + p.PER_HOP_SIZE)

        tmp
            .fill(
                bufferXOR(
                    tmp,
                    prg.createPRG(key, iv).digest(Header.BETA_LENGTH + p.PER_HOP_SIZE)
                ), 0, Header.BETA_LENGTH + p.PER_HOP_SIZE)

        this.data = this.data || Buffer.alloc(p.ADDRESS_SIZE + p.PROVING_VALUES_SIZE + p.COMPRESSED_PUBLIC_KEY_LENGTH)

        this.data
            .fill(tmp.slice(0, p.ADDRESS_SIZE), 0, p.ADDRESS_SIZE)
            .fill(tmp.slice(p.ADDRESS_SIZE + p.MAC_SIZE, p.PER_HOP_SIZE), p.ADDRESS_SIZE, p.ADDRESS_SIZE + p.PROVING_VALUES_SIZE)

        this.gamma
            .fill(tmp.slice(p.ADDRESS_SIZE, p.ADDRESS_SIZE + p.MAC_SIZE), 0, p.MAC_SIZE)

        this.beta
            .fill(tmp.slice(p.PER_HOP_SIZE, p.PER_HOP_SIZE + Header.BETA_LENGTH), 0, Header.BETA_LENGTH)
    }

    transformForNextNode() {
        this.alpha
            .fill(secp256k1.publicKeyTweakMul(this.alpha, Header.deriveBlinding(this.alpha, this.derivedSecret)), 0, p.COMPRESSED_PUBLIC_KEY_LENGTH)
    }

    static deriveTagParameters(secret) {
        if (!secret || !secp256k1.publicKeyVerify(secret))
            throw Error('General error.')

        return hkdf(secret, TAG_SIZE, { salt: HASH_KEY_TAGGING })
    }

    static deriveCipherParameters(secret) {
        if (!secret || !secp256k1.publicKeyVerify(secret))
            throw Error('General error.')

        const keyAndIV = hkdf(secret, prp.KEY_LENGTH + prp.IV_LENGTH, { salt: HASH_KEY_PRP })

        const key = keyAndIV.slice(0, prp.KEY_LENGTH)
        const iv = keyAndIV.slice(prp.KEY_LENGTH)

        return { key, iv }
    }

    static derivePRGParameters(secret, byteOffset = 0) {
        if (!secret || !secp256k1.publicKeyVerify(secret))
            throw Error('General error.')

        const keyAndIV = hkdf(secret, prg.KEY_LENGTH + prg.IV_LENGTH, { salt: HASH_KEY_PRG })

        const key = keyAndIV.slice(0, prg.KEY_LENGTH)
        const iv = {
            iv: Buffer.concat(
                [
                    keyAndIV.slice(prg.KEY_LENGTH, prg.KEY_LENGTH + prg.IV_LENGTH),
                    bufferADD(
                        Buffer.alloc(4).fill(0),
                        Math.floor(byteOffset / prg.BLOCK_LENGTH)
                    )
                ], prg.BLOCK_LENGTH
            ),
            byteOffset: byteOffset % prg.BLOCK_LENGTH
        }

        return { key, iv }
    }

    static deriveBlinding(alpha, secret) {
        if (!secret || !secp256k1.publicKeyVerify(secret))
            throw Error('General error.')

        if (!alpha || !secp256k1.publicKeyVerify(alpha))
            throw Error('General error.')
        return hkdf(
            Buffer.concat(
                [alpha, secret],
                2 * p.COMPRESSED_PUBLIC_KEY_LENGTH
            ),
            p.PRIVATE_KEY_LENGTH, { salt: HASH_KEY_BLINDINGS })
    }

    static deriveTransactionKey(secret) {
        if (!secret || !secp256k1.publicKeyVerify(secret))
            throw Error('General error')

        return hkdf(secret, p.KEY_LENGTH, { salt: HASH_KEY_TX })
    }

    static deriveTransactionKeyBlinding(secret) {
        if (!secret || !secp256k1.publicKeyVerify(secret))
            throw Error('General error')

        return hkdf(secret, p.KEY_LENGTH, { salt: HASH_KEY_TX_BLINDED })
    }

    static createMAC(secret, msg) {
        if (!secret || !secp256k1.publicKeyVerify(secret))
            throw Error('General error')

        if (!msg || !Buffer.isBuffer(msg))
            throw Error('Invalid message. Please provide message as a Buffer.')

        const key = hkdf(secret, MAC_KEY_LENGTH, { salt: HASH_KEY_HMAC })

        return crypto.createHmac('sha256', key)
            .update(msg)
            .digest()
    }

    toString() {
        return 'Header:\n' +
            '|-> Alpha:\n' +
            '|---> ' + bs58.encode(this.alpha) + '\n' +
            '|-> Beta:\n' +
            '|---> ' + bs58.encode(this.beta) + '\n' +
            '|-> Gamma:\n' +
            '|---> ' + bs58.encode(this.gamma) + '\n'
    }
}

module.exports = withIs(Header, { className: 'Header', symbolName: '@validitylabs/hopper/Header' })