import { privKeyToPeerId } from './privKeyToPeerId'
import PeerId from 'peer-id'
import assert from 'assert'

const peerIdPrivKey = Uint8Array.from([
  160, 134, 102, 188, 161, 54, 60, 176, 11, 84, 2, 187, 235, 109, 71, 246, 184, 66, 150, 243, 187, 160, 242, 249, 91,
  16, 129, 223, 85, 136, 166, 19
])

const peerIdString = '16Uiu2HAm2VD6owCxPEZwP6Moe1jzapqziVeaTXf1h7jVzu5dW1mk'

describe(`converting privKey to peerId`, function () {
  it('convert back and forth', function () {
    const deserialized = privKeyToPeerId(peerIdPrivKey)

    assert(deserialized.toB58String() === peerIdString)
  })

  it('extract privKey from peerId and reconstruct it', async function () {
    const pId = await PeerId.create({ keyType: 'secp256k1' })

    const deserialized = privKeyToPeerId(pId.privKey.marshal())

    assert(deserialized.toB58String() === pId.toB58String())
  })

  it('check incorrect private key size', function () {
    const incorrectPrivKey = new Uint8Array(1).fill(0)

    assert.throws(() => privKeyToPeerId(incorrectPrivKey))
  })
})
