import type { Event } from './types'
import { expect } from 'chai'
import { pubKeyToAddress } from '../../utils'
import * as logs from './logs'
import * as fixtures from './logs.fixtures.spec'

// @TODO: add more tests
describe('test topic utils', function () {
  it('should convert FUNDED log to FUNDED event', async function () {
    const actual = logs.toFundedChannelEvent(fixtures.FUNDED_LOG)
    const expected = fixtures.FUNDED_EVENT

    expectBaseEventsToBeEqual(actual, expected)
    expect(actual.data.recipient.toHex()).to.equal(expected.data.recipient.toHex(), 'recipient')
    expect(actual.data.counterparty.toHex()).to.equal(expected.data.counterparty.toHex(), 'counterparty')
    expect(actual.data.recipientAmount.toString()).to.equal(expected.data.recipientAmount.toString(), 'recipientAmount')
    expect(actual.data.counterpartyAmount.toString()).to.equal(
      expected.data.counterpartyAmount.toString(),
      'counterpartyAmount'
    )
    expect(actual.data.funder).to.equal(expected.data.funder, 'funder')

    const actualRecipientAddress = await pubKeyToAddress(actual.data.recipient)
    const actualCounterpartyAddress = await pubKeyToAddress(actual.data.counterparty)

    expect(actualRecipientAddress.toHex()).to.equal('0x74cdeAb5AE6efFDC70699731EC3ab55f35fb41eA')
    expect(actualCounterpartyAddress.toHex()).to.equal('0x044aEf65B5A3f18ab9aD3A09012747C0397b9089')
  })

  it('should convert OPENED log to OPENED event', async function () {
    const actual = logs.toOpenedChannelEvent(fixtures.OPENED_LOG)
    const expected = fixtures.OPENED_EVENT

    expectBaseEventsToBeEqual(actual, expected)
    expect(actual.data.opener.toHex()).to.equal(expected.data.opener.toHex(), 'opener')
    expect(actual.data.counterparty.toHex()).to.equal(expected.data.counterparty.toHex(), 'counterparty')

    const actualOpenerAddress = await pubKeyToAddress(actual.data.opener)
    const actualCounterpartyAddress = await pubKeyToAddress(actual.data.counterparty)

    expect(actualOpenerAddress.toHex()).to.equal('0xf73A34405D1349476B5500Ea0381A1fcc87e8AEb')
    expect(actualCounterpartyAddress.toHex()).to.equal('0x20516d47c46Bcd67D19898FDA6aE8A68B3022429')
  })
})

const expectBaseEventsToBeEqual = (actual: Event<any>, expected: Event<any>) => {
  expect(actual.transactionHash).to.equal(expected.transactionHash, 'transactionHash')
  expect(actual.blockNumber.toString()).to.equal(expected.blockNumber.toString(), 'blockNumber')
  expect(actual.transactionIndex.toString()).to.equal(expected.transactionIndex.toString(), 'transactionIndex')
  expect(actual.logIndex.toString()).to.equal(expected.logIndex.toString(), 'logIndex')
}
