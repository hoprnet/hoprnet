import assert from 'assert'
import { randomBytes } from 'crypto'
import secp256k1 from 'secp256k1'
import { randomInteger, u8aToHex } from '@hoprnet/hopr-utils'
import * as utils from '.'
import { Signature } from '../types/primitives'

const generatePair = () => {
  // generate private key
  let privKey: Uint8Array
  do {
    privKey = randomBytes(32)
  } while (!secp256k1.privateKeyVerify(privKey))

  // get the public key in a compressed format
  const pubKey = secp256k1.publicKeyCreate(privKey)

  const address = secp256k1.publicKeyConvert(pubKey)

  return {
    privKey,
    pubKey,
    address
  }
}

const generateMsg = () => randomBytes(32)

describe('test utils', function () {
  it('should sign and verify messages', async function () {
    const { privKey, pubKey } = generatePair()

    for (let i = 0; i < 40; i++) {
      const message = generateMsg()
      const signature = Signature.create(message, privKey)
      assert(await utils.verify(message, signature, pubKey), `check that signature is verifiable`)

      let exponent = randomInteger(0, 7)
      let index = randomInteger(0, message.length - 1)

      message[index] = message[index] ^ (1 << exponent)

      if (await utils.verify(message, signature, pubKey)) {
        // @TODO change to assert.fail
        console.log(`found invalid signature <${u8aToHex(signature.serialize())}>, byte #${index}, bit #${exponent}`)
      }
    }
  })

  it('should compute a winning probability and convert it to float', function () {
    for (let i = 0; i < 10; i++) {
      let prob = Math.random()

      let winProb = utils.computeWinningProbability(prob)

      assert(Math.abs(prob - utils.getWinProbabilityAsFloat(winProb)) <= 0.0001)
    }
  })
})
