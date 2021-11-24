import assert from 'assert'
import { utils } from 'ethers'
import BN from 'bn.js'
import { Address, PublicKey, Hash, Balance, NativeBalance, Signature } from './primitives'

const privateKey = '0xe17fe86ce6e99f4806715b0c9412f8dad89334bf07f72d5834207a9d8f19d7f8'
const uncompressedPubKey =
  '0x041464586aeaea0eb5736884ca1bf42d165fc8e2243b1d917130fb9e321d7a93b8fb0699d4f177f9c84712f6d7c5f6b7f4f6916116047fa25c79ef806fc6c9523e'
const publicKey = '0x021464586aeaea0eb5736884ca1bf42d165fc8e2243b1d917130fb9e321d7a93b8'
const address = '0x115Bc5B501CdD8D1fA5098D3c9Be8dd5954CA371'

describe('test Address primitive', function () {
  const empty = new Address(new Uint8Array({ length: Address.SIZE }))
  const larger = new Address(new Uint8Array({ length: Address.SIZE }).fill(1))

  it('should have a size of 20', function () {
    assert.strictEqual(Address.SIZE, 20)
  })

  it('should create Address from Uint8Array', function () {
    assert.strictEqual(new Address(utils.arrayify(address)).toHex(), address)
  })

  it('should create Address from string', function () {
    assert.strictEqual(Address.fromString(address).toHex(), address)
  })

  it('should correctly serialize', function () {
    assert.strictEqual(new Address(Address.fromString(address).serialize()).toHex(), address)
  })

  it('should be equal', function () {
    assert(Address.fromString(address).eq(Address.fromString(address)), 'addresses not equal')
  })

  it('should compare correctly', function () {
    assert.strictEqual(empty.compare(empty), 0)
    assert.strictEqual(empty.compare(larger), -1)
    assert.strictEqual(larger.compare(empty), 1)
  })

  it('should be less than correctly', function () {
    assert(empty.lt(larger))
    assert(!larger.lt(empty))
  })

  it('should sort addresses correctly', function () {
    const [partyA, partyB] = empty.sortPair(larger)

    assert.strictEqual(partyA.toHex(), empty.toHex())
    assert.strictEqual(partyB.toHex(), larger.toHex())
  })
})

describe('test PublicKey primitive', function () {
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

  it('should correctly serialize', function () {
    assert.strictEqual(new PublicKey(PublicKey.fromString(publicKey).serialize()).toHex(), publicKey)
  })

  it('should be equal', function () {
    assert(PublicKey.fromString(publicKey).eq(PublicKey.fromString(publicKey)), 'public keys not equal')
  })

  it('should recover public key', function () {
    // As taken from an Ethereum transaction
    const address = Address.fromString('0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266')

    const r = '0xbcae4d37e3a1cd869984d1d68f9242291773cd33d26f1e754ecc1a9bfaee7d17'
    const s = '0x0b755ab5f6375595fc7fc245c45f6598cc873719183733f4c464d63eefd8579b'
    const v = 1

    const hash = '0xfac7acad27047640b069e8157b61623e3cb6bb86e6adf97151f93817c291f3cf'

    assert(PublicKey.fromSignature(hash, r, s, v).toAddress().eq(address))
  })
})

describe('test Hash primitive', function () {
  const hashPreImage = 'hello world'
  const hash = '0x47173285a8d7341e5e972fc677286384f802f8ef42a5ec5f03bbfa254cb01fad'

  it('should have a size of 32', function () {
    assert.strictEqual(Hash.SIZE, 32)
  })

  it('should create Hash from Uint8Array', function () {
    assert.strictEqual(new Hash(utils.arrayify(hash)).toHex(), hash)
  })

  it('should correctly serialize', function () {
    assert.strictEqual(new Hash(utils.arrayify(hash)).toHex(), hash)
  })

  it('should correctly hash value', function () {
    assert.strictEqual(Hash.create(utils.toUtf8Bytes(hashPreImage)).toHex(), hash)
  })

  it('should clone hash', function () {
    const _hash = new Hash(utils.arrayify(hash))
    const hashCloned = _hash.clone()

    assert.strictEqual(_hash.toHex(), hashCloned.toHex())
  })

  it('should hash again', function () {
    assert.strictEqual(
      Hash.create(utils.toUtf8Bytes(hashPreImage)).hash().toHex(),
      '0x04cd40a3ea7972c6f30142d02fd5ddcac438fe6c59e634cecb827fbee9d385fc'
    )
  })
})

describe('test Balance primitive', function () {
  const balance = new BN(1)

  it('should have a size of 32', function () {
    assert.strictEqual(Balance.SIZE, 32)
  })

  it('should create Balance from BN', function () {
    assert.strictEqual(new Balance(balance).toBN().toString(), balance.toString())
  })

  it('should create BN', function () {
    assert.strictEqual(new Balance(balance).toBN().toString(), balance.toString())
  })

  it('should correctly serialize & deserialize', function () {
    assert.strictEqual(Balance.deserialize(new Balance(balance).serialize()).toBN().toString(), balance.toString())
  })

  it('should format Balance', function () {
    assert.strictEqual(new Balance(balance).toFormattedString(), '0.000000000000000001 txHOPR')
  })
})

describe('test NativeBalance primitive', function () {
  const balance = new BN(1)

  it('should have a size of 32', function () {
    assert.strictEqual(NativeBalance.SIZE, 32)
  })

  it('should create NativeBalance from BN', function () {
    assert.strictEqual(new NativeBalance(balance).toBN().toString(), balance.toString())
  })

  it('should create BN', function () {
    assert.strictEqual(new NativeBalance(balance).toBN().toString(), balance.toString())
  })

  it('should correctly serialize', function () {
    assert.strictEqual(
      NativeBalance.deserialize(new NativeBalance(balance).serialize()).toBN().toString(),
      balance.toString()
    )
  })

  it('should format NativeBalance', function () {
    assert.strictEqual(new NativeBalance(balance).toFormattedString(), '0.000000000000000001 xDAI')
  })
})

describe('test Signature primitive', function () {
  /* test with signature with recovery = 0 */
  const message1 = utils.keccak256(utils.toUtf8Bytes('hello'))
  const signature1 =
    '0x583ced30525d3b0663c223c834b80c4662043f7a2aa4f001354a2858f517571b08d65c7340e9067a0dcc18b3529ac2aab3dc3067621607b1268b645923cf003f'
  const recovery1 = 0

  /* test with signature with recovery = 1 */
  const signature2 =
    '0x92f3745d95d33c79041e9c8d8c13ecacc468d4839745613aa531622b9f26827b7e2fb5d3cf4b53d52ff48a9434c5ff9009ea049f86f00f9aa7c0ae8a6b6cc7ec'
  const message2 = utils.keccak256(utils.toUtf8Bytes('blahblah_'))
  const recovery2 = 1

  it('should have a size of 64', function () {
    assert.strictEqual(Signature.SIZE, 64)
  })

  it('should create Signature from Uint8Array and number', function () {
    const s = new Signature(utils.arrayify(signature1), recovery1)

    assert.strictEqual(utils.hexlify(s.signature), signature1)
    assert.strictEqual(s.recovery, recovery1)
  })

  it('malformed signatures should fail', function () {
    assert.throws(() => new Signature(new Uint8Array(32), 0))
    assert.throws(() => new Signature(new Uint8Array(64), 20))
  })

  it('should create Signature from message', function () {
    const s = Signature.create(utils.arrayify(message1), utils.arrayify(privateKey))

    assert.strictEqual(utils.hexlify(s.signature), signature1)
    assert.strictEqual(s.recovery, recovery1)
  })

  it('should verify Signature', function () {
    const s1 = new Signature(utils.arrayify(signature1), recovery1)
    assert(s1.verify(utils.arrayify(message1), PublicKey.fromString(publicKey)))

    const s2 = new Signature(utils.arrayify(signature2), recovery2)
    assert(s2.verify(utils.arrayify(message2), PublicKey.fromString(publicKey)))
  })

  it('should correctly serialize & deserialize', function () {
    const serialized1 = new Signature(utils.arrayify(signature1), recovery1).serialize()
    assert.strictEqual(serialized1.length, Signature.SIZE)

    const s1 = Signature.deserialize(serialized1)
    assert.strictEqual(s1.recovery, 0)
    assert(s1.verify(utils.arrayify(message1), PublicKey.fromString(publicKey)))

    const serialized2 = new Signature(utils.arrayify(signature2), recovery2).serialize()
    assert.strictEqual(serialized2.length, Signature.SIZE)

    const s2 = Signature.deserialize(serialized2)
    assert.strictEqual(s2.recovery, 1)
    assert(s2.verify(utils.arrayify(message2), PublicKey.fromString(publicKey)))
  })
})
