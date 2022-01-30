import assert from 'assert'
import sinon from 'sinon'
import { _createTestState } from '../../../v2/'
import { closeChannel } from './close'

const peerId = '16Uiu2HAmRFjDov6sbcZeppbnNFFTdx5hFoBzr8csBgevtKUex8y9'
const invalidPeerId = 'definetly not a valid peerId'

let node = sinon.fake() as any

describe('closeChannel', () => {
  it('should close channel', async () => {
    const expectedStatus = { channelStatus: 2, receipt: 'receipt', closureWaitTime: 2 }

    node.closeChannel = sinon.fake.returns({ status: expectedStatus.channelStatus, receipt: expectedStatus.receipt })
    node.smartContractInfo = sinon.fake.returns({ channelClosureSecs: expectedStatus.closureWaitTime * 60 })
    const closureStatus = await closeChannel({ peerId, node })

    assert.deepEqual(closureStatus, expectedStatus)
  })
  it('should not return closureWaitTime if status === ChannelStatus.PendingToClose', async () => {
    const expectedStatus = { channelStatus: 3, receipt: 'receipt' }

    node.closeChannel = sinon.fake.returns({ status: expectedStatus.channelStatus, receipt: expectedStatus.receipt })
    node.smartContractInfo = sinon.fake.returns({ channelClosureSecs: 60 })
    const closureStatus = await closeChannel({ peerId, node })

    assert.deepEqual(closureStatus, expectedStatus)
  })

  it('should fail on invalid peerId', async () => {
    const expectedStatus = { channelStatus: 3, receipt: 'receipt' }

    node.closeChannel = sinon.fake.returns({ status: expectedStatus.channelStatus, receipt: expectedStatus.receipt })
    node.smartContractInfo = sinon.fake.returns({ channelClosureSecs: 60 })
    try {
      await closeChannel({ peerId: invalidPeerId, node })
    } catch (error) {
      return assert.equal(error.message, 'invalidPeerId')
    }
    throw Error()
  })
  it('should fail when node call fails', async () => {
    node.closeChannel = sinon.fake.rejects('')
    node.smartContractInfo = sinon.fake.rejects('')
    try {
      await closeChannel({ peerId: peerId, node })
    } catch (error) {
      return assert(error.message.includes('failure'))
    }
    throw Error()
  })
})
