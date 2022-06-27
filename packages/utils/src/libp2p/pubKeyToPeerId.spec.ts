import { pubKeyToPeerId } from './pubKeyToPeerId.js'
import { createFromPubKey } from '@libp2p/peer-id-factory'
import { peerIdFromKeys, peerIdFromString } from '@libp2p/peer-id'
import { keys } from '@libp2p/crypto'
import assert from 'assert'
import { u8aToHex } from '../u8a/toHex.js'

const peerIdPubKey = Uint8Array.from([
  2, 104, 233, 189, 80, 129, 44, 179, 234, 88, 101, 46, 60, 61, 48, 87, 12, 238, 242, 64, 28, 86, 221, 89, 184, 117,
  211, 7, 89, 143, 116, 64, 235
])

const prefixedPeerIdPubKey = Uint8Array.from([
  8, 2, 18, 33, 2, 104, 233, 189, 80, 129, 44, 179, 234, 88, 101, 46, 60, 61, 48, 87, 12, 238, 242, 64, 28, 86, 221, 89,
  184, 117, 211, 7, 89, 143, 116, 64, 235
])

const secp256k1PubKey = new keys.supportedKeys.secp256k1.Secp256k1PublicKey(peerIdPubKey)

const peerIdPubKeyHex = '0268e9bd50812cb3ea58652e3c3d30570ceef2401c56dd59b875d307598f7440eb'

const peerIdString = '16Uiu2HAm2VD6owCxPEZwP6Moe1jzapqziVeaTXf1h7jVzu5dW1mk'

describe(`converting pubkey to peerId`, function () {
  it('from hex string', function () {
    const deserializedWithPrefx = pubKeyToPeerId(`0x${peerIdPubKeyHex}`)
    const deserialized = pubKeyToPeerId(peerIdPubKeyHex)

    assert(deserialized.equals(deserializedWithPrefx))

    assert(deserialized.privateKey == null)
    assert(deserialized.publicKey != null)
    assert(deserialized.type === 'secp256k1')
    assert(deserialized.toString() === peerIdString)
  })

  it('from u8a', function () {
    const deserialized = pubKeyToPeerId(peerIdPubKey)

    assert(deserialized.privateKey == null)
    assert(deserialized.publicKey != null)
    assert(deserialized.type === 'secp256k1')
    assert(deserialized.toString() === peerIdString)
  })

  it('interoperability with libp2p methods', async function () {
    const pId = pubKeyToPeerId(peerIdPubKey)

    const deserializedFromKeys = await peerIdFromKeys(prefixedPeerIdPubKey)
    const deserializedFromString = peerIdFromString(peerIdString)
    const deserializedFromPubKey = await createFromPubKey(secp256k1PubKey)

    assert(pId.equals(deserializedFromPubKey))
    assert(pId.equals(deserializedFromKeys))
    assert(pId.equals(deserializedFromString))
  })

  it('check invalid arguments', function () {
    const incorrectPrivKey = new Uint8Array(1).fill(0)
    assert.throws(() => pubKeyToPeerId(incorrectPrivKey))

    const tooShortString = `0x12`
    assert.throws(() => pubKeyToPeerId(tooShortString))

    const invalidCharacter = `0y`
    assert.throws(() => pubKeyToPeerId(invalidCharacter))
  })
})
