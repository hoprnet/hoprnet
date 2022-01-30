import assert from 'assert'
import sinon from 'sinon'
import { closeChannel } from './close'
import { STATUS_CODES } from '../../'

const peerId = '16Uiu2HAmRFjDov6sbcZeppbnNFFTdx5hFoBzr8csBgevtKUex8y9'
const invalidPeerId = 'definetly not a valid peerId'

let node = sinon.fake() as any

describe('closeChannel', () => {
  it('should close channel', async () => {
    const expectedStatus = { channelStatus: 2, receipt: 'receipt' }
    node.closeChannel = sinon.fake.returns({ status: expectedStatus.channelStatus, receipt: expectedStatus.receipt })

    const closureStatus = await closeChannel(node, peerId)
    assert.deepEqual(closureStatus, expectedStatus)
  })

  it('should fail on invalid peerId', async () => {
    const expectedStatus = { channelStatus: 3, receipt: 'receipt' }
    node.closeChannel = sinon.fake.returns({ status: expectedStatus.channelStatus, receipt: expectedStatus.receipt })

    assert.rejects(() => {
      return closeChannel(node, invalidPeerId)
    }, STATUS_CODES.INVALID_PEERID)
  })

  it('should fail when node call fails', async () => {
    const expectedStatus = { channelStatus: 3, receipt: 'receipt' }
    node.closeChannel = sinon.fake.throws('unknown error')

    assert.rejects(() => {
      return closeChannel(node, peerId)
    }, STATUS_CODES.UNKNOWN_FAILURE)
  })
})
