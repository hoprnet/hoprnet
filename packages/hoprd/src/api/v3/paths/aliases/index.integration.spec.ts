import request from 'supertest'
import sinon from 'sinon'
import chaiResponseValidator from 'chai-openapi-response-validator'
import chai, { expect } from 'chai'

import { createTestApiInstance, INVALID_PEER_ID, ALICE_PEER_ID } from '../../fixtures.js'
import { STATUS_CODES } from '../../utils.js'

let node = sinon.fake() as any

const ALIAS = 'some_alias'

describe('GET /aliases', () => {
  let service: any
  before(async function () {
    const loaded = await createTestApiInstance(node)

    service = loaded.service

    // @ts-ignore ESM / CommonJS compatibility issue
    chai.use(chaiResponseValidator.default(loaded.api.apiDoc))
  })

  it('should successfuly get aliases', async () => {
    await request(service).post('/api/v3/aliases').send({
      peerId: ALICE_PEER_ID.toString(),
      alias: ALIAS
    })

    const res = await request(service).get(`/api/v3/aliases`)
    expect(res.status).to.equal(200)
    expect(res).to.satisfyApiSpec
    expect(res.body).to.deep.equal({
      [ALIAS]: ALICE_PEER_ID.toString()
    })
  })
})

describe('POST /aliases', () => {
  let service: any
  before(async function () {
    const loaded = await createTestApiInstance(node)

    service = loaded.service
  })
  it('should set alias successfuly', async () => {
    const res = await request(service).post('/api/v3/aliases').send({
      peerId: ALICE_PEER_ID.toString(),
      alias: ALIAS
    })
    expect(res.status).to.equal(201)
    expect(res).to.satisfyApiSpec
    expect(res.body).to.be.empty
  })
  it('should return 400 error on invalid peerId', async () => {
    const res = await request(service).post('/api/v3/aliases').send({
      peerId: INVALID_PEER_ID,
      alias: ALIAS
    })
    expect(res.status).to.equal(400)
    expect(res).to.satisfyApiSpec
    expect(res.body).to.deep.equal({
      status: STATUS_CODES.INVALID_PEERID
    })
  })
})
