'use strict'

const secp256k1 = require('secp256k1')
const crypto = require('crypto')
const Multihash = require('multihashes')
const bs58 = require('bs58')
const forEachRight = require('lodash.foreachright');

const createProvingValues = require('./createProvingValues')
const prg = require('../../prg')
const { bufferXOR } = require('../../../utils')
const constants = require('../../constants')

const p = require('./parameters')

module.exports = (Header, header, peerIds) => {
    function checkPeerIds() {
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
            let mul = Buffer.alloc(p.PRIVATE_KEY_LENGTH).fill(0)
            mul[p.PRIVATE_KEY_LENGTH - 1] = 1
            const G = secp256k1.publicKeyCreate(mul)

            secrets = []

            do {
                privKey = crypto.randomBytes(p.PRIVATE_KEY_LENGTH)
            } while (!secp256k1.privateKeyVerify(privKey))
            header.alpha
                .fill(secp256k1.publicKeyCreate(privKey), 0, p.COMPRESSED_PUBLIC_KEY_LENGTH)

            privKey.copy(mul, 0, 0, p.PRIVATE_KEY_LENGTH)

            peerIds.forEach((peerId, index) => {
                // parallel
                // thread 1
                const alpha = secp256k1.publicKeyTweakMul(G, mul)
                // secp256k1.publicKeyVerify(alpha)

                // thread 2
                const secret = secp256k1.publicKeyTweakMul(peerId.pubKey.marshal(), mul)
                // secp256k1.publicKeyVerify(secret)
                // end parallel

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
        const filler = Buffer.alloc(p.PER_HOP_SIZE * (constants.MAX_HOPS - 1)).fill(0)
        let length

        for (let index = 0; index < (constants.MAX_HOPS - 1); index++) {
            let { key, iv } = Header.derivePRGParameters(secrets[index], p.LAST_HOP_SIZE + (constants.MAX_HOPS - 1 - index) * p.PER_HOP_SIZE)

            length = (index + 1) * p.PER_HOP_SIZE

            bufferXOR(
                filler.slice(0, length),
                prg.createPRG(key, iv).digest(length)
            ).copy(filler, 0, 0, length)
        }

        return filler
    }

    function createBetaAndGamma(secrets, filler, identifier) {
        const tmp = Buffer.alloc(Header.BETA_LENGTH - p.PER_HOP_SIZE)

        forEachRight(secrets, (secret, index) => {
            const { key, iv } = Header.derivePRGParameters(secret)

            let paddingLength = (constants.MAX_HOPS - secrets.length) * p.PER_HOP_SIZE

            if (index === secrets.length - 1) {
                console.log(Multihash.decode(peerIds[index].id).digest.toString('hex'))

                header.beta
                    .fill(Multihash.decode(peerIds[index].id).digest, 0, p.DESINATION_SIZE)
                    .fill(identifier, p.DESINATION_SIZE, p.DESINATION_SIZE + p.IDENTIFIER_SIZE)

                if (paddingLength > 0) {
                    header.beta.fill(0, p.LAST_HOP_SIZE, paddingLength)
                }

                header.beta
                    .fill(
                        bufferXOR(
                            header.beta.slice(0, p.LAST_HOP_SIZE),
                            prg.createPRG(key, iv).digest(p.LAST_HOP_SIZE)
                        ),
                        0, p.LAST_HOP_SIZE)
                    .fill(filler, p.LAST_HOP_SIZE + paddingLength, Header.BETA_LENGTH)

            } else {
                console.log(Multihash.decode(peerIds[index + 1].id).digest.toString('hex'))
                tmp
                    .fill(header.beta, 0, Header.BETA_LENGTH - p.PER_HOP_SIZE)

                header.beta
                    .fill(Multihash.decode(peerIds[index + 1].id).digest, 0, p.ADDRESS_SIZE)
                    .fill(header.gamma, p.ADDRESS_SIZE, p.ADDRESS_SIZE + p.MAC_SIZE)
                    .fill(tmp, p.PER_HOP_SIZE, Header.BETA_LENGTH)

                createProvingValues(Header, header.beta.slice(p.ADDRESS_SIZE + p.MAC_SIZE, p.ADDRESS_SIZE + p.MAC_SIZE + p.PROVING_VALUES_SIZE), secrets, index)

                header.beta
                    .fill(
                        bufferXOR(
                            header.beta,
                            prg.createPRG(key, iv).digest(Header.BETA_LENGTH)
                        ), 0, Header.BETA_LENGTH)

                console.log(header.beta.slice(0, p.ADDRESS_SIZE).toString('hex'))

                // header.beta
                //     .copy(tmp, 0, 0, Header.BETA_LENGTH - (ADDRESS_SIZE + MAC_SIZE + PROVING_VALUES_SIZE))
                // tmp
                //     .copy(header.beta, ADDRESS_SIZE + MAC_SIZE + PROVING_VALUES_SIZE, 0, Header.BETA_LENGTH - (ADDRESS_SIZE + MAC_SIZE + PROVING_VALUES_SIZE))

                // console.log(peerIds[index + 1].toB58String())
                // Multihash
                //     .decode(peerIds[index + 1].id).digest
                //     .copy(header.beta, 0, 0, ADDRESS_SIZE)
                // header.gamma
                //     .copy(header.beta, ADDRESS_SIZE, 0, MAC_SIZE)

                // header.beta = bufferXOR(
                //     header.beta,
                //     prg.createPRG(key, iv).digest(Header.BETA_LENGTH)
                // )
            }

            header.gamma
                .fill(Header.createMAC(secret, header.beta), 0, p.MAC_SIZE)
        })
    }

    function printValues(header, secrets) {
        console.log(
            peerIds.reduce((str, peerId, index) => {
                str = str + '\nsecret[' + index + ']: ' + bs58.encode(secrets[index]) + '\n' +
                    'peerId[' + index + ']: ' + peerId.toB58String() + '\n'
                    + 'peerId[' + index + '] pubkey ' + bs58.encode(peerId.pubKey.marshal())

                return str
            }, header.toString()))
    }

    checkPeerIds()
    const secrets = generateKeyShares(peerIds)
    const identifier = crypto.randomBytes(p.IDENTIFIER_SIZE)
    const filler = generateFiller(secrets)
    createBetaAndGamma(secrets, filler, identifier)

    printValues(header, secrets)

    return {
        header: header,
        secrets: secrets,
        identifier: identifier
    }
}

