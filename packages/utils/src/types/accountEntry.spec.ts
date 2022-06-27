import assert from 'assert'
import { Multiaddr } from 'multiaddr'
import { AccountEntry } from './accountEntry.js'
import BN from 'bn.js'
import { PublicKey } from './publicKey.js'

// TODO: move these & similar into constants file
const PARTY_A = PublicKey.fromPrivKeyString('0xc14b8faa0a9b8a5fa4453664996f23a7e7de606d42297d723fc4a794f375e260')
const PARTY_A_PEERID = PARTY_A.toPeerId()
const PARTY_A_ADDRESS = PARTY_A.toAddress()
const PARTY_A_MULTI_ADDR = new Multiaddr('/p2p/16Uiu2HAm3rUQdpCz53tK1MVUUq9NdMAU6mFgtcXrf71Ltw6AStzk')
const PARTY_A_MULTI_ADDR_WITH_ROUTING = new Multiaddr(
  '/ip4/34.65.237.196/tcp/9091/p2p/16Uiu2HAm3rUQdpCz53tK1MVUUq9NdMAU6mFgtcXrf71Ltw6AStzk'
)

describe('AccountEntry', function () {
  it('create, serialize, deserialize - without routable address', function () {
    const accountEntry = new AccountEntry(PARTY_A, PARTY_A_MULTI_ADDR, new BN(1))

    assert(!accountEntry.containsRouting)
    assert(accountEntry.hasAnnounced)

    assert(accountEntry.getPeerId().equals(PARTY_A_PEERID))
    assert(accountEntry.getAddress().eq(PARTY_A_ADDRESS))

    const serialized = accountEntry.serialize()

    assert(serialized.length == AccountEntry.SIZE)

    const deserialized = AccountEntry.deserialize(serialized)

    assert(accountEntry.publicKey.eq(deserialized.publicKey))
    assert(accountEntry.multiAddr.equals(deserialized.multiAddr))
    assert(accountEntry.updatedBlock.eq(deserialized.updatedBlock))
  })

  it('create, serialize, deserialize - with routable address', function () {
    const accountEntry = new AccountEntry(PARTY_A, PARTY_A_MULTI_ADDR_WITH_ROUTING, new BN(1))

    assert(accountEntry.containsRouting)
    assert(accountEntry.hasAnnounced)

    assert(accountEntry.getPeerId().equals(PARTY_A_PEERID))
    assert(accountEntry.getAddress().eq(PARTY_A_ADDRESS))

    const serialized = accountEntry.serialize()

    assert(serialized.length == AccountEntry.SIZE)

    const deserialized = AccountEntry.deserialize(serialized)

    assert(accountEntry.publicKey.eq(deserialized.publicKey))
    assert(accountEntry.multiAddr.equals(deserialized.multiAddr))
    assert(accountEntry.updatedBlock.eq(deserialized.updatedBlock))
  })

  it('create, serialize, deserialize - without an address', function () {
    const accountEntry = new AccountEntry(PARTY_A, undefined, new BN(1))

    assert(!accountEntry.containsRouting)
    assert(!accountEntry.hasAnnounced)

    assert(accountEntry.getPeerId().equals(PARTY_A_PEERID))
    assert(accountEntry.getAddress().eq(PARTY_A_ADDRESS))

    const serialized = accountEntry.serialize()

    assert(serialized.length == AccountEntry.SIZE)

    const deserialized = AccountEntry.deserialize(serialized)

    assert(accountEntry.publicKey.eq(deserialized.publicKey))
    assert(accountEntry.updatedBlock.eq(deserialized.updatedBlock))
  })
})
