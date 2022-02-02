import assert from 'assert'
import sinon from 'sinon'
import { listChannels, openChannel } from './channels'
import { Balance, ChannelEntry, NativeBalance } from '@hoprnet/hopr-utils'
import { invalidTestPeerId, testChannelId, testPeerId, testPeerIdInstance } from '../fixtures'
import BN from 'bn.js'
import { STATUS_CODES } from '../'

let node = sinon.fake() as any
node.getId = sinon.fake.returns(testPeerIdInstance)

describe('listChannels', function () {
  const testChannel = ChannelEntry.createMock()
  node.getChannelsFrom = sinon.fake.returns(Promise.resolve([testChannel]))
  node.getChannelsTo = sinon.fake.returns(Promise.resolve([testChannel]))

  it('should get channels list including closed', async function () {
    const { incoming, outgoing } = await listChannels(node, true)
    assert.equal(incoming.length, 1)
    assert.equal(outgoing.length, 1)
  })
  it('should get channels list excluding closed', async function () {
    const { incoming, outgoing } = await listChannels(node, false)

    assert.equal(incoming.length, 0)
    assert.equal(outgoing.length, 0)
  })
})

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
