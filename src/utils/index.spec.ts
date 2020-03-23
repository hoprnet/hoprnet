import assert from 'assert'
import { randomBytes } from 'crypto'
import secp256k1 from 'secp256k1'
import * as utils from '.'
import * as u8a from '../core/u8a'

const pair = {
  privKey: u8a.stringToU8a('0x9feaac2858974b0e16f6e3cfa7c21db6c7bbcd2094daa651ff3d5bb48a57b759'),
  pubKey: u8a.stringToU8a('0x03950056bd3c566eb3ac90b4e8cb0e93a648bf8000833161d679bd802505b224b5'),
  address: u8a.stringToU8a('0x81E1192eae6d7289A610956CaE1C4b76e083Eb39')
}

const generatePair = () => {
  // generate private key
  let privKey
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

const generateMsg = () => {
  return randomBytes(32)
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

  it('should sign and verify signer', async function() {
    const { privKey, pubKey } = generatePair()

    const message = generateMsg()
    const signature = await utils.sign(message, privKey)
    const signer = await utils.signer(message, signature)

    assert(u8a.u8aEquals(pubKey, signer), `check that message is signed correctly`)
  })

  it('should sign and verify messages', async function() {
    const { privKey, pubKey } = generatePair()

    const message = generateMsg()
    const signature = await utils.sign(message, privKey)
    assert(await utils.verify(message, signature, pubKey), `check that signature is verifiable`)

    message[0] ^= 0xff
    assert(!(await utils.verify(message, signature, pubKey)), `check that manipulated message is not verifiable`)
  })

  it('should get private key using public key', async function() {
    const pubKey = await utils.privKeyToPubKey(pair.privKey)

    assert(u8a.u8aEquals(pubKey, pair.pubKey))
  })

  it('should get address using public key', async function() {
    const address = await utils.pubKeyToAccountId(pair.pubKey)

    assert(u8a.u8aEquals(address, pair.address))
  })
})
