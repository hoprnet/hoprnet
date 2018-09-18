'use strict'

const crypto = require('crypto')
const EventEmitter = require('events');
const utils = require('web3').utils
const { bufferXOR_in_place, bufferXOR } = require('../utils')
const secp256k1 = require('secp256k1')

const HASH_LENGTH = 16
const SIGNATURE_LENGTH = 32
const TIMEOUT = 1000 * 60 // 1 minute

class PaymentChannelContract extends EventEmitter {
    constructor() {

    }
}

function InvalidTXContract(keyHalfA, keyHalfB, hashedKey, encryptedSignature, signature, pubKey) {
    if (!Buffer.isBuffer(signature) || signature.length !== SIGNATURE_LENGTH)
        throw Error('Invalid input')

    if (!Buffer.isBuffer(encryptedSignature) || encryptedSignature.length !== SIGNATURE_LENGTH)
        throw Error('Invalid input')

    if (!Buffer.isBuffer(pubKey) || !secp256k1.publicKeyVerify(pubKey))
        throw Error('Invalid input')

    if (!Buffer.isBuffer(hashedKey) || hashedKey.length !== HASH_LENGTH)
        throw Error('Invalid input')

    let _keyHalfA = utils.sha3(keyHalfA)
    let _keyHalfB = utils.sha3(keyHalfB)

    let key = utils.sha3(bufferXOR_in_place(keyHalfA, keyHalfB))

    if (hashedKey.compare(utils.sha3(utils.sha3(key))) === 0)
        throw Error('...')

    let txSignature = bufferXOR(encryptedSignature, key)

    if (!secp256k1.verify(Buffer.from('some tx'), txSignature, pubKey))
        throw Error('...')

    let bitStr = Buffer.concat([_keyHalfA, _keyHalfB, encryptedSignature, hashedKey], 3 * HASH_LENGTH + SIGNATURE_LENGTH)

    if (!secp256k1.verify(bitStr, signature, pubKey))
        throw Error('...') //TODO
}

function NoACKContract(hashedKey, signature, pubKey, nexthop) {
    if (!Buffer.isBuffer(hashedKey) || hashedKey.length !== HASH_LENGTH)
        throw Error('Invalid input')

    if (!Buffer.isBuffer(signature) || signature.length !== SIGNATURE_LENGTH)
        throw Error('Invalid input')

    if (!Buffer.isBuffer(pubKey) || !secp256k1.publicKeyVerify(pubKey))
        throw Error('Invalid input')

    if (!secp256k1.verify(hashedKey, signature, pubKey)) {
        throw Error()
    } else {
        setTimeout(() => {
            punish(nexthop)
        }, TIMEOUT)
    }
}

function WrongACKContract(signatureNextHop, pubKeyNextHop, key, signatureHop, pubKeyHop, hashedKey) {
    let bitStr = utils.sha3(
        Buffer.concat(
            [key, signatureHop],
            HASH_LENGTH + SIGNATURE_LENGTH
        )
    ).slice(2)

    if (!secp256k1.verify(Buffer.from(bitStr), signatureNextHop, pubKeyNextHop)) 
        throw Error('Drop transaction')

    if (!secp256k1.verify(hashedKey, signatureHop, pubKeyHop))
        throw Error('Invalid signature')

    if (!Buffer.isBuffer(signature) || signature.length !== SIGNATURE_LENGTH)
        throw Error('Invalid input')

    if (!utils.sha3(key).slice(2).compare(hashedKey) === 0) {
        setTimeout(() => {
            punish(pubKeyNextHop, 'hard')
        }, TIMEOUT)
    } else {
        // misuse of calling node
        punish(pubKeyHop)
    }
}

function punish(hop) {


}
