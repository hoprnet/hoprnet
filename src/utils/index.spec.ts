import assert from 'assert'
import { randomBytes } from 'crypto'
// @ts-ignore-next-line
import secp256k1 from 'secp256k1'

import * as utils from '.'

const generatePairs = () => {
  // generate private key
  let privKey
  do {
    privKey = randomBytes(32)
  } while (!secp256k1.privateKeyVerify(privKey))

  // get the public key in a compressed format
  const pubKey = secp256k1.publicKeyCreate(privKey)

  return {
    privKey,
    pubKey
  }
}

describe('test utils', function() {
  it('should hash values', async function() {
    const testMsg = new Uint8Array([0, 0, 0, 0])

    assert.deepEqual(
      await utils.hash(testMsg),
      /* prettier-ignore */
      new Uint8Array([232,231,118,38,88,111,115,185,85,54,76,123,75,191,11,183,247,104,94,189,64,232,82,177,100,99,58,74,203,211,36,76])
    )
  })

  it('should sign and verify messages', async function() {
    const { privKey, pubKey } = generatePairs()

    const message = randomBytes(32)
    const signature = await utils.sign(message, privKey, pubKey)
    assert(await utils.verify(message, signature, pubKey), `check that signature is verifiable`)

    message[0] ^= 0xff
    assert(!(await utils.verify(message, signature, pubKey)), `check that manipulated message is not verifiable`)
  })

  it('should get address using public key', async function() {
    const { privKey, pubKey } = generatePairs()

    console.log(utils.pubKeyToAccountId(pubKey))
  })
})
