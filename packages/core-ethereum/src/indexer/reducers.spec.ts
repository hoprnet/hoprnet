import chai, { expect } from 'chai'
import chaiAsPromised from 'chai-as-promised'
import * as reducers from './reducers'
import * as fixtures from './reducers.fixtures.spec'
import { expectAccountsToBeEqual, expectChannelsToBeEqual } from './fixtures'

chai.use(chaiAsPromised)

// @TODO: add more tests
describe('test indexer reducers', function () {
  it("should create FUNDED channel entry when it's a new channel", async function () {
    const channelEntry = await reducers.onChannelFunded(fixtures.FUNDED_EVENT)
    expectChannelsToBeEqual(channelEntry, fixtures.FUNDED_CHANNEL)
  })

  it('should reduce to FUNDED_2 channel entry', async function () {
    const channelEntry = await reducers.onChannelFunded(fixtures.FUNDED_EVENT_2, fixtures.FUNDED_CHANNEL)
    expectChannelsToBeEqual(channelEntry, fixtures.FUNDED_CHANNEL_2)
  })

  it('should reduce to OPENED channel entry', async function () {
    const channelEntry = await reducers.onChannelOpened(fixtures.OPENED_EVENT, fixtures.FUNDED_CHANNEL_2)
    expectChannelsToBeEqual(channelEntry, fixtures.OPENED_CHANNEL)
  })

  it('should reduce to REDEEMED channel entry', async function () {
    const channelEntry = await reducers.onTicketRedeemed(fixtures.REDEEMED_EVENT, fixtures.OPENED_CHANNEL)
    expectChannelsToBeEqual(channelEntry, fixtures.REDEEMED_CHANNEL)
  })

  it('should reduce to CLOSING channel entry', async function () {
    const channelEntry = await reducers.onChannelPendingToClose(fixtures.CLOSING_EVENT, fixtures.REDEEMED_CHANNEL)
    expectChannelsToBeEqual(channelEntry, fixtures.CLOSING_CHANNEL)
  })

  it('should reduce to REDEEMED_2 channel entry', async function () {
    const channelEntry = await reducers.onTicketRedeemed(fixtures.REDEEMED_EVENT_2, fixtures.CLOSING_CHANNEL)
    expectChannelsToBeEqual(channelEntry, fixtures.REDEEMED_CHANNEL_2)
  })

  it('should reduce to CLOSED channel entry', async function () {
    const channelEntry = await reducers.onChannelClosed(fixtures.CLOSED_EVENT, fixtures.REDEEMED_CHANNEL_2)
    expectChannelsToBeEqual(channelEntry, fixtures.CLOSED_CHANNEL)
  })

  it('should fail to reduce UNINITIALIZED -> OPEN', async function () {
    expect(reducers.onChannelOpened(fixtures.OPENED_EVENT, fixtures.EMPTY_CHANNEL)).rejectedWith(
      ".onChannelOpened' failed because channel is not in 'FUNDED' status"
    )
  })

  it('should fail to reduce FUNDED -> CLOSING', async function () {
    expect(reducers.onChannelPendingToClose(fixtures.CLOSING_EVENT, fixtures.FUNDED_CHANNEL)).to.be.rejectedWith(
      ".onChannelPendingToClose' failed because channel is not in 'OPEN' status"
    )
  })

  it('should fail to reduce FUNDED -> REDEEM', async function () {
    expect(reducers.onTicketRedeemed(fixtures.REDEEMED_EVENT, fixtures.FUNDED_CHANNEL)).to.be.rejectedWith(
      ".onTicketRedeemed' failed because channel is not in 'OPEN' or 'PENDING' status"
    )
  })

  it('should fail to reduce OPENED -> UNINITIALIZED', async function () {
    expect(reducers.onChannelClosed(fixtures.CLOSED_EVENT, fixtures.OPENED_CHANNEL)).to.be.rejectedWith(
      ".onChannelClosed' failed because channel is not in 'PENDING' status"
    )
  })

  it('should fail to reduce FUNDED -> REDEEM', async function () {
    expect(reducers.onTicketRedeemed(fixtures.REDEEMED_EVENT, fixtures.FUNDED_CHANNEL)).to.be.rejectedWith(
      ".onTicketRedeemed' failed because channel is not in 'OPEN' or 'PENDING' status"
    )
  })

  it('should fail to reduce CLOSED -> REDEEM', async function () {
    expect(reducers.onTicketRedeemed(fixtures.REDEEMED_EVENT, fixtures.CLOSED_CHANNEL)).to.be.rejectedWith(
      ".onTicketRedeemed' failed because channel is not in 'OPEN' or 'PENDING' status"
    )
  })

  it("should create INITIALIZED account entry when it's a new account", async function () {
    const accountEntry = await reducers.onAccountInitialized(fixtures.ACCOUNT_INITIALIZED_EVENT)
    expectAccountsToBeEqual(accountEntry, fixtures.INITIALIZED_ACCOUNT)
  })

  it('should reduce to SECRET_UPDATED account entry', async function () {
    const accountEntry = await reducers.onAccountSecretUpdated(
      fixtures.ACCOUNT_SECRET_UPDATED_EVENT,
      fixtures.INITIALIZED_ACCOUNT
    )
    expectAccountsToBeEqual(accountEntry, fixtures.SECRET_UPDATED_ACCOUNT)
  })
})
