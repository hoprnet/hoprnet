import request from 'supertest'
import sinon from 'sinon'
import chaiResponseValidator from 'chai-openapi-response-validator'
import chai, { expect } from 'chai'
import { createTestApiInstance, ALICE_ETHEREUM_ADDR, BOB_ETHEREUM_ADDR } from '../../../fixtures.js'
import { STATUS_CODES } from '../../../utils.js'
import { Balance, BalanceType, U256, ChannelStatus, ChannelEntry, Ticket } from '@hoprnet/hopr-utils'

let node = sinon.fake() as any
node.getTickets = sinon.fake.returns([Ticket.default()])

describe('GET /channels/{channelid}/tickets', () => {
  let service: any

  const incoming = new ChannelEntry(
    ALICE_ETHEREUM_ADDR.clone(),
    BOB_ETHEREUM_ADDR.clone(),
    new Balance('1', BalanceType.HOPR),
    U256.one(),
    ChannelStatus.Open,
    U256.one(),
    U256.one()
  )

  before(async function () {
    const loaded = await createTestApiInstance(node)

    service = loaded.service

    // @ts-ignore ESM / CommonJS compatibility issue
    chai.use(chaiResponseValidator.default(loaded.api.apiDoc))
  })

  it('should get tickets successfully', async () => {
    const res = await request(service).get(`/api/v3/channels/${incoming.get_id().to_hex()}/tickets`)
    expect(res).to.satisfyApiSpec
    expect(res.status).to.equal(200)
  })

  it('should fail when no tickets to get', async () => {
    node.getTickets = sinon.fake.returns([])
    const res = await request(service).get(`/api/v3/channels/${incoming.get_id().to_hex()}/tickets`)
    expect(res.status).to.equal(404)
    expect(res).to.satisfyApiSpec
    expect(res.body).to.deep.equal({ status: STATUS_CODES.TICKETS_NOT_FOUND })
  })

  it('should validate channel id', async () => {
    const res = await request(service).get(`/api/v3/channels/invalidchannelid/tickets`)
    expect(res.status).to.equal(400)
    expect(res).to.satisfyApiSpec
    expect(res.body).to.deep.equal({
      status: STATUS_CODES.INVALID_CHANNELID
    })
  })

  it('should fail when node call fails', async () => {
    node.getTickets = sinon.fake.throws('')

    const res = await request(service).get(`/api/v3/channels/${incoming.get_id().to_hex()}/tickets`)
    expect(res.status).to.equal(422)
    expect(res).to.satisfyApiSpec
  })
})
