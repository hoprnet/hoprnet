import assert from 'assert'
import sinon from 'sinon'
import { ChannelEntry } from '@hoprnet/hopr-utils'
import { STATUS_CODES } from '../../utils'
import { invalidTestPeerId, testPeerId, testPeerIdInstance, ALICE_PEER_ID as COUNTERPARTY } from '../../fixtures'
import { getChannel, closeChannel } from './{peerid}'
import { formatIncomingChannel, formatOutgoingChannel } from '.'

let node = sinon.fake() as any

describe('getChannel', function () {
  const outgoingMock = ChannelEntry.createMock()
  const incomingMock = ChannelEntry.createMock()
  node.getId = sinon.fake.returns(testPeerIdInstance)

  it('should return no channels', async function () {
    assert.rejects(
      () => {
        return getChannel(node, COUNTERPARTY.toB58String(), 'incoming')
      },
      (err: Error) => {
        return err.message.includes(STATUS_CODES.CHANNEL_NOT_FOUND)
      }
    )
    assert.rejects(
      () => {
        return getChannel(node, COUNTERPARTY.toB58String(), 'outgoing')
      },
      (err: Error) => {
        return err.message.includes(STATUS_CODES.CHANNEL_NOT_FOUND)
      }
    )
  })

  it('should return outgoing channel', async function () {
    node.getChannel = sinon.stub()
    node.getChannel.withArgs(testPeerIdInstance, COUNTERPARTY).resolves(outgoingMock)

    const outgoing = await getChannel(node, COUNTERPARTY.toB58String(), 'outgoing')
    assert.notEqual(outgoing, undefined)
    assert.deepEqual(outgoing, formatOutgoingChannel(outgoingMock))
  })

  it('should return outgoing and incoming channels', async function () {
    node.getChannel = sinon.stub()
    node.getChannel.withArgs(testPeerIdInstance, COUNTERPARTY).resolves(outgoingMock)
    node.getChannel.withArgs(COUNTERPARTY, testPeerIdInstance).resolves(incomingMock)

    const outgoing = await getChannel(node, COUNTERPARTY.toB58String(), 'outgoing')
    assert.notEqual(outgoing, undefined)
    assert.deepEqual(outgoing, formatOutgoingChannel(outgoingMock))
    const incoming = await getChannel(node, COUNTERPARTY.toB58String(), 'incoming')
    assert.notEqual(incoming, undefined)
    assert.deepEqual(incoming, formatIncomingChannel(incomingMock))
  })
})

describe('closeChannel', () => {
  it('should close channel', async () => {
    const expectedStatus = { channelStatus: 2, receipt: 'receipt' }
    node.closeChannel = sinon.fake.returns({ status: expectedStatus.channelStatus, receipt: expectedStatus.receipt })

    const closureStatus = await closeChannel(node, testPeerId, 'outgoing')
    assert.deepEqual(closureStatus, expectedStatus)
  })

  it('should fail on invalid peerId', async () => {
    const expectedStatus = { channelStatus: 3, receipt: 'receipt' }
    node.closeChannel = sinon.fake.returns({ status: expectedStatus.channelStatus, receipt: expectedStatus.receipt })

    assert.rejects(
      () => {
        return closeChannel(node, invalidTestPeerId, 'outgoing')
      },
      (err: Error) => {
        return err.message.includes(STATUS_CODES.INVALID_PEERID)
      }
    )
  })

  it('should fail when node call fails', async () => {
    node.closeChannel = sinon.fake.throws('unknown error')

    assert.rejects(
      () => {
        return closeChannel(node, testPeerId, 'outgoing')
      },
      // we only care if it throws
      (_err: Error) => {
        return true
      }
    )
  })
})
