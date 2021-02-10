import type { Event } from './types'
import { expect } from 'chai'
import { u8aToHex } from '@hoprnet/hopr-utils'
import { pubKeyToAccountId } from '../../utils'
import * as logs from './logs'
import * as fixtures from './logs.fixtures.spec'

describe('test topic utils', function () {
  it('should convert SECRET_HASHED_SET log to SECRET_HASHED_SET event', async function () {
    const actual = logs.toSecretHashSetEvent(fixtures.SECRET_HASHED_SET_LOG)
    const expected = fixtures.SECRET_HASHED_SET_EVENT

    expectBaseEventsToBeEqual(actual, expected)
    expect(actual.data.account.toHex()).to.equal(expected.data.account.toHex(), 'account')
    expect(u8aToHex(actual.data.secretHash)).to.equal(u8aToHex(expected.data.secretHash), 'secretHash')
    expect(actual.data.counter.toString()).to.equal(expected.data.counter.toString(), 'counter')
  })

  it('should convert FUNDED log to FUNDED event', async function () {
    const actual = logs.toFundedChannelEvent(fixtures.FUNDED_LOG)
    const expected = fixtures.FUNDED_EVENT

    expectBaseEventsToBeEqual(actual, expected)
    expect(actual.data.funder.toHex()).to.equal(expected.data.funder.toHex(), 'funder')
    expect(actual.data.recipient.toHex()).to.equal(expected.data.recipient.toHex(), 'recipient')
    expect(actual.data.counterparty.toHex()).to.equal(expected.data.counterparty.toHex(), 'counterparty')
    expect(actual.data.recipientAmount.toString()).to.equal(expected.data.recipientAmount.toString(), 'recipientAmount')
    expect(actual.data.counterpartyAmount.toString()).to.equal(
      expected.data.counterpartyAmount.toString(),
      'counterpartyAmount'
    )

    const actualRecipientAccountId = await pubKeyToAccountId(actual.data.recipient)
    const actualCounterpartyAccountId = await pubKeyToAccountId(actual.data.counterparty)

    expect(actualRecipientAccountId.toHex()).to.equal('0x74cdeAb5AE6efFDC70699731EC3ab55f35fb41eA')
    expect(actualCounterpartyAccountId.toHex()).to.equal('0x044aEf65B5A3f18ab9aD3A09012747C0397b9089')
  })

  it('should convert OPENED log to OPENED event', async function () {
    const actual = logs.toOpenedChannelEvent(fixtures.OPENED_LOG)
    const expected = fixtures.OPENED_EVENT

    expectBaseEventsToBeEqual(actual, expected)
    expect(actual.data.opener.toHex()).to.equal(expected.data.opener.toHex(), 'opener')
    expect(actual.data.counterparty.toHex()).to.equal(expected.data.counterparty.toHex(), 'counterparty')

    const actualOpenerAccountId = await pubKeyToAccountId(actual.data.opener)
    const actualCounterpartyAccountId = await pubKeyToAccountId(actual.data.counterparty)

    expect(actualOpenerAccountId.toHex()).to.equal('0xf73A34405D1349476B5500Ea0381A1fcc87e8AEb')
    expect(actualCounterpartyAccountId.toHex()).to.equal('0x20516d47c46Bcd67D19898FDA6aE8A68B3022429')
  })

  it('should convert CLOSING log to InitiatedChannelClosure event', async function () {
    const actual = logs.toInitiatedChannelClosureEvent(fixtures.CLOSING_LOG)
    const expected = fixtures.CLOSING_EVENT

    expectBaseEventsToBeEqual(actual, expected)
    expect(actual.data.initiator.toHex()).to.equal(expected.data.initiator.toHex(), 'initiator')
    expect(actual.data.counterparty.toHex()).to.equal(expected.data.counterparty.toHex(), 'counterparty')

    const actualInitiatorAccountId = await pubKeyToAccountId(actual.data.initiator)
    const actualCounterpartyAccountId = await pubKeyToAccountId(actual.data.counterparty)

    expect(actualInitiatorAccountId.toHex()).to.equal('0x9c62B096e229bD6B929212728beD7996498a66e8')
    expect(actualCounterpartyAccountId.toHex()).to.equal('0x9AC4c344608CD61Af8dDC9ecab9077E3Fab87bEa')
  })

  // @TODO: add CLOSED test
})

const expectBaseEventsToBeEqual = (actual: Event<any>, expected: Event<any>) => {
  expect(actual.transactionHash).to.equal(expected.transactionHash, 'transactionHash')
  expect(actual.blockNumber.toString()).to.equal(expected.blockNumber.toString(), 'blockNumber')
  expect(actual.transactionIndex.toString()).to.equal(expected.transactionIndex.toString(), 'transactionIndex')
  expect(actual.logIndex.toString()).to.equal(expected.logIndex.toString(), 'logIndex')
}
