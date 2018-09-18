'use strict'

const secp256k1 = require('secp256k1')
const hkdf = require('futoin-hkdf')
const withIs = require('class-is')
const crypto = require('crypto')

const prp = require('./prp')
const prg = require('./prg')
const { bufferXOR_in_place, bufferADD_in_place } = require('../utils')

const KAPPA = 16
const HASH_LENGTH = KAPPA
const BLOCK_LENGTH = KAPPA

const PRIVATE_KEY_LENGTH = 32
const COMPRESSED_PUBLIC_KEY_LENGTH = 33
const HASH_KEY_PRG = 'P'
const HASH_KEY_PRP = 'W'
const HASH_KEY_BLINDINGS = 'B'
const HASH_KEY_HMAC = 'H'
const HASH_KEY_TAGGING = 'T'
const HASH_KEY_TX = 'Tx'
const HASH_KEY_TX_BLINDED = 'Tx_'

const DESTINATION_SIZE = 2
const MAC_SIZE = 1
const SIGNATURE_SIZE = 4
const PER_HOP_OVERHEAD = 2 //DESTINATION_SIZE + MAC_SIZE + 3 * SIGNATURE_SIZE

const IDENTIFIER_SIZE = 1


const NUMBER_OF_HOPS = 2
const MAX_NUMBER_OF_HOPS = 3

class Header {
    constructor(alpha, beta, gamma) {
        if (!Buffer.isBuffer(alpha) || !secp256k1.publicKeyVerify(alpha))
            throw Error('Wrong input values.')

        if (!Buffer.isBuffer(beta) || beta.length != Header.BETA_LENGTH)
            throw Error('Wrong input values.')

        if (!Buffer.isBuffer(gamma) || gamma.length < HASH_LENGTH)
            throw Error('Wrong input values.')

        this.alpha = alpha
        this.beta = beta
        this.gamma = gamma
    }

    static get NUMBER_OF_HOPS() {
        return NUMBER_OF_HOPS
    }

    static get MAX_NUMBER_OF_HOPS() {
        return MAX_NUMBER_OF_HOPS
    }

    static get BETA_LENGTH() {
        return (2 * MAX_NUMBER_OF_HOPS + IDENTIFIER_SIZE) * KAPPA
    }

    static get KAPPA() {
        return KAPPA
    }

    static get PUBLIC_KEY_LENGTH() {
        return COMPRESSED_PUBLIC_KEY_LENGTH
    }

    static get PRIVATE_KEY_LENGTH() {
        return PRIVATE_KEY_LENGTH
    }

    static get PER_HOP_OVERHEAD() {
        return PER_HOP_OVERHEAD
    }

    static fromBuffer(buf) {
        if (!Buffer.isBuffer(buf) || buf.length < prp.MIN_LENGTH + Header.BETA_LENGTH + Header.PUBLIC_KEY_LENGTH)
            throw Error('Wrong input. Expected a Buffer of size ' + prp.MIN_LENGTH + Header.BETA_LENGTH + Header.PUBLIC_KEY_LENGTH + ' bytes.')

        return new Header(
            buf.slice(0, Header.PUBLIC_KEY_LENGTH),
            buf.slice(Header.PUBLIC_KEY_LENGTH, Header.PUBLIC_KEY_LENGTH + Header.BETA_LENGTH),
            buf.slice(Header.PUBLIC_KEY_LENGTH + Header.BETA_LENGTH))
    }

    toBuffer() {
        return Buffer.concat([this.alpha, this.beta, this.gamma], this.alpha.length + this.beta.length + this.gamma.length)
    }

    forwardTransform(secretKey) {
        if (!secp256k1.privateKeyVerify(secretKey))
            throw Error('Invalid private key.')

        let derived_secret = secp256k1.publicKeyTweakMul(this.alpha, secretKey)

        if (this.gamma.compare(Header.createMAC(derived_secret, this.beta)) != 0)
            throw Error('General error')

        this.alpha = secp256k1.publicKeyTweakMul(this.alpha, Header.deriveBlinding(this.alpha, derived_secret))

        let { key, iv } = Header.derivePRGParameters(derived_secret)

        this.beta = bufferXOR_in_place(
            Buffer.concat([this.beta, Buffer.alloc(Header.PER_HOP_OVERHEAD * Header.KAPPA).fill(0)], this.beta.length + 2 * Header.KAPPA),
            prg.createPRG(key, iv).digest(Header.BETA_LENGTH + Header.PER_HOP_OVERHEAD * Header.KAPPA)
        )

        this.address = this.beta.slice(0, Header.KAPPA)
        this.derived_secret = derived_secret

        this.gamma = this.beta.slice(Header.KAPPA, 2 * Header.KAPPA)
        this.beta = this.beta.slice(2 * Header.KAPPA)

        return this

        // if (address === ) { // TODO
        //     cbDeliver(null, derived_secret)
        // } // else if (address === 'exit') { // TODO
        //   //  cbExit(null, this, derived_secret, address)
        // // } 
        // else {
        //     this.gamma = this.beta.slice(Header.KAPPA, 2 * Header.KAPPA)
        //     this.beta = this.beta.slice(2 * Header.KAPPA)

        //     cbForward(null, this, derived_secret, address)
        // }
    }

    static deriveTagParameters(secret) {
        if (!secret || !secp256k1.publicKeyVerify(secret))
            throw Error('General error')

        return hkdf(secret, prp.KEY_LENGTH + prp.IV_LENGTH, { salt: HASH_KEY_PRP })
    }

    static deriveCipherParameters(secret) {
        if (!secret || !secp256k1.publicKeyVerify(secret))
            throw Error('General error.')

        let keyAndIV = hkdf(secret, prp.KEY_LENGTH + prp.IV_LENGTH, { salt: HASH_KEY_PRP })

        let key = keyAndIV.slice(0, prp.KEY_LENGTH)
        let iv = keyAndIV.slice(prp.KEY_LENGTH)

        return { key, iv }
    }

    static derivePRGParameters(secret, blockOffset = 0) {
        if (!secret || !secp256k1.publicKeyVerify(secret))
            throw Error('General error.')

        let keyAndIV = hkdf(secret, prg.PRG_KEY_LENGTH + prg.PRG_IV_LENGTH, { salt: HASH_KEY_PRG })

        let key = keyAndIV.slice(0, HASH_LENGTH)
        let iv = bufferADD_in_place(
            Buffer.concat(
                [
                    keyAndIV.slice(HASH_LENGTH),
                    Buffer.alloc(4).fill(0)
                ], HASH_LENGTH),
            blockOffset)

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
                2 * Header.PUBLIC_KEY_LENGTH
            ),
            Header.PRIVATE_KEY_LENGTH, { salt: HASH_KEY_BLINDINGS })
    }

    static deriveTransactionKey (secret) {
        return hkdf(secret, BLOCK_LENGTH, { salt: HASH_KEY_TX })
    }

    static deriveTransactionKeyBlinding (secret) {
        return hkdf(secret, BLOCK_LENGTH, { salt: HASH_KEY_TX_BLINDED })
    }

    static deriveTaggingParameters (secret) {
        return hkdf(secret, BLOCK_LENGTH, { salt: HASH_KEY_TAGGING })
    }

    static createMAC(secret, msg) {
        let key = hkdf(secret, HASH_LENGTH, { salt: HASH_KEY_HMAC })
    
        return crypto.createHmac('sha256', key)
            .update(msg)
            .digest()
    }

    static generateHeader(publicKeys, destination) {
        return createHeader(publicKeys, destination)
    }
}
module.exports = withIs(Header, { className: 'Header' })

const createHeader = require('./headerCreation')
