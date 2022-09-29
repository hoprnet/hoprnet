import assert from 'assert'
import { peerIdFromString } from '@libp2p/peer-id'
import { stringToU8a, u8aEquals, u8aToHex } from '../u8a/index.js'
import { PublicKey } from './publicKey.js'
import { Address } from './primitives.js'

const privateKey = '0xe17fe86ce6e99f4806715b0c9412f8dad89334bf07f72d5834207a9d8f19d7f8'
const uncompressedPubKey =
  '0x041464586aeaea0eb5736884ca1bf42d165fc8e2243b1d917130fb9e321d7a93b8fb0699d4f177f9c84712f6d7c5f6b7f4f6916116047fa25c79ef806fc6c9523e'
const uncompressedInvalidPubKey =
  '0x051464586aeaea0eb5736884ca1bf42d165fc8e2243b1d917130fb9e321d7a93b8fb0699d4f177f9c84712f6d7c5f6b7f4f6916116047fa25c79ef806fc6c9523e'
const uncompressedPubKeyWithoutPrefix =
  '0x1464586aeaea0eb5736884ca1bf42d165fc8e2243b1d917130fb9e321d7a93b8fb0699d4f177f9c84712f6d7c5f6b7f4f6916116047fa25c79ef806fc6c9523e'

const compressedPubKey = '0x021464586aeaea0eb5736884ca1bf42d165fc8e2243b1d917130fb9e321d7a93b8'
const compressedInvalidPubKey = '0x041464586aeaea0eb5736884ca1bf42d165fc8e2243b1d917130fb9e321d7a93b8'

const b58String = '16Uiu2HAkvoGszJh3KCuxxZPsNjcCN5X1PHbnhAyGzvSyub88b679'
const address = '0x115Bc5B501CdD8D1fA5098D3c9Be8dd5954CA371'

describe('test PublicKey primitive', function () {
  it('from private key', function () {
    const pKey = PublicKey.fromPrivKey(stringToU8a(privateKey))

    assert(u8aToHex(pKey.serializeUncompressed()) === uncompressedPubKey)
    assert(u8aToHex(pKey.serializeCompressed()) === compressedPubKey)
    assert(pKey.toString() === b58String)
  })

  it('from Uint8Array', function () {
    const pKeyFromUncompressed = PublicKey.deserialize(stringToU8a(uncompressedPubKey))

    assert(u8aToHex(pKeyFromUncompressed.serializeUncompressed()) === uncompressedPubKey)
    assert(u8aToHex(pKeyFromUncompressed.serializeCompressed()) === compressedPubKey)
    assert(pKeyFromUncompressed.toString() === b58String)

    const pKeyFromCompressed = PublicKey.deserialize(stringToU8a(compressedPubKey))

    assert(u8aToHex(pKeyFromCompressed.serializeUncompressed()) === uncompressedPubKey)
    assert(u8aToHex(pKeyFromCompressed.serializeCompressed()) === compressedPubKey)
    assert(pKeyFromCompressed.toString() === b58String)

    const pKeyFromUnPrefixedUncompressed = PublicKey.deserialize(stringToU8a(uncompressedPubKeyWithoutPrefix))

    assert(u8aToHex(pKeyFromUnPrefixedUncompressed.serializeUncompressed()) === uncompressedPubKey)
    assert(u8aToHex(pKeyFromUnPrefixedUncompressed.serializeCompressed()) === compressedPubKey)
    assert(pKeyFromUnPrefixedUncompressed.toString() === b58String)

    assert.throws(() => PublicKey.deserialize(stringToU8a(uncompressedInvalidPubKey)))
    assert.throws(() => PublicKey.deserialize(stringToU8a(compressedInvalidPubKey)))
  })

  it('from string', function () {
    const pKeyFromUncompressed = PublicKey.fromString(uncompressedPubKey)

    assert(u8aToHex(pKeyFromUncompressed.serializeUncompressed()) === uncompressedPubKey)
    assert(u8aToHex(pKeyFromUncompressed.serializeCompressed()) === compressedPubKey)
    assert(pKeyFromUncompressed.toString() === b58String)

    const pKeyFromCompressed = PublicKey.fromString(compressedPubKey)

    assert(u8aToHex(pKeyFromCompressed.serializeUncompressed()) === uncompressedPubKey)
    assert(u8aToHex(pKeyFromCompressed.serializeCompressed()) === compressedPubKey)
    assert(pKeyFromCompressed.toString() === b58String)

    const pKeyFromUnPrefixedUncompressed = PublicKey.fromString(uncompressedPubKeyWithoutPrefix)

    assert(u8aToHex(pKeyFromUnPrefixedUncompressed.serializeUncompressed()) === uncompressedPubKey)
    assert(u8aToHex(pKeyFromUnPrefixedUncompressed.serializeCompressed()) === compressedPubKey)
    assert(pKeyFromUnPrefixedUncompressed.toString() === b58String)

    assert.throws(() => PublicKey.fromString(uncompressedInvalidPubKey))
    assert.throws(() => PublicKey.fromString(compressedInvalidPubKey))
  })

  it('from PeerId', function () {
    const pId = peerIdFromString(b58String)

    const pKey = PublicKey.fromPeerId(pId)

    assert(u8aToHex(pKey.serializeUncompressed()) === uncompressedPubKey)
    assert(u8aToHex(pKey.serializeCompressed()) === compressedPubKey)
    assert(pKey.toString() === b58String)

    const pKeyfromB58String = PublicKey.fromPeerIdString(b58String)

    assert(u8aToHex(pKeyfromB58String.serializeUncompressed()) === uncompressedPubKey)
    assert(u8aToHex(pKeyfromB58String.serializeCompressed()) === compressedPubKey)
    assert(pKeyfromB58String.toString() === b58String)
  })

  it('equals', function () {
    const pKeyFromUncompressed = PublicKey.fromString(uncompressedPubKey)
    const pKeyFromCompressed = PublicKey.fromString(compressedPubKey)

    assert(pKeyFromUncompressed.eq(pKeyFromUncompressed))
    assert(pKeyFromCompressed.eq(pKeyFromCompressed))
    assert(pKeyFromCompressed.eq(pKeyFromUncompressed))
    assert(pKeyFromUncompressed.eq(pKeyFromCompressed))
  })

  it('toAddress', function () {
    const pKeyFromUncompressed = PublicKey.fromString(uncompressedPubKey)
    const pKeyFromCompressed = PublicKey.fromString(compressedPubKey)

    assert(pKeyFromCompressed.toAddress().toHex().toLowerCase() === address.toLowerCase())
    assert(pKeyFromUncompressed.toAddress().toHex().toLowerCase() === address.toLowerCase())
  })

  it('should recover public key', function () {
    // As taken from an Ethereum transaction
    const address = Address.fromString('0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266')

    const r = '0xbcae4d37e3a1cd869984d1d68f9242291773cd33d26f1e754ecc1a9bfaee7d17'
    const s = '0x0b755ab5f6375595fc7fc245c45f6598cc873719183733f4c464d63eefd8579b'
    const v = 1

    const hash = '0xfac7acad27047640b069e8157b61623e3cb6bb86e6adf97151f93817c291f3cf'

    assert(PublicKey.fromSignatureString(hash, r, s, v).toAddress().eq(address))
  })

  it('to uncompressed hex', function () {
    const pKeyFromUncompressed = PublicKey.fromString(uncompressedPubKey)
    const pKeyFromCompressed = PublicKey.fromString(compressedPubKey)

    assert(pKeyFromCompressed.toUncompressedPubKeyHex() === uncompressedPubKeyWithoutPrefix)
    assert(pKeyFromUncompressed.toUncompressedPubKeyHex() === uncompressedPubKeyWithoutPrefix)
  })

  it('to compressed hex', function () {
    const pKeyFromUncompressed = PublicKey.fromString(uncompressedPubKey)
    const pKeyFromCompressed = PublicKey.fromString(compressedPubKey)

    assert(pKeyFromCompressed.toCompressedPubKeyHex() === compressedPubKey)
    assert(pKeyFromUncompressed.toCompressedPubKeyHex() === compressedPubKey)
  })

  it('to string', function () {
    const pKeyFromUncompressed = PublicKey.fromString(uncompressedPubKey)
    const pKeyFromCompressed = PublicKey.fromString(compressedPubKey)

    assert(pKeyFromCompressed.toString() === b58String)
    assert(pKeyFromUncompressed.toString() === b58String)
  })

  it('serialize array', function () {
    const uncompressed = PublicKey.fromString(uncompressedPubKey)
    const compressed = PublicKey.fromString(compressedPubKey)

    const result = PublicKey.serializeArray([compressed, uncompressed])

    assert(result.length == PublicKey.SIZE_COMPRESSED + PublicKey.SIZE_UNCOMPRESSED)

    assert(
      u8aEquals(Uint8Array.from([...compressed.serializeCompressed(), ...uncompressed.serializeUncompressed()]), result)
    )

    assert(PublicKey.serializeArray([]).length == 0)
  })

  it('deserialize array', function () {
    const uncompressed = PublicKey.fromString(uncompressedPubKey)
    const compressed = PublicKey.fromString(compressedPubKey)

    const result = PublicKey.deserializeArray(
      Uint8Array.from([...compressed.serializeCompressed(), ...uncompressed.serializeUncompressed()])
    )

    assert(result.length == 2, `Must contain two PublicKeys`)

    assert(result[0].isCompressed, `First one must be compressed`)
    assert(result[0].eq(compressed))
    assert(!result[1].isCompressed, `Second one must be uncompressed`)
    assert(result[1].eq(uncompressed))

    assert(PublicKey.deserializeArray(Uint8Array.from([])).length == 0)

    assert.throws(
      () =>
        PublicKey.deserializeArray(
          Uint8Array.from([2, ...compressed.serializeCompressed(), ...uncompressed.serializeUncompressed()])
        ),
      Error('Invalid prefix 0xb8 at 33')
    )

    assert.throws(
      () =>
        PublicKey.deserializeArray(
          Uint8Array.from([...compressed.serializeCompressed(), 2, ...uncompressed.serializeUncompressed()])
        ),
      Error('Invalid prefix 0xb8 at 66')
    )

    assert.throws(
      () => PublicKey.deserializeArray(Uint8Array.from([2])),
      Error('Invalid array length. U8a has 32 to few elements')
    )

    assert.throws(
      () => PublicKey.deserializeArray(Uint8Array.from([3])),
      Error('Invalid array length. U8a has 32 to few elements')
    )

    assert.throws(
      () => PublicKey.deserializeArray(Uint8Array.from([4])),
      Error('Invalid array length. U8a has 64 to few elements')
    )
  })
})
