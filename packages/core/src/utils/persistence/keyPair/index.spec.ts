import { serializeKeyPair } from './serialize'
import { deserializeKeyPair } from './deserialize'

import PeerId from 'peer-id'

import { randomBytes } from 'crypto'
import assert from 'assert'

import { randomInteger, u8aEquals } from '@hoprnet/hopr-utils'

describe('keypair/index.spec.ts test serialisation and deserialisation of encrypted keypair', function () {
  it('should serialize and deserialize a keypair', async function () {
    this.timeout(5000)
    const password = randomBytes(randomInteger(1, 33))

    const peerId = await PeerId.create({ keyType: 'secp256k1' })

    const firstEncoding = await serializeKeyPair(peerId, password)
    const secondEncoding = await serializeKeyPair(peerId, password)

    assert(
      !u8aEquals(firstEncoding, secondEncoding),
      'Serialization of same peerId should lead to different ciphertexts'
    )

    const serializedKeyPair = await serializeKeyPair(peerId, password)
    assert(
      u8aEquals((await deserializeKeyPair(serializedKeyPair, password)).marshal(), peerId.marshal()),
      'PeerId must be recoverable from serialized peerId'
    )

    const [salt, mac, iv, ciphertext] = [
      serializedKeyPair.subarray(0, 32),
      serializedKeyPair.subarray(32, 64),
      serializedKeyPair.subarray(64, 80),
      serializedKeyPair.subarray(80, 112)
    ]

    try {
      const manipulatedSalt = Uint8Array.from(salt)
      manipulatedSalt.set(randomBytes(1), randomInteger(0, manipulatedSalt.length - 1))
      await deserializeKeyPair(Uint8Array.from([...manipulatedSalt, ...mac, ...iv, ...ciphertext]), password)
      assert.fail('Shoud fail with manipulated salt')
    } catch {}

    try {
      const manipulatedMac = Uint8Array.from(mac)
      manipulatedMac.set(randomBytes(1), randomInteger(0, manipulatedMac.length - 1))
      await deserializeKeyPair(Uint8Array.from([...salt, ...manipulatedMac, ...iv, ...ciphertext]), password)
      assert.fail('Shoud fail with manipulated MAC')
    } catch {}

    try {
      const manipulatedCiphertext = Uint8Array.from([...iv, ...ciphertext])
      manipulatedCiphertext.set(randomBytes(1), randomInteger(0, manipulatedCiphertext.length - 1))
      await deserializeKeyPair(Uint8Array.from([...salt, ...mac, ...manipulatedCiphertext]), password)
      assert.fail('Shoud fail with manipulated ciphertext')
    } catch {}
  })
})
