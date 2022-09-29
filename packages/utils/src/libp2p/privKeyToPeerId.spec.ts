import { privKeyToPeerId } from './privKeyToPeerId.js'
import { createFromPrivKey } from '@libp2p/peer-id-factory'
import { peerIdFromKeys, peerIdFromString } from '@libp2p/peer-id'
import { keys } from '@libp2p/crypto'

import assert from 'assert'

const peerIdPrivKey = Uint8Array.from([
  160, 134, 102, 188, 161, 54, 60, 176, 11, 84, 2, 187, 235, 109, 71, 246, 184, 66, 150, 243, 187, 160, 242, 249, 91,
  16, 129, 223, 85, 136, 166, 19
])

const peerIdPubKey = Uint8Array.from([
  2, 104, 233, 189, 80, 129, 44, 179, 234, 88, 101, 46, 60, 61, 48, 87, 12, 238, 242, 64, 28, 86, 221, 89, 184, 117,
  211, 7, 89, 143, 116, 64, 235
])

const prefixedPeerIdPrivKey = Uint8Array.from([
  8, 2, 18, 32, 160, 134, 102, 188, 161, 54, 60, 176, 11, 84, 2, 187, 235, 109, 71, 246, 184, 66, 150, 243, 187, 160,
  242, 249, 91, 16, 129, 223, 85, 136, 166, 19
])
const prefixedPeerIdPubKey = Uint8Array.from([
  8, 2, 18, 33, 2, 104, 233, 189, 80, 129, 44, 179, 234, 88, 101, 46, 60, 61, 48, 87, 12, 238, 242, 64, 28, 86, 221, 89,
  184, 117, 211, 7, 89, 143, 116, 64, 235
])

const secp256k1PrivKey = new keys.supportedKeys.secp256k1.Secp256k1PrivateKey(peerIdPrivKey, peerIdPubKey)

const peerIdPrivKeyHex = 'a08666bca1363cb00b5402bbeb6d47f6b84296f3bba0f2f95b1081df5588a613'

const peerIdString = '16Uiu2HAm2VD6owCxPEZwP6Moe1jzapqziVeaTXf1h7jVzu5dW1mk'

describe(`converting privKey to peerId`, function () {
  it('from hex string', function () {
    const deserilaizedWithPrefix = privKeyToPeerId(`0x${peerIdPrivKeyHex}`)
    const deserialized = privKeyToPeerId(peerIdPrivKeyHex)

    assert(deserialized.equals(deserilaizedWithPrefix))

    assert(deserialized.privateKey != null)
    assert(deserialized.publicKey != null)
    assert(deserialized.type === 'secp256k1')
    assert(deserialized.toString() === peerIdString)
  })

  it('from u8a', function () {
    const deserialized = privKeyToPeerId(peerIdPrivKey)

    assert(deserialized.privateKey != null)
    assert(deserialized.publicKey != null)
    assert(deserialized.type === 'secp256k1')
    assert(deserialized.toString() === peerIdString)
  })

  it('interoperability with libp2p methods', async function () {
    const pId = privKeyToPeerId(peerIdPrivKey)

    const deserializedFromKeys = await peerIdFromKeys(prefixedPeerIdPubKey, prefixedPeerIdPrivKey)
    const deserializedFromString = peerIdFromString(peerIdString)
    const deserializedFromPrivKey = await createFromPrivKey(secp256k1PrivKey)

    assert(pId.equals(deserializedFromPrivKey))
    assert(pId.equals(deserializedFromKeys))
    assert(pId.equals(deserializedFromString))
  })

  it('check invalid arguments', function () {
    const incorrectPrivKey = new Uint8Array(1).fill(0)
    assert.throws(() => privKeyToPeerId(incorrectPrivKey))

    const tooShortString = `0x12`
    assert.throws(() => privKeyToPeerId(tooShortString))

    const invalidCharacter = `0y`
    assert.throws(() => privKeyToPeerId(invalidCharacter))
  })
})
