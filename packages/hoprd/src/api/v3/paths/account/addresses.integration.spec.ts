import request from 'supertest'
import sinon from 'sinon'
import chaiResponseValidator from 'chai-openapi-response-validator'
import chai, { expect } from 'chai'

import { createTestApiInstance, ALICE_PEER_ID, ALICE_ETHEREUM_ADDR } from '../../fixtures.js'

let node = sinon.fake() as any

describe('GET /account/addresses', () => {
  let service: any
  before(async function () {
    const loaded = await createTestApiInstance(node)

    service = loaded.service

    // @ts-ignore ESM / CommonJS compatibility issue
    chai.use(chaiResponseValidator.default(loaded.api.apiDoc))
  })

  it('should return addresses', async () => {
    node.getId = sinon.fake.returns(ALICE_PEER_ID)
    node.getEthereumAddress = sinon.fake.returns(ALICE_ETHEREUM_ADDR)

    const res = await request(service).get('/api/v3/account/addresses')
    expect(res.status).to.equal(200)
    expect(res).to.satisfyApiSpec
    expect(res.body).to.deep.equal({
      native: ALICE_ETHEREUM_ADDR.to_string(),
      hopr: ALICE_PEER_ID.toString()
    })
  })
})
