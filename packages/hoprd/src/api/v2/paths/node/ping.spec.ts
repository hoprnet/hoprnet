import sinon from 'sinon'
import assert from 'assert'
import { ping } from './ping'
import { STATUS_CODES } from '../../'

const peerId = '16Uiu2HAmRFjDov6sbcZeppbnNFFTdx5hFoBzr8csBgevtKUex8y9'
const invalidPeerId = 'definetly not a valid peerId'

let node = sinon.fake() as any

describe('ping', () => {
  it('should ping successfuly', async () => {
    node.ping = sinon.fake.returns({ latency: 10 })
    const res = await ping({ node, peerId })
    assert.equal(res.latency, 10)
  })
  it('should return error on invalid peerId', async () => {
    node.ping = sinon.fake.returns({ latency: 10 })
    assert.throws(() => ping({ node, peerId: invalidPeerId }), STATUS_CODES.INVALID_PEERID)
  })
  it('should return propper error on ping fail', async () => {
    node.ping = sinon.fake.throws('')
    assert.throws(() => ping({ node, peerId }))
  })
})
