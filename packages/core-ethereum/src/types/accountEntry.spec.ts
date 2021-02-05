import { expect } from 'chai'
import BN from 'bn.js'
import { u8aToHex } from '@hoprnet/hopr-utils'
import AccountEntry from './accountEntry'
import { BYTES27_LENGTH } from '../constants'

describe('test AccountEntry', function () {
  it('should be empty', function () {
    const accountEntry = new AccountEntry(undefined, {
      blockNumber: new BN(0),
      transactionIndex: new BN(0),
      logIndex: new BN(0),
      hashedSecret: new Uint8Array({
        length: BYTES27_LENGTH
      }).fill(0),
      counter: new BN(0)
    })

    expect(accountEntry.blockNumber.toString()).to.equal('0')
    expect(accountEntry.transactionIndex.toString()).to.equal('0')
    expect(accountEntry.logIndex.toString()).to.equal('0')
    expect(u8aToHex(accountEntry.hashedSecret)).to.equal('0x000000000000000000000000000000000000000000000000000000')
    expect(accountEntry.counter.toString()).to.equal('0')
  })

  it('should contain the right values', function () {
    const accountEntry = new AccountEntry(undefined, {
      blockNumber: new BN(1),
      transactionIndex: new BN(2),
      logIndex: new BN(3),
      hashedSecret: new Uint8Array(Buffer.from([1, 2, 4, 5]), undefined, BYTES27_LENGTH),
      counter: new BN(10)
    })

    expect(accountEntry.blockNumber.toString()).to.equal('1')
    expect(accountEntry.transactionIndex.toString()).to.equal('2')
    expect(accountEntry.logIndex.toString()).to.equal('3')
    expect(u8aToHex(accountEntry.hashedSecret)).to.equal('0x010204050000000000000000000000000000000000000000000000')
    expect(accountEntry.counter.toString()).to.equal('10')
  })
})
