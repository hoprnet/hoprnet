import assert from 'assert'
import { _createTestState } from '.'
import { openChannel } from './channel'
import sinon from 'sinon'

const peerId = '16Uiu2HAmRFjDov6sbcZeppbnNFFTdx5hFoBzr8csBgevtKUex8y9'
const invalidPeerId = 'definetly not a valid peerId'

let node = sinon.fake() as any

describe('openChannel', () => {
  it('should open channel', async () => {
    const state = _createTestState()
    node.getBalance = sinon.fake.returns({ native: '1', hopr: '10' })
    node.openChannel = sinon.fake.returns('hash')

    const channelId = await openChannel({ amountToFundStr: '0.01', counterpartyPeerId: peerId, node, state })
    assert.equal(channelId, 'hash')
  })

  it('should fail on invalid peerId or amountToFund', async () => {
    const state = _createTestState()
    node.getBalance = sinon.fake.returns({ native: '1', hopr: '10' })
    node.openChannel = sinon.fake.returns('hash')

    const channelId = (await openChannel({
      amountToFundStr: '0.01',
      counterpartyPeerId: invalidPeerId,
      node,
      state
    })) as Error
    assert.equal(channelId.message, 'invalidPeerId')
    const channelId2 = (await openChannel({
      amountToFundStr: '0.01abcd',
      counterpartyPeerId: peerId,
      node,
      state
    })) as Error
    assert.equal(channelId2.message, 'invalidAmountToFund')
  })

  it('should fail when channel is already open', async () => {
    const state = _createTestState()
    node.getBalance = sinon.fake.returns({ native: '1', hopr: '10' })
    node.openChannel = sinon.fake.throws('Channel is already opened')

    const channelId = (await openChannel({
      amountToFundStr: '0.01',
      counterpartyPeerId: invalidPeerId,
      node,
      state
    })) as Error
    assert.equal(channelId.message, 'channelAlreadyOpen')
  })

  it('should fail when amount to fund is bigger than current balance', async () => {
    const state = _createTestState()
    node.getBalance = sinon.fake.returns({ native: '1', hopr: '10' })
    node.openChannel = sinon.fake.throws('Channel is already opened')

    const channelId = (await openChannel({
      amountToFundStr: '1000000000',
      counterpartyPeerId: invalidPeerId,
      node,
      state
    })) as Error
    assert.equal(channelId.message, 'notEnoughFunds')
  })
})
