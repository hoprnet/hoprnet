import assert from 'assert'
import sinon from 'sinon'
import { getChannels, openChannel } from '.'
import { Balance, ChannelEntry, NativeBalance } from '@hoprnet/hopr-utils'
import { ALICE_PEER_ID, INVALID_PEER_ID } from '../../fixtures'
import BN from 'bn.js'
import { STATUS_CODES } from '../../utils'

let node = sinon.fake() as any
node.getId = sinon.fake.returns(ALICE_PEER_ID)

const CHANNEL_ID = ChannelEntry.createMock().getId()

describe('getChannels', function () {
  const testChannel = ChannelEntry.createMock()
  node.getChannelsFrom = sinon.fake.returns(Promise.resolve([testChannel]))
  node.getChannelsTo = sinon.fake.returns(Promise.resolve([testChannel]))

  it('should get channels list including closed', async function () {
    const { incoming, outgoing } = await getChannels(node, true)
    assert.equal(incoming.length, 1)
    assert.equal(outgoing.length, 1)
  })
  it('should get channels list excluding closed', async function () {
    const { incoming, outgoing } = await getChannels(node, false)

    assert.equal(incoming.length, 0)
    assert.equal(outgoing.length, 0)
  })
})

node.getNativeBalance = sinon.fake.returns(new NativeBalance(new BN(10)))
node.getBalance = sinon.fake.returns(new Balance(new BN(1)))
node.openChannel = sinon.fake.returns(
  Promise.resolve({
    channelId: CHANNEL_ID,
    receipt: 'testReceipt'
  })
)

describe('openChannel', () => {
  it('should open channel', async () => {
    const channel = await openChannel(node, ALICE_PEER_ID.toB58String(), '1')
    assert.deepEqual(channel, {
      channelId: CHANNEL_ID.toHex(),
      receipt: 'testReceipt'
    })
  })

  it('should fail on invalid peerId or amountToFund', async () => {
    assert.rejects(
      () => {
        return openChannel(node, INVALID_PEER_ID, '1')
      },
      (err: Error) => {
        return err.message.includes(STATUS_CODES.INVALID_PEERID)
      }
    )
    assert.rejects(
      () => {
        return openChannel(node, ALICE_PEER_ID.toB58String(), 'abc')
      },
      (err: Error) => {
        return err.message.includes(STATUS_CODES.INVALID_AMOUNT)
      }
    )
    assert.rejects(
      () => {
        return openChannel(node, ALICE_PEER_ID.toB58String(), '10000000')
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
        return openChannel(node, ALICE_PEER_ID.toB58String(), '1')
      },
      (err: Error) => {
        return err.message.includes(STATUS_CODES.CHANNEL_ALREADY_OPEN)
      }
    )
  })
})
