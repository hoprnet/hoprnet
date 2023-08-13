import request from 'supertest'
import sinon from 'sinon'
import chaiResponseValidator from 'chai-openapi-response-validator'
import chai, { expect } from 'chai'
import { createTestApiInstance, ALICE_PEER_ID, INVALID_PEER_ID } from '../../../../fixtures.js'
import { STATUS_CODES } from '../../../../utils.js'

let node = sinon.fake() as any
node.getTickets = sinon.fake.returns(['ticket'])
node.redeemTicketsInChannel = sinon.fake()

describe('POST /channels/{peerId}/tickets/redeem', () => {
  let service: any
  before(async function () {
    const loaded = await createTestApiInstance(node)

    service = loaded.service

    // @ts-ignore ESM / CommonJS compatibility issue
    chai.use(chaiResponseValidator.default(loaded.api.apiDoc))
  })
  it('should redeem tickets successfully', async () => {
    const res = await request(service).post(`/api/v2/channels/${ALICE_PEER_ID.toString()}/tickets/redeem`)
    expect(res.status).to.equal(204)
    expect(res).to.satisfyApiSpec
    expect(res.body).to.be.empty
  })

  it('should fail when no tickets to redeem', async () => {
    node.getTickets = sinon.fake.returns([])
    const res = await request(service).post(`/api/v2/channels/${ALICE_PEER_ID.toString()}/tickets/redeem`)
    expect(res.status).to.equal(404)
    expect(res).to.satisfyApiSpec
    expect(res.body).to.deep.equal({ status: STATUS_CODES.TICKETS_NOT_FOUND })
  })

  it('should validate peerId', async () => {
    const res = await request(service).post(`/api/v2/channels/${INVALID_PEER_ID}/tickets/redeem`)
    expect(res.status).to.equal(400)
    expect(res).to.satisfyApiSpec
    expect(res.body).to.deep.equal({
      status: STATUS_CODES.INVALID_PEERID
    })
  })

  it('should fail when node call fails', async () => {
    node.getTickets = sinon.fake.throws('')

    const res = await request(service).post(`/api/v2/channels/${ALICE_PEER_ID.toString()}/tickets/redeem`)
    expect(res.status).to.equal(422)
    expect(res).to.satisfyApiSpec
  })
})
