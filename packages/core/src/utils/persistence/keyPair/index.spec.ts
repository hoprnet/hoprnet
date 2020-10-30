import { serializeKeyPair } from './serialize'
import { deserializeKeyPair } from './deserialize'

import { decode, encode } from 'rlp'
import PeerId from 'peer-id'

import { randomBytes } from 'crypto'
import assert from 'assert'

import { randomInteger, u8aEquals } from '@hoprnet/hopr-utils'

describe('keypair/index.spec.ts test serialisation and deserialisation of encrypted keypair', function () {
  it('should serialize and deserialize a keypair', async function () {
    this.timeout(5000)
    const password = randomBytes(randomInteger(1, 33))

    const peerId = await PeerId.create({ keyType: 'secp256k1' })

    assert(
      !u8aEquals(await serializeKeyPair(peerId, password), await serializeKeyPair(peerId, password)),
      'Serialization of same peerId should lead to different ciphertexts'
    )

    const serializedKeyPair = await serializeKeyPair(peerId, password)
    assert(
      u8aEquals((await deserializeKeyPair(serializedKeyPair, password)).marshal(), peerId.marshal()),
      'PeerId must be recoverable from serialized peerId'
    )

    const [salt, mac, encodedCiphertext] = (decode(serializedKeyPair) as unknown) as [Buffer, Buffer, Buffer]
    try {
      const manipulatedSalt = Buffer.from(salt)
      manipulatedSalt.set(randomBytes(1), randomInteger(0, manipulatedSalt.length))
      await deserializeKeyPair(encode([manipulatedSalt, mac, encodedCiphertext]), password)
      assert.fail('Shoud fail with manipulated salt')
    } catch {}

    try {
      const manipulatedMac = Buffer.from(salt)
      manipulatedMac.set(randomBytes(1), randomInteger(0, manipulatedMac.length))
      await deserializeKeyPair(encode([salt, manipulatedMac, encodedCiphertext]), password)
      assert.fail('Shoud fail with manipulated MAC')
    } catch {}

    try {
      const manipulatedCiphertext = Buffer.from(salt)
      manipulatedCiphertext.set(randomBytes(1), randomInteger(0, manipulatedCiphertext.length))
      await deserializeKeyPair(encode([salt, mac, manipulatedCiphertext]), password)
      assert.fail('Shoud fail with manipulated ciphertext')
    } catch {}
  })
})
