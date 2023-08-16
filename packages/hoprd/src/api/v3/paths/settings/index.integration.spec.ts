import request from 'supertest'
import sinon from 'sinon'
import chaiResponseValidator from 'chai-openapi-response-validator'
import chai, { expect } from 'chai'
import { createTestApiInstance } from '../../fixtures.js'

let node = sinon.fake() as any
node.getChannelStrategy = sinon.fake.returns('passive')

describe('GET /settings', () => {
  let service: any
  before(async function () {
    const loaded = await createTestApiInstance(node)

    service = loaded.service

    // @ts-ignore ESM / CommonJS compatibility issue
    chai.use(chaiResponseValidator.default(loaded.api.apiDoc))
  })

  it('should return all settings', async () => {
    const res = await request(service).get(`/api/v3/settings`)
    expect(res.status).to.equal(200)
    expect(res).to.satisfyApiSpec
  })
})
