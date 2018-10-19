'use strict'

const secp256k1 = require('secp256k1')
const hkdf = require('futoin-hkdf')
const withIs = require('class-is')
const crypto = require('crypto')
const last = require('lodash.last')
const Multihash = require('multihashes')
const bs58 = require('bs58')

const parallel = require('async/parallel')
const reduce = require('async/reduce')
var forEachRight = require('lodash.foreachright');



const prp = require('./prp')
const prg = require('./prg')
const { bufferXOR, bufferADD_in_place } = require('../utils')

const KAPPA = 16
const HASH_LENGTH = KAPPA

const PRIVATE_KEY_LENGTH = 32
const HASH_KEY_PRG = 'P'
const HASH_KEY_PRP = 'W'
const HASH_KEY_BLINDINGS = 'B'
const HASH_KEY_HMAC = 'H'
const HASH_KEY_TAGGING = 'T'
const HASH_KEY_TX = 'Tx'
const HASH_KEY_TX_BLINDED = 'Tx_'

const ADDRESS_SIZE = 32
const DESINATION_SIZE = ADDRESS_SIZE
const MAC_SIZE = 32
const PROVING_VALUES_SIZE = 0
const IDENTIFIER_SIZE = 16

const SIGNATURE_SIZE = 16
const TAG_SIZE = 16
const COMPRESSED_PUBLIC_KEY_LENGTH = 33


class Header {
    constructor(buf, options) {
        if (!Buffer.isBuffer(buf))
            throw Error('Expected a buffer. Got \"' + typeof buf + '\" instead.')
        if (!options || options.maxHops === undefined)
            throw Error('Invalid input values.')

        if (buf.length != Header.SIZE(options.maxHops))
            throw Error('Wrong input. Please provide a Buffer of size ' + Header.SIZE(options.maxHops) + ' .')

        this.buffer = buf

        this.MAX_HOPS = options.maxHops
    }

    set alpha(buf) {
        if (!Buffer.isBuffer(buf))
            throw Error('Expected a buffer. Got \"' + typeof buf + '\" instead.')

        buf.copy(this.buffer, 0, 0, COMPRESSED_PUBLIC_KEY_LENGTH)
    }
    get alpha() {
        return this.buffer.slice(0, COMPRESSED_PUBLIC_KEY_LENGTH)
    }

    set beta(buf) {
        if (!Buffer.isBuffer(buf))
            throw Error('Expected a buffer. Got \"' + typeof buf + '\" instead.')

        buf.copy(this.buffer, COMPRESSED_PUBLIC_KEY_LENGTH, 0, Header.BETA_LENGTH(this.MAX_HOPS))
    }
    get beta() {
        return this.buffer.slice(COMPRESSED_PUBLIC_KEY_LENGTH, COMPRESSED_PUBLIC_KEY_LENGTH + Header.BETA_LENGTH(this.MAX_HOPS))
    }

    set gamma(buf) {
        if (!Buffer.isBuffer(buf))
            throw Error('Expected a buffer. Got \"' + typeof buf + '\" instead.')

        buf.copy(this.buffer, COMPRESSED_PUBLIC_KEY_LENGTH + Header.BETA_LENGTH(this.MAX_HOPS), 0, MAC_SIZE)
    }
    get gamma() {
        return this.buffer.slice(COMPRESSED_PUBLIC_KEY_LENGTH + Header.BETA_LENGTH(this.MAX_HOPS), COMPRESSED_PUBLIC_KEY_LENGTH + Header.BETA_LENGTH(this.MAX_HOPS) + MAC_SIZE)
    }

    get address() {
        return Multihash.encode(this._address, 'sha2-256')
    }
    set address(buf) {
        if (!Buffer.isBuffer(buf))
            throw Error('Expected a buffer. Got \"' + typeof buf + '\" instead.')

        this._address = Buffer.isBuffer(this._address) ?
            buf.copy(this._address, 0, 0, ADDRESS_SIZE) :
            Buffer.alloc(ADDRESS_SIZE).fill(buf, 0, ADDRESS_SIZE)
    }

    static BETA_LENGTH(maxHops) {
        return (ADDRESS_SIZE + MAC_SIZE + PROVING_VALUES_SIZE) * (maxHops - 1) + DESINATION_SIZE + IDENTIFIER_SIZE
    }

    static SIZE(maxHops) {
        return COMPRESSED_PUBLIC_KEY_LENGTH + this.BETA_LENGTH(maxHops) + MAC_SIZE
    }

    static fromBuffer(buf, maxHops) {
        if (!Buffer.isBuffer(buf) || buf.length != Header.SIZE(maxHops))
            throw Error('Wrong input. Expected a Buffer of size ' + Header.SIZE(maxHops) + ' bytes.')

        return new Header(
            buf,
            { maxHops: maxHops }
        )
    }

    toBuffer() {
        return this.buffer
    }

    forwardTransform(secretKey, peerId) {
        if (!secp256k1.privateKeyVerify(secretKey))
            throw Error('Invalid private key.')

        this.derivedSecret = secp256k1.publicKeyTweakMul(this.alpha, secretKey)

        this.alpha = secp256k1.publicKeyTweakMul(this.alpha, Header.deriveBlinding(this.alpha, this.derivedSecret))

        if (this.gamma.compare(Header.createMAC(this.derivedSecret, this.beta)) != 0)
            throw Error('General error')

        const { key, iv } = Header.derivePRGParameters(this.derivedSecret)

        const tmp = Buffer
        .alloc(Header.BETA_LENGTH(this.MAX_HOPS) + ADDRESS_SIZE + MAC_SIZE + PROVING_VALUES_SIZE)
        .fill(this.beta, 0, Header.BETA_LENGTH(this.MAX_HOPS))
        .fill(0, Header.BETA_LENGTH(this.MAX_HOPS), ADDRESS_SIZE + MAC_SIZE + PROVING_VALUES_SIZE)
        // this.beta.copy(tmp, 0, 0, Header.BETA_LENGTH(this.MAX_HOPS))

        bufferXOR(
            tmp,
            prg.createPRG(key, iv).digest(Header.BETA_LENGTH(this.MAX_HOPS) + ADDRESS_SIZE + MAC_SIZE + PROVING_VALUES_SIZE)
        ).copy(tmp, 0, 0, Header.BETA_LENGTH(this.MAX_HOPS) + ADDRESS_SIZE + MAC_SIZE + PROVING_VALUES_SIZE)

        this.address = tmp.slice(0, ADDRESS_SIZE)

        this.gamma = tmp.slice(ADDRESS_SIZE, ADDRESS_SIZE + MAC_SIZE)

        this.beta = tmp.slice(ADDRESS_SIZE + MAC_SIZE + PROVING_VALUES_SIZE)

        return this
    }

    static deriveTagParameters(secret) {
        if (!secret || !secp256k1.publicKeyVerify(secret))
            throw Error('General error')

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
            iv: bufferADD_in_place(
                Buffer.concat(
                    [
                        keyAndIV.slice(prg.KEY_LENGTH, prg.KEY_LENGTH + prg.IV_LENGTH),
                        Buffer.alloc(4).fill(0)
                    ], HASH_LENGTH),
                Math.floor(byteOffset / prg.BLOCK_LENGTH)
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
                2 * COMPRESSED_PUBLIC_KEY_LENGTH
            ),
            PRIVATE_KEY_LENGTH, { salt: HASH_KEY_BLINDINGS })
    }

    static deriveTransactionKey(secret) {
        if (!secret || !secp256k1.publicKeyVerify(secret))
            throw Error('General error')

        return hkdf(secret, SIGNATURE_SIZE, { salt: HASH_KEY_TX })
    }

    static deriveTransactionKeyBlinding(secret) {
        if (!secret || !secp256k1.publicKeyVerify(secret))
            throw Error('General error')

        return hkdf(secret, SIGNATURE_SIZE, { salt: HASH_KEY_TX_BLINDED })
    }

    static deriveTaggingParameters(secret) {
        if (!secret || !secp256k1.publicKeyVerify(secret))
            throw Error('General error')

        return hkdf(secret, TAG_SIZE, { salt: HASH_KEY_TAGGING })
    }

    static createMAC(secret, msg) {
        if (!secret || !secp256k1.publicKeyVerify(secret))
            throw Error('General error')

        if (!msg || !Buffer.isBuffer(msg))
            throw Error('Invalid message. Please provide message as a Buffer.')

        const key = hkdf(secret, HASH_LENGTH, { salt: HASH_KEY_HMAC })

        return crypto.createHmac('sha256', key)
            .update(msg)
            .digest()
    }

    static createHeader(peerIds, options) {
        function checkInputValues() {
            // if (!options || options.maxHops === undefined || (peerIds.length + 1) > options.maxHops)
            //     throw Error('Invalid input values. Please provide a reasonable number of hops, where \"hops\" <= \"maxHops\".')

            if (!Array.isArray(peerIds))
                throw Error('Expected array of peerIds. Got ' + typeof publicKeys)

            peerIds.forEach((peerId, index) => {
                if (peerId === undefined || peerId.id === undefined || peerId.pubKey === undefined)
                    throw Error('Invalid peerId at index ' + index + '.')
            })
        }

        function generateKeyShares() {
            let done = false, alpha, secrets, privKey

            // Generate the Diffie-Hellman key shares and
            // the respective blinding factors for the
            // relays.
            // There exists a negligible, but NON-ZERO,
            // probability that the key share is chosen
            // such that it yields non-group elements.
            do {
                // initialize values
                let mul = Buffer.alloc(PRIVATE_KEY_LENGTH).fill(0)
                mul[PRIVATE_KEY_LENGTH - 1] = 1
                const G = secp256k1.publicKeyCreate(mul)

                secrets = []

                do {
                    privKey = crypto.randomBytes(PRIVATE_KEY_LENGTH)
                } while (!secp256k1.privateKeyVerify(privKey))
                header.alpha = secp256k1.publicKeyCreate(privKey)

                privKey.copy(mul, 0, 0, PRIVATE_KEY_LENGTH)

                peerIds.forEach((peerId, index) => {
                    const alpha = secp256k1.publicKeyTweakMul(G, mul)
                    const secret = secp256k1.publicKeyTweakMul(peerId.pubKey.marshal(), mul)

                    if (!secp256k1.publicKeyVerify(alpha) || !secp256k1.publicKeyVerify(secret))
                        return

                    mul = secp256k1.privateKeyTweakMul(mul, Header.deriveBlinding(alpha, secret))

                    if (!secp256k1.privateKeyVerify(mul))
                        return

                    secrets.push(secret)

                    if (index == peerIds.length - 1)
                        done = true
                })
            } while (!done)

            return secrets
        }

        function generateFiller(secrets) {
            const filler = Buffer.alloc((ADDRESS_SIZE + MAC_SIZE + PROVING_VALUES_SIZE) * (options.maxHops - 1)).fill(0)
            let length

            for (let index = 0; index < (options.maxHops - 1); index++) {
                let { key, iv } = Header.derivePRGParameters(secrets[index], DESINATION_SIZE + IDENTIFIER_SIZE + (options.maxHops - 1 - index) * (ADDRESS_SIZE + MAC_SIZE + PROVING_VALUES_SIZE))

                length = (index + 1) * (ADDRESS_SIZE + MAC_SIZE + PROVING_VALUES_SIZE)

                bufferXOR(
                    filler.slice(0, length),
                    prg.createPRG(key, iv).digest(length)
                ).copy(filler, 0, 0, length)
            }

            return filler
        }

        function createBetaAndGamma(secrets, filler, identifier) {
            const tmp = Buffer.alloc(Header.BETA_LENGTH(options.maxHops) - (ADDRESS_SIZE + MAC_SIZE + PROVING_VALUES_SIZE))

            forEachRight(secrets, (secret, index) => {
                const { key, iv } = Header.derivePRGParameters(secret)

                let paddingLength = (options.maxHops - secrets.length) * (ADDRESS_SIZE + MAC_SIZE + PROVING_VALUES_SIZE)

                if (index === secrets.length - 1) {
                    console.log(peerIds[index].toB58String())
                    Multihash
                        .decode(peerIds[index].id).digest
                        .copy(header.beta, 0, 0, ADDRESS_SIZE)
                        
                    identifier
                        .copy(header.beta, DESINATION_SIZE, 0, IDENTIFIER_SIZE)

                    if (paddingLength > 0) {
                        header.beta.fill(0, DESINATION_SIZE + IDENTIFIER_SIZE, paddingLength)
                    }
                    
                    bufferXOR(
                        header.beta.slice(0, DESINATION_SIZE + IDENTIFIER_SIZE),
                        prg.createPRG(key, iv).digest(DESINATION_SIZE + IDENTIFIER_SIZE)
                    )
                        .copy(header.beta, 0, 0, DESINATION_SIZE + IDENTIFIER_SIZE + paddingLength)

                    filler
                        .copy(header.beta, DESINATION_SIZE + IDENTIFIER_SIZE + paddingLength, 0, (options.maxHops - 1) * (ADDRESS_SIZE + MAC_SIZE + PROVING_VALUES_SIZE))
                } else {
                    header.beta
                        .copy(tmp, 0, 0, Header.BETA_LENGTH(options.maxHops) - (ADDRESS_SIZE + MAC_SIZE + PROVING_VALUES_SIZE))
                    tmp
                        .copy(header.beta, ADDRESS_SIZE + MAC_SIZE + PROVING_VALUES_SIZE, 0, Header.BETA_LENGTH(options.maxHops) - (ADDRESS_SIZE + MAC_SIZE + PROVING_VALUES_SIZE))

                    console.log(peerIds[index + 1].toB58String())
                    Multihash
                        .decode(peerIds[index + 1].id).digest
                        .copy(header.beta, 0, 0, ADDRESS_SIZE)
                    header.gamma
                        .copy(header.beta, ADDRESS_SIZE, 0, MAC_SIZE)

                    header.beta = bufferXOR(
                        header.beta,
                        prg.createPRG(key, iv).digest(Header.BETA_LENGTH(options.maxHops))
                    )
                }

                header.gamma = Header.createMAC(secret, header.beta)
            })
        }

        function printValues(header, secrets) {
            console.log(
                peerIds.reduce((str, peerId, index) => {
                    str = str + '\nsecret[' + index + ']: ' + bs58.encode(secrets[index]) + '\n' +
                        'peerId[' + index + ']: ' + peerId.toB58String() + '\n' +
                        + 'peerId[' + index + '] pubkey ' + bs58.encode(peerId.pubKey.marshal())

                    return str
                }, header.toString()))
        }

        checkInputValues()
        const header = new Header(Buffer.alloc(Header.SIZE(options.maxHops)), { maxHops: options.maxHops })

        const secrets = generateKeyShares(peerIds)
        const identifier = crypto.randomBytes(IDENTIFIER_SIZE)
        const filler = generateFiller(secrets)

        createBetaAndGamma(secrets, filler, identifier)

        // printValues(header, secrets)
        return {
            header: header,
            secrets: secrets,
            identifier: identifier
        }
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