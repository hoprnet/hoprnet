import request from 'supertest'
import sinon from 'sinon'
import chaiResponseValidator from 'chai-openapi-response-validator'
import chai, { expect } from 'chai'
import { createTestApiInstance, invalidTestPeerId, testPeerId } from '../../fixtures'
import { STATUS_CODES } from '../../utils'

let node = sinon.fake() as any
const { api, service } = createTestApiInstance(node)
chai.use(chaiResponseValidator(api.apiDoc))

describe('POST /node/ping', () => {
  it('should ping successfuly', async () => {
    node.ping = sinon.fake.returns({ latency: 10 })

    const res = await request(service).post(`/api/v2/node/ping`).send({ peerId: testPeerId })
    expect(res.status).to.equal(200)
    expect(res).to.satisfyApiSpec
    expect(res.body).to.deep.equal({ latency: 10 })
  })

  it('should return error on invalid peerId', async () => {
    node.ping = sinon.fake.returns({ latency: 10 })

    const res = await request(service).post(`/api/v2/node/ping`).send({ peerId: invalidTestPeerId })
    expect(res.status).to.equal(400)
    expect(res).to.satisfyApiSpec
    expect(res.body).to.deep.equal({ status: STATUS_CODES.INVALID_PEERID })
  })

  it('should return propper error on ping fail', async () => {
    node.ping = sinon.fake.throws('')

    const res = await request(service).post(`/api/v2/node/ping`).send({ peerId: testPeerId })
    expect(res.status).to.equal(422)
    expect(res).to.satisfyApiSpec
    expect(res.body.status).to.equal(STATUS_CODES.UNKNOWN_FAILURE)
  })
})
