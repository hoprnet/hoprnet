import assert from 'assert'
import { randomBytes } from 'crypto'
import secp256k1 from 'secp256k1'
import { randomInteger, u8aEquals, u8aToHex } from '@hoprnet/hopr-utils'
import * as utils from '.'

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
  it('should hash values', async function () {
    const testMsg = new Uint8Array([0, 0, 0, 0])

    assert(
      u8aEquals(
        (await utils.hash(testMsg)).serialize(),
        new Uint8Array([
          232,
          231,
          118,
          38,
          88,
          111,
          115,
          185,
          85,
          54,
          76,
          123,
          75,
          191,
          11,
          183,
          247,
          104,
          94,
          189,
          64,
          232,
          82,
          177,
          100,
          99,
          58,
          74,
          203,
          211,
          36,
          76
        ])
      )
    )
  })

  it('should sign and verify signer', async function () {
    const { privKey, pubKey } = generatePair()

    const message = generateMsg()
    const signature = await utils.sign(message, privKey)
    const signer = await utils.signer(message, signature)

    assert(u8aEquals(pubKey, signer), `check that message is signed correctly`)
  })

  it('should sign and verify messages', async function () {
    const { privKey, pubKey } = generatePair()

    for (let i = 0; i < 40; i++) {
      const message = generateMsg()
      const signature = await utils.sign(message, privKey)
      assert(await utils.verify(message, signature, pubKey), `check that signature is verifiable`)

      let exponent = randomInteger(0, 7)
      let index = randomInteger(0, message.length - 1)

      message[index] = message[index] ^ (1 << exponent)

      if (await utils.verify(message, signature, pubKey)) {
        // @TODO change to assert.fail
        console.log(`found invalid signature <${u8aToHex(signature)}>, byte #${index}, bit #${exponent}`)
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
