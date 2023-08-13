import request from 'supertest'
import sinon from 'sinon'
import chaiResponseValidator from 'chai-openapi-response-validator'
import chai, { expect } from 'chai'
import { createTestApiInstance, ALICE_PEER_ID, INVALID_PEER_ID, TICKET_MOCK } from '../../../fixtures.js'
import { STATUS_CODES } from '../../../utils.js'

let node = sinon.fake() as any
node.getTickets = sinon.fake.returns([TICKET_MOCK])

describe('GET /channels/{peerId}/tickets', () => {
  let service: any
  before(async function () {
    const loaded = await createTestApiInstance(node)

    service = loaded.service

    // @ts-ignore ESM / CommonJS compatibility issue
    chai.use(chaiResponseValidator.default(loaded.api.apiDoc))
  })

  it('should get tickets successfully', async () => {
    const res = await request(service).get(`/api/v2/channels/${ALICE_PEER_ID.toString()}/tickets`)
    expect(res).to.satisfyApiSpec
  })

  it('should fail when no tickets to get', async () => {
    node.getTickets = sinon.fake.returns([])
    const res = await request(service).get(`/api/v2/channels/${ALICE_PEER_ID.toString()}/tickets`)
    expect(res.status).to.equal(404)
    expect(res).to.satisfyApiSpec
    expect(res.body).to.deep.equal({ status: STATUS_CODES.TICKETS_NOT_FOUND })
  })

  it('should validate peerId', async () => {
    const res = await request(service).get(`/api/v2/channels/${INVALID_PEER_ID}/tickets`)
    expect(res.status).to.equal(400)
    expect(res).to.satisfyApiSpec
    expect(res.body).to.deep.equal({
      status: STATUS_CODES.INVALID_PEERID
    })
  })

  it('should fail when node call fails', async () => {
    node.getTickets = sinon.fake.throws('')

    const res = await request(service).get(`/api/v2/channels/${ALICE_PEER_ID.toString()}/tickets`)
    expect(res.status).to.equal(422)
    expect(res).to.satisfyApiSpec
  })
})
