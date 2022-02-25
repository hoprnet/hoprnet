import request from 'supertest'
import sinon from 'sinon'
import chaiResponseValidator from 'chai-openapi-response-validator'
import chai, { expect } from 'chai'
import { createTestApiInstance } from '../../fixtures'

let node = sinon.fake() as any
node.getChannelStrategy = sinon.fake.returns('passive')

const { api, service } = createTestApiInstance(node)
chai.use(chaiResponseValidator(api.apiDoc))

describe('GET /settings', () => {
  it('should return all settings', async () => {
    const res = await request(service).get(`/api/v2/settings`)
    expect(res.status).to.equal(200)
    expect(res).to.satisfyApiSpec
  })
})
