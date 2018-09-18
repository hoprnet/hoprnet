'use strict'

const last = require('lodash.last')
const secp256k1 = require('secp256k1')
const crypto = require('crypto')
const { bufferXOR_in_place } = require('../utils')

const prg = require('./prg')
const Header = require('./header')


module.exports = (publicKeys, destination) => {
    if (!Array.isArray(publicKeys))
            throw Error('Expected array of public keys. Got ' + typeof publicKeys)

        if (!publicKeys.reduce((acc, pKey) => {
            if (!Buffer.isBuffer(pKey))
                throw Error('Public keys must be provided as Buffers. Got ' + typeof pKey)

            return acc && secp256k1.publicKeyVerify(pKey)
        }, true))
            throw Error('One or more of the public keys seem to invalid.')

        if (!Buffer.isBuffer(destination) || destination.length === 0 || destination.length > (2 * Header.MAX_NUMBER_OF_HOPS * Header.KAPPA))
            throw Error('Wrong destination. Please provide a Buffer whose size satisfies: 0 < size < ' + 2 * Header.MAX_NUMBER_OF_HOPS * Header.KAPPA + '.')

        let { secrets, alpha } = generateKeyShares(publicKeys)

        secrets.forEach(secret => console.log('secret ' + secret.toString()))

        let nodes = publicKeys.reduce((nodes, pkey) => {
            nodes.push(crypto.createHash('sha256')
                .update(pkey)
                .digest())

            return nodes
        }, [])

        nodes.forEach(node => console.log(node.toString()))

        let identifier = crypto.randomBytes(Header.KAPPA)
        // secrets = secrets.reverse()
        let { beta, gamma } = createBetaAndGamma(destination, nodes, secrets, generateFiller(secrets), identifier)

        return {
            header: new Header(alpha, beta, gamma),
            secrets: secrets,
            identifier: identifier
        }
}

function generateKeyShares(pubKeys) {
    if (!Array.isArray(pubKeys))
        throw Error('Expected array of public keys. Got ' + typeof pubKeys)

    if (!pubKeys.reduce((acc, pKey) => {
        if (!Buffer.isBuffer(pKey))
            throw Error('Public keys must be provided as Buffers. Got ' + typeof pKey)

        if (pKey.length != Header.PUBLIC_KEY_LENGTH)
            throw Error('Public key must be of length ' + Header.PUBLIC_KEY_LENGTH + '.')

        return acc && secp256k1.publicKeyVerify(pKey)
    }, true))
        throw Error('One or more of the public keys seem to invalid.')

    let done = false, alpha, secrets, privKey, pubKey

    // Generate the Diffie-Hellman key shares and
    // the respective blinding factors for the
    // relays.
    // There exists a negligible, but NON-ZERO,
    // probability that the key share is chosen
    // such that it yields non-group elements.
    do {
        // initialize values
        let mul = Buffer.alloc(Header.PRIVATE_KEY_LENGTH).fill(0)
        mul[mul.length - 1] = 1
        secrets = []

        do {
            privKey = crypto.randomBytes(Header.PRIVATE_KEY_LENGTH)
        } while (!secp256k1.privateKeyVerify(privKey))
        pubKey = secp256k1.publicKeyCreate(privKey)
        alpha = pubKey

        pubKeys.forEach((pKey, index) => {
            let multiplicator = secp256k1.privateKeyTweakMul(privKey, mul)
            secrets.push(secp256k1.publicKeyTweakMul(pKey, multiplicator))
            alpha = secp256k1.publicKeyTweakMul(alpha, mul)

            if (!secp256k1.publicKeyVerify(alpha) || !secp256k1.publicKeyVerify(last(secrets)))
                return

            mul = secp256k1.privateKeyTweakMul(mul, Header.deriveBlinding(alpha, last(secrets)))

            if (!secp256k1.privateKeyVerify(mul))
                return

            if (index == pubKeys.length - 1)
                done = true
        })
    } while (!done)

    alpha = pubKey
    return { secrets, alpha }
}

function createBetaAndGamma(destination, nodes, secrets, filler, identifier) {
    return secrets.reduceRight((acc, secret, index, secrets) => {
        let { beta, gamma } = acc
        let { key, iv } = Header.derivePRGParameters(secret)
        if (index === secrets.length - 1) {
            beta = Buffer.concat(
                [
                    bufferXOR_in_place(
                        Buffer.concat([
                            destination,
                            identifier,
                            Buffer.alloc((2 * (Header.MAX_NUMBER_OF_HOPS - (index + 1)) + 3) * Header.KAPPA - destination.length).fill(0)
                        ], (2 * (Header.MAX_NUMBER_OF_HOPS - (index + 1)) + 3) * Header.KAPPA),
                        prg.createPRG(key, iv).digest((2 * (Header.MAX_NUMBER_OF_HOPS - Header.NUMBER_OF_HOPS) + 3) * Header.KAPPA))  
                    ,
                    filler
                ]
            )
        } else {
            beta = bufferXOR_in_place(
                prg.createPRG(key, iv).digest((2 * Header.MAX_NUMBER_OF_HOPS + 1) * Header.KAPPA),
                Buffer.concat([nodes[index].slice(0, Header.KAPPA), gamma, beta.slice(0, (2 * Header.MAX_NUMBER_OF_HOPS - 1) * Header.KAPPA)], Header.BETA_LENGTH)
            )
        }
        gamma = Header.createMAC(secret, beta)
        
        return { beta, gamma }
    }, {})
}

function generateFiller(secrets) {
    if (!Array.isArray(secrets))
        throw Error('Expected shared secrets as an array. Got ' + typeof secrets)

    if (!secrets.reduce((acc, secret) => {
        if (!Buffer.isBuffer(secret))
            throw Error('Shared secret must be provided as Buffers. Got ' + typeof secret)

        // Check whether the given secrets are group elements,
        // respectively potential public keys
        return acc && secp256k1.publicKeyVerify(secret)
    }, true))
        throw Error('One or more of the given shared secrets seems to be not a group element.')

    return secrets.slice(0, secrets.length - 1).reduce((filler, secret, index, secrets) => {
        let { key, iv } = Header.derivePRGParameters(secret, 2 * (Header.MAX_NUMBER_OF_HOPS - (index + 1)) + 3)

        filler = bufferXOR_in_place(
            filler,
            prg.createPRG(key, iv).digest(2 * (index + 1) * Header.KAPPA)
        )

        if (index === secrets.length - 1) {
            return filler
        } else {
            return Buffer.concat([filler, Buffer.alloc(2 * Header.KAPPA).fill(0)], filler.length + 2 * KAPPA)
        }
    }, Buffer.alloc(2 * Header.KAPPA).fill(0))
}