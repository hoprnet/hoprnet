import request from 'supertest'
import sinon from 'sinon'
import chaiResponseValidator from 'chai-openapi-response-validator'
import chai, { expect } from 'chai'
import { createTestApiInstance, invalidTestPeerId, testPeerId } from '../../fixtures'
import { STATUS_CODES } from '../../utils'

let node = sinon.fake() as any
const { api, service } = createTestApiInstance(node)
chai.use(chaiResponseValidator(api.apiDoc))

describe('DELETE /channels/{peerId}', () => {
  it('should close channel', async () => {
    node.closeChannel = sinon.fake.returns({ status: 2, receipt: 'receipt' })

    const res = await request(service).delete(`/api/v2/channels/${testPeerId}`)
    expect(res.status).to.equal(200)
    expect(res).to.satisfyApiSpec
    expect(res.body).to.deep.equal({
      channelStatus: 'Open',
      receipt: 'receipt'
    })
  })

  it('should fail on invalid peerId', async () => {
    const expectedStatus = { channelStatus: 3, receipt: 'receipt' }
    node.closeChannel = sinon.fake.returns({ status: expectedStatus.channelStatus, receipt: expectedStatus.receipt })

    const res = await request(service).delete(`/api/v2/channels/${invalidTestPeerId}`)
    expect(res.status).to.equal(400)
    expect(res).to.satisfyApiSpec
    expect(res.body).to.deep.equal({ status: STATUS_CODES.INVALID_PEERID })
  })

  it('should fail when node call fails', async () => {
    node.closeChannel = sinon.fake.throws('unknown error')

    const res = await request(service).delete(`/api/v2/channels/${testPeerId}`)
    expect(res.status).to.equal(422)
    expect(res).to.satisfyApiSpec
  })
})
