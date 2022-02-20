import request from 'supertest'
import sinon from 'sinon'
import chaiResponseValidator from 'chai-openapi-response-validator'
import chai, { expect } from 'chai'
import { createTestApiInstance, invalidTestPeerId, testAlias, testPeerId } from '../../fixtures'
import { STATUS_CODES } from '../../utils'

let node = sinon.fake() as any

const { api, service } = createTestApiInstance(node)
chai.use(chaiResponseValidator(api.apiDoc))

describe('GET /aliases', () => {
  it('should successfuly get aliases', async () => {
    await request(service).post('/api/v2/aliases').send({
      peerId: testPeerId,
      alias: testAlias
    })

    const res = await request(service).get(`/api/v2/aliases`)
    expect(res.status).to.equal(200)
    expect(res).to.satisfyApiSpec
    expect(res.body).to.deep.equal({
      [testAlias]: testPeerId
    })
  })
})

describe('POST /aliases', () => {
  it('should set alias successfuly', async () => {
    const res = await request(service).post('/api/v2/aliases').send({
      peerId: testPeerId,
      alias: testAlias
    })
    expect(res.status).to.equal(201)
    expect(res).to.satisfyApiSpec
    expect(res.body).to.be.empty
  })
  it('should return 400 error on invalid peerId', async () => {
    const res = await request(service).post('/api/v2/aliases').send({
      peerId: invalidTestPeerId,
      alias: testAlias
    })
    expect(res.status).to.equal(400)
    expect(res).to.satisfyApiSpec
    expect(res.body).to.deep.equal({
      status: STATUS_CODES.INVALID_PEERID
    })
  })
})
