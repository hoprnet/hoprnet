import type BN from 'bn.js'
import type { ChannelEntry, AccountEntry } from '../types'
import chai, { expect } from 'chai'
import chaiAsPromised from 'chai-as-promised'
import { u8aToHex } from '@hoprnet/hopr-utils'
import * as reducers from './reducers'
import * as fixtures from './reducers.fixtures.spec'

chai.use(chaiAsPromised)

// @TODO: add more tests
describe('test indexer reducers', function () {
  context('channels', function () {
    it("should create FUNDED channel entry when it's a new channel", async function () {
      const channelEntry = await reducers.onFundedChannel(fixtures.FUNDED_EVENT)
      expectChannelEntriesToBeEqual(channelEntry, fixtures.FUNDED_CHANNEL)
    })

    it('should reduce to FUNDED_2 channel entry', async function () {
      const channelEntry = await reducers.onFundedChannel(fixtures.FUNDED_EVENT_2, fixtures.FUNDED_CHANNEL)
      expectChannelEntriesToBeEqual(channelEntry, fixtures.FUNDED_CHANNEL_2)
    })

    it('should reduce to OPENED channel entry', async function () {
      const channelEntry = await reducers.onOpenedChannel(fixtures.OPENED_EVENT, fixtures.FUNDED_CHANNEL_2)
      expectChannelEntriesToBeEqual(channelEntry, fixtures.OPENED_CHANNEL)
    })

    it('should reduce to REDEEMED channel entry', async function () {
      const channelEntry = await reducers.onRedeemedTicket(fixtures.REDEEMED_EVENT, fixtures.OPENED_CHANNEL)
      expectChannelEntriesToBeEqual(channelEntry, fixtures.REDEEMED_CHANNEL)
    })

    it('should reduce to CLOSING channel entry', async function () {
      const channelEntry = await reducers.onInitiatedChannelClosure(fixtures.CLOSING_EVENT, fixtures.REDEEMED_CHANNEL)
      expectChannelEntriesToBeEqual(channelEntry, fixtures.CLOSING_CHANNEL)
    })

    it('should reduce to REDEEMED_2 channel entry', async function () {
      const channelEntry = await reducers.onRedeemedTicket(fixtures.REDEEMED_EVENT_2, fixtures.CLOSING_CHANNEL)
      expectChannelEntriesToBeEqual(channelEntry, fixtures.REDEEMED_CHANNEL_2)
    })

    it('should reduce to CLOSED channel entry', async function () {
      const channelEntry = await reducers.onClosedChannel(fixtures.CLOSED_EVENT, fixtures.REDEEMED_CHANNEL_2)
      expectChannelEntriesToBeEqual(channelEntry, fixtures.CLOSED_CHANNEL)
    })

    it('should fail to reduce UNINITIALIZED -> OPEN', async function () {
      expect(reducers.onOpenedChannel(fixtures.OPENED_EVENT, fixtures.EMPTY_CHANNEL)).rejectedWith(
        "'onOpenedChannel' failed because channel is not in 'FUNDED' status"
      )
    })

    it('should fail to reduce FUNDED -> CLOSING', async function () {
      expect(reducers.onInitiatedChannelClosure(fixtures.CLOSING_EVENT, fixtures.FUNDED_CHANNEL)).to.be.rejectedWith(
        "'onInitiatedChannelClosure' failed because channel is not in 'OPEN' status"
      )
    })

    it('should fail to reduce FUNDED -> REDEEM', async function () {
      expect(reducers.onRedeemedTicket(fixtures.REDEEMED_EVENT, fixtures.FUNDED_CHANNEL)).to.be.rejectedWith(
        "'onRedeemedTicket' failed because channel is not in 'OPEN' or 'PENDING' status"
      )
    })

    it('should fail to reduce OPENED -> UNINITIALIZED', async function () {
      expect(reducers.onClosedChannel(fixtures.CLOSED_EVENT, fixtures.OPENED_CHANNEL)).to.be.rejectedWith(
        "'onClosedChannel' failed because channel is not in 'PENDING' status"
      )
    })

    it('should fail to reduce FUNDED -> REDEEM', async function () {
      expect(reducers.onRedeemedTicket(fixtures.REDEEMED_EVENT, fixtures.FUNDED_CHANNEL)).to.be.rejectedWith(
        "'onRedeemedTicket' failed because channel is not in 'OPEN' or 'PENDING' status"
      )
    })

    it('should fail to reduce CLOSED -> REDEEM', async function () {
      expect(reducers.onRedeemedTicket(fixtures.REDEEMED_EVENT, fixtures.CLOSED_CHANNEL)).to.be.rejectedWith(
        "'onRedeemedTicket' failed because channel is not in 'OPEN' or 'PENDING' status"
      )
    })
  })

  context('accounts', function () {
    it('should create account entry when hashed secret is set', async function () {
      const accountEntry = await reducers.onSecretHashSet(fixtures.SECRET_SET_EVENT)
      expectAccountEntriesToBeEqual(accountEntry, fixtures.INITIALIZED_ACCOUNT)
    })
  })
})

const expectChannelEntriesToBeEqual = (actual: ChannelEntry, expected: ChannelEntry) => {
  expectLogPositionsToBeEqual(actual, expected)
  expect(actual.deposit.toString()).to.equal(expected.deposit.toString(), 'deposit')
  expect(actual.partyABalance.toString()).to.equal(expected.partyABalance.toString(), 'partyABalance')
  expect(actual.closureTime.toString()).to.equal(expected.closureTime.toString(), 'closureTime')
  expect(actual.stateCounter.toString()).to.equal(expected.stateCounter.toString(), 'stateCounter')
  expect(actual.closureByPartyA).to.equal(actual.closureByPartyA, 'closureByPartyA')
}

const expectAccountEntriesToBeEqual = (actual: AccountEntry, expected: AccountEntry) => {
  expectLogPositionsToBeEqual(actual, expected)
  expect(u8aToHex(actual.hashedSecret)).to.equal(u8aToHex(expected.hashedSecret), 'hashedSecret')
  expect(actual.counter.toString()).to.equal(expected.counter.toString(), 'counter')
}

const expectLogPositionsToBeEqual = (
  actual: { blockNumber: BN; transactionIndex: BN; logIndex: BN },
  expected: { blockNumber: BN; transactionIndex: BN; logIndex: BN }
) => {
  expect(actual.blockNumber.toString()).to.equal(expected.blockNumber.toString(), 'blockNumber')
  expect(actual.transactionIndex.toString()).to.equal(expected.transactionIndex.toString(), 'transactionIndex')
  expect(actual.logIndex.toString()).to.equal(expected.logIndex.toString(), 'logIndex')
}
