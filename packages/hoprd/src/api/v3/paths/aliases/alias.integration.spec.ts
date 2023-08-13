import request from 'supertest'
import sinon from 'sinon'
import chaiResponseValidator from 'chai-openapi-response-validator'
import chai, { expect } from 'chai'
import { createTestApiInstance, ALICE_PEER_ID } from '../../fixtures.js'
import { STATUS_CODES } from '../../utils.js'

let node = sinon.fake() as any

const ALIAS = 'some_alias'

describe('GET /aliases/{alias}', () => {
  let service: any
  before(async function () {
    const loaded = await createTestApiInstance(node)

    service = loaded.service

    // @ts-ignore ESM / CommonJS compatibility issue
    chai.use(chaiResponseValidator.default(loaded.api.apiDoc))
  })

  it('should successfuly get alias', async () => {
    await request(service).post('/api/v3/aliases').send({
      peerId: ALICE_PEER_ID.toString(),
      alias: ALIAS
    })

    const res = await request(service).get(`/api/v3/aliases/${ALIAS}`)
    expect(res.status).to.equal(200)
    expect(res).to.satisfyApiSpec
    expect(res.body).to.deep.equal({
      peerId: ALICE_PEER_ID.toString()
    })
  })
  it('should return 404 on invalid peerId', async () => {
    const res = await request(service).get(`/api/v3/aliases/nonExistingAlias`)

    expect(res.status).to.equal(404)
    expect(res).to.satisfyApiSpec
    expect(res.body).to.deep.equal({ status: STATUS_CODES.PEERID_NOT_FOUND })
  })
})

describe('DELETE /aliases/{alias}', () => {
  let service: any
  before(async function () {
    const loaded = await createTestApiInstance(node)

    service = loaded.service

    // @ts-ignore ESM / CommonJS compatibility issue
    chai.use(chaiResponseValidator.default(loaded.api.apiDoc))
  })

  it('should remove alias successfuly', async () => {
    await request(service).post('/api/v3/aliases').send({
      peerId: ALICE_PEER_ID.toString(),
      alias: ALIAS
    })

    const res = await request(service).delete(`/api/v3/aliases/${ALIAS}`)

    expect(res.status).to.equal(204)
    expect(res).to.satisfyApiSpec
    expect(res.body).to.be.empty
  })
  it("should return 204 even if the alias doesn't exist", async () => {
    await request(service).post('/api/v3/aliases').send({
      peerId: ALICE_PEER_ID.toString(),
      alias: 'nonExistingAlias'
    })

    const res = await request(service).delete(`/api/v3/aliases/${ALIAS}`)

    expect(res.status).to.equal(204)
    expect(res).to.satisfyApiSpec
    expect(res.body).to.be.empty
  })
})
