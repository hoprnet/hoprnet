import { expect } from 'chai'
import { Multiaddr } from 'multiaddr'
import { Address, AccountEntry } from '.'
import BN from 'bn.js'

// TODO: move these & similar into constants file
const EMPTY_ADDRESS = new Address(new Uint8Array({ length: Address.SIZE }))

const PARTY_A_ADDRESS = Address.fromString('0x55CfF15a5159239002D57C591eF4ACA7f2ACAfE6')
const PARTY_A_MULTI_ADDR = new Multiaddr(
  '/ip4/34.65.237.196/tcp/9091/p2p/16Uiu2HAmThyWP5YWutPmYk9yUZ48ryWyZ7Cf6pMTQduvHUS9sGE7'
)

describe('AccountEntry', function () {
  it('should be empty', function () {
    const accountEntry = AccountEntry.deserialize(new Uint8Array({ length: AccountEntry.SIZE }))

    expect(accountEntry.address.toHex()).to.equal(EMPTY_ADDRESS.toHex())
    expect(accountEntry.multiAddr).to.equal(undefined)
  })

  it('should contain the right values', function () {
    const accountEntry = AccountEntry.deserialize(
      new AccountEntry(PARTY_A_ADDRESS, PARTY_A_MULTI_ADDR, new BN('1')).serialize()
    )

    expect(accountEntry.address.toHex()).to.equal(PARTY_A_ADDRESS.toHex())
    expect(accountEntry.multiAddr.toString()).to.equal(PARTY_A_MULTI_ADDR.toString())
  })
})
