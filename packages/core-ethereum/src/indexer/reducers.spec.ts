import chai, { expect } from 'chai'
import chaiAsPromised from 'chai-as-promised'
import * as reducers from './reducers'
import * as fixtures from './reducers.fixtures.spec'

chai.use(chaiAsPromised)

// @TODO: add more tests
describe('test indexer reducers', function () {
  it('should fail to reduce FUNDED -> REDEEM', async function () {
    expect(reducers.onTicketRedeemed(fixtures.REDEEMED_EVENT, fixtures.FUNDED_CHANNEL)).to.be.rejectedWith(
      ".onTicketRedeemed' failed because channel is not in 'OPEN' or 'PENDING' status"
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
})
