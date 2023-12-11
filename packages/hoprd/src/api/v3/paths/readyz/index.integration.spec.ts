import request from 'supertest'
import sinon from 'sinon'
import chaiResponseValidator from 'chai-openapi-response-validator'
import chai, { expect } from 'chai'
import { createTestApiInstance } from '../../fixtures.js'
import { Hopr } from '@hoprnet/hopr-utils'

let node = sinon.fake() as Hopr

describe('GET /readyz', () => {
  let service: any
  before(async function () {
    const loaded = await createTestApiInstance(node)

    service = loaded.service

    // @ts-ignore ESM / CommonJS compatibility issue
    chai.use(chaiResponseValidator.default(loaded.api.apiDoc))
  })

  it('should return 200 when started', async () => {
    node.isRunning = sinon.fake.returns(true)

    const res = await request(service).get(`/api/v3/readyz`)
    expect(res.status).to.equal(200)
    expect(res).to.satisfyApiSpec
  })

  it('should not return 200 when not started', async () => {
    node.isRunning = sinon.fake.returns(false)

    const res = await request(service).get(`/api/v3/readyz`)
    expect(res.status).to.equal(422)
    expect(res).to.satisfyApiSpec
  })
})
