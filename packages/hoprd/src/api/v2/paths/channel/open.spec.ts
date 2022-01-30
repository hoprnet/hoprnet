import assert from 'assert'
import sinon from 'sinon'
import { STATUS_CODES } from '../../'
import { openChannel } from './open'

const peerId = '16Uiu2HAmRFjDov6sbcZeppbnNFFTdx5hFoBzr8csBgevtKUex8y9'
const invalidPeerId = 'definetly not a valid peerId'

let node = sinon.fake() as any

describe('openChannel', () => {
  it('should open channel', async () => {
    node.getBalance = sinon.fake.returns({ native: '1', hopr: '10' })
    node.openChannel = sinon.fake.returns('hash')

    const channelId = await openChannel(node, peerId, '1')
    assert.equal(channelId, 'hash')
  })

  it('should fail on invalid peerId or amountToFund', async () => {
    node.getBalance = sinon.fake.returns({ native: '1', hopr: '10' })
    node.openChannel = sinon.fake.returns('hash')

    assert.rejects(() => {
      return openChannel(node, invalidPeerId, '1')
    }, STATUS_CODES.INVALID_PEERID)
    assert.rejects(() => {
      return openChannel(node, peerId, 'abc')
    }, STATUS_CODES.INVALID_AMOUNT)
    assert.rejects(() => {
      return openChannel(node, peerId, '10000000')
    }, STATUS_CODES.NOT_ENOUGH_BALANCE)
  })

  it('should fail when channel is already open', async () => {
    node.getBalance = sinon.fake.returns({ native: '1', hopr: '10' })
    node.openChannel = sinon.fake.throws('Channel is already opened')

    assert.rejects(() => {
      return openChannel(node, peerId, '1')
    }, STATUS_CODES.CHANNEL_ALREADY_OPEN)
  })
})
