import assert from 'assert'
import sinon from 'sinon'
import BN from 'bn.js'
import { Balance, NativeBalance } from '@hoprnet/hopr-utils'
import { STATUS_CODES } from '../../'
import { openChannel } from './open'
import { testPeerId, testChannelId, invalidTestPeerId } from '../../fixtures'

let node = sinon.fake() as any
node.getNativeBalance = sinon.fake.returns(new NativeBalance(new BN(10)))
node.getBalance = sinon.fake.returns(new Balance(new BN(1)))
node.openChannel = sinon.fake.returns(
  Promise.resolve({
    channelId: testChannelId
  })
)

describe('openChannel', () => {
  it('should open channel', async () => {
    const channelId = await openChannel(node, testPeerId, '1')
    assert.equal(channelId, channelId)
  })

  it('should fail on invalid peerId or amountToFund', async () => {
    assert.rejects(
      () => {
        return openChannel(node, invalidTestPeerId, '1')
      },
      (err: Error) => {
        return err.message.includes(STATUS_CODES.INVALID_PEERID)
      }
    )
    assert.rejects(
      () => {
        return openChannel(node, testPeerId, 'abc')
      },
      (err: Error) => {
        return err.message.includes(STATUS_CODES.INVALID_AMOUNT)
      }
    )
    assert.rejects(
      () => {
        return openChannel(node, testPeerId, '10000000')
      },
      (err: Error) => {
        return err.message.includes(STATUS_CODES.NOT_ENOUGH_BALANCE)
      }
    )
  })

  it('should fail when channel is already open', async () => {
    node.openChannel = sinon.fake.throws('Channel is already opened')

    assert.rejects(
      () => {
        return openChannel(node, testPeerId, '1')
      },
      (err: Error) => {
        return err.message.includes(STATUS_CODES.CHANNEL_ALREADY_OPEN)
      }
    )
  })
})
