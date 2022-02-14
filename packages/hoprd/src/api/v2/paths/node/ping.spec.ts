import sinon from 'sinon'
import assert from 'assert'
import { ping } from './ping'
import { STATUS_CODES } from '../../utils'
import { invalidTestPeerId, testPeerId } from '../../fixtures'

let node = sinon.fake() as any

describe('ping', () => {
  it('should ping successfuly', async () => {
    node.ping = sinon.fake.returns({ latency: 10 })
    const res = await ping({ node, peerId: testPeerId })
    assert.equal(res.latency, 10)
  })

  it('should return error on invalid peerId', async () => {
    node.ping = sinon.fake.returns({ latency: 10 })
    assert.rejects(
      () => ping({ node, peerId: invalidTestPeerId }),
      (err: Error) => {
        return err.message.includes(STATUS_CODES.INVALID_PEERID)
      }
    )
  })

  it('should return propper error on ping fail', async () => {
    node.ping = sinon.fake.throws('')
    assert.rejects(
      () => ping({ node, peerId: testPeerId }),
      () => {
        return true
      }
    )
  })
})
