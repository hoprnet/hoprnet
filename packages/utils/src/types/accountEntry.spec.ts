import assert from 'assert'
import { Multiaddr } from 'multiaddr'
import { Address, AccountEntry } from '.'
import { encode } from 'bs58'
import BN from 'bn.js'
import { publicKeyVerify } from 'secp256k1'

// TODO: move these & similar into constants file
const EMPTY_ADDRESS = new Address(new Uint8Array({ length: Address.SIZE }))

const PARTY_A_ADDRESS = Address.fromString('0x55CfF15a5159239002D57C591eF4ACA7f2ACAfE6')
const PARTY_A_MULTI_ADDR = new Multiaddr(
  '/ip4/34.65.237.196/tcp/9091/p2p/16Uiu2HAmThyWP5YWutPmYk9yUZ48ryWyZ7Cf6pMTQduvHUS9sGE7'
)

describe('AccountEntry', function () {
  it('should be empty', function () {
    const accountEntry = AccountEntry.deserialize(new Uint8Array({ length: AccountEntry.SIZE }))

    assert(accountEntry.address.eq(EMPTY_ADDRESS))
    assert(accountEntry.multiAddr === undefined)
  })

  it('should contain the right values', function () {
    const accountEntry = AccountEntry.deserialize(
      new AccountEntry(PARTY_A_ADDRESS, PARTY_A_MULTI_ADDR, new BN('1')).serialize()
    )

    assert(accountEntry.address.eq(PARTY_A_ADDRESS))
    assert(accountEntry.multiAddr.equals(PARTY_A_MULTI_ADDR))
  })

  it('should fail on invalid public keys', function () {
    const INVALID_PUBLIC_KEY = Uint8Array.from([0x03, ...new Uint8Array(32).fill(0xff)])

    assert(!publicKeyVerify(INVALID_PUBLIC_KEY), 'Public key must not be invalid')

    const MULTIHASH_PREFIX = Uint8Array.from([0x00, 0x25, 0x08, 0x02, 0x12, 0x21])
    const INVALID_ENCODED_KEY = encode(Uint8Array.from([...MULTIHASH_PREFIX, ...INVALID_PUBLIC_KEY]))

    assert.throws(
      () =>
        new AccountEntry(PARTY_A_ADDRESS, new Multiaddr(`/ip4/1.2.3.4/tcp/0/p2p/${INVALID_ENCODED_KEY}`), new BN('1')),
      Error('Multiaddr does not contain a valid public key.')
    )
  })
})
