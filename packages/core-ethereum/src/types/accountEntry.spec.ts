import { expect } from 'chai'
import BN from 'bn.js'
import { Address } from '.'
import Public from './public'
import AccountEntry from './accountEntry'
import Hash from './hash'
import { stringToU8a } from '@hoprnet/hopr-utils'

// TODO: move these & similar into constants file
const EMPTY_ADDRESS = new Address(new Uint8Array({ length: Address.SIZE }))
const EMPTY_PUBKEY = new Public(new Uint8Array({ length: Public.SIZE }))
const EMPTY_SECRET = new Hash(new Uint8Array({ length: Hash.SIZE }))

const partyAPubKey = Public.fromString('0x03362b7b26bddb151a03056422d37119eab3a716562b6c3efdc62dec1540c9b091')
const partyA = Address.fromString('0x55CfF15a5159239002D57C591eF4ACA7f2ACAfE6')
const secret = new Hash(stringToU8a('0xb8b37f62ec82443e5b5557c5a187fe3686790620cc04c06187c48f8636caac89')) // secret

describe('AccountEntry', function () {
  it('should be empty', function () {
    const accountEntry = AccountEntry.deserialize(new Uint8Array({ length: AccountEntry.SIZE }))

    expect(accountEntry.address.toHex()).to.equal(EMPTY_ADDRESS.toHex())
    expect(accountEntry.publicKey.toHex()).to.equal(EMPTY_PUBKEY.toHex())
    expect(accountEntry.secret.toHex()).to.equal(EMPTY_SECRET.toHex())
    expect(accountEntry.counter.toString()).to.equal('0')
  })

  it('should contain the right values when only address passed', function () {
    const accountEntry = AccountEntry.deserialize(new AccountEntry(partyA).serialize())

    expect(accountEntry.address.toHex()).to.equal(partyA.toHex())
    expect(accountEntry.publicKey.toHex()).to.equal(EMPTY_PUBKEY.toHex())
    expect(accountEntry.secret.toHex()).to.equal(EMPTY_SECRET.toHex())
    expect(accountEntry.counter.toString()).to.equal('0')
  })

  it('should contain the right values', function () {
    const accountEntry = AccountEntry.deserialize(new AccountEntry(partyA, partyAPubKey, secret, new BN(1)).serialize())

    expect(accountEntry.address.toHex()).to.equal(partyA.toHex())
    expect(accountEntry.publicKey.toHex()).to.equal(partyAPubKey.toHex())
    expect(accountEntry.secret.toHex()).to.equal(secret.toHex())
    expect(accountEntry.counter.toString()).to.equal('1')
  })
})
