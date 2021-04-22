import assert from 'assert'
import { Address, PublicKey } from './primitives'
import { utils } from 'ethers'

const privateKey = '0xe17fe86ce6e99f4806715b0c9412f8dad89334bf07f72d5834207a9d8f19d7f8'
const uncompressedPubKey =
  '0x041464586aeaea0eb5736884ca1bf42d165fc8e2243b1d917130fb9e321d7a93b8fb0699d4f177f9c84712f6d7c5f6b7f4f6916116047fa25c79ef806fc6c9523e'
const publicKey = '0x021464586aeaea0eb5736884ca1bf42d165fc8e2243b1d917130fb9e321d7a93b8'
const address = '0x115Bc5B501CdD8D1fA5098D3c9Be8dd5954CA371'

describe.only('test Address primitive', function () {
  it('should have a size of 20', function () {
    assert.strictEqual(Address.SIZE, 20)
  })

  it('should create Address from Uint8Array', function () {
    assert.strictEqual(new Address(utils.arrayify(address)).toHex(), address)
  })

  it('should create Address from string', function () {
    assert.strictEqual(Address.fromString(address).toHex(), address)
  })

  it('should serialize correctly', function () {
    assert.strictEqual(new Address(Address.fromString(address).serialize()).toHex(), address)
  })

  it('should be equal', function () {
    assert(Address.fromString(address).eq(Address.fromString(address)), 'addresses not equal')
  })
})

describe.only('test PublicKey primitive', function () {
  it('should have a size of 33', function () {
    assert.strictEqual(PublicKey.SIZE, 33)
  })

  it('should create PublicKey from Uint8Array', function () {
    assert.strictEqual(new PublicKey(utils.arrayify(publicKey)).toHex(), publicKey)
  })

  it('should create PublicKey from string', function () {
    assert.strictEqual(PublicKey.fromString(publicKey).toHex(), publicKey)
  })

  it('should create PublicKey from uncompressed public key', function () {
    assert.strictEqual(PublicKey.fromUncompressedPubKey(utils.arrayify(uncompressedPubKey)).toHex(), publicKey)
  })

  it('should create PublicKey from private key', function () {
    assert.strictEqual(PublicKey.fromPrivKey(utils.arrayify(privateKey)).toHex(), publicKey)
  })

  it('should create the correct Address', function () {
    assert.strictEqual(PublicKey.fromString(publicKey).toAddress().toHex(), address)
  })
})
