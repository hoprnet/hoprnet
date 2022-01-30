import assert from 'assert'
import sinon from 'sinon'
import { _createTestState } from '../../../v2/'
import { redeemTickets } from './redeemTickets'

let node = sinon.fake() as any

describe('redeemTickets', () => {
  it('should redeem tickets', async () => {
    await redeemTickets({ node })
  })
  it('fails when node call fails', async () => {
    node.redeemAllTickets = sinon.fake.rejects('')
    try {
      await redeemTickets({ node })
    } catch (error) {
      return assert(error.message.includes('failure'))
    }
    throw Error
  })
})
