'use strict'

const crypto = require('crypto')
const Header = require('./header')
const secp256k1 = require('secp256k1')

function generateDummyKeys(amount) {
    let privKey, publicKey

    let result = []

    for (let i = 0; i < amount; i++) {
        do {
            privKey = crypto.randomBytes(Header.PRIVATE_KEY_LENGTH)
        } while (!secp256k1.privateKeyVerify(privKey))
        publicKey = secp256k1.publicKeyCreate(privKey)

        result.push({
            privKey: privKey, 
            publicKey: publicKey
        })
    }

    return result
}

module.exports.createTestHeader = function (amount) {
    let keys = generateDummyKeys(amount)

    let destination = crypto.randomBytes(2 * Header.KAPPA)

    console.log('destination: ' + destination)
    let nodes = [keys[0].publicKey, keys[0].publicKey]
    let result = Header.generateHeader(
        nodes,
        destination
    )
    
    result.keys = keys
    return result
}

// function test() {
//     const msglength = 234

//     crypto.randomBytes(msglength + KEY_LENGTH + IV_LENGTH, (err, buf) => {
//         const msg = buf.slice(0, msglength)
//         const key = buf.slice(msglength, msglength + KEY_LENGTH)
//         const iv = buf.slice(msglength + KEY_LENGTH)

//         console.log(msg)
//         const encrypted = module.exports.encrypt(msg, key, iv)
//         console.log(module.exports.decrypt(encrypted, key, iv))
//     })
// }
// test()