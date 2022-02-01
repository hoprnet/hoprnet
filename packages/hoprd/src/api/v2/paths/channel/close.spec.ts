import assert from 'assert'
import sinon from 'sinon'
import { closeChannel } from './close'
import { STATUS_CODES } from '../../'
import { invalidTestPeerId, testPeerId } from '../../fixtures'

let node = sinon.fake() as any

describe('closeChannel', () => {
  it('should close channel', async () => {
    const expectedStatus = { channelStatus: 2, receipt: 'receipt' }
    node.closeChannel = sinon.fake.returns({ status: expectedStatus.channelStatus, receipt: expectedStatus.receipt })

    const closureStatus = await closeChannel(node, testPeerId)
    assert.deepEqual(closureStatus, expectedStatus)
  })

  it('should fail on invalid peerId', async () => {
    const expectedStatus = { channelStatus: 3, receipt: 'receipt' }
    node.closeChannel = sinon.fake.returns({ status: expectedStatus.channelStatus, receipt: expectedStatus.receipt })

    assert.rejects(
      () => {
        return closeChannel(node, invalidTestPeerId)
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
        return closeChannel(node, testPeerId)
      },
      // we only care if it throws
      (_err: Error) => {
        return true
      }
    )
  })
})
