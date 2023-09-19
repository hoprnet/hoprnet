import sinon from 'sinon'
import { ALICE_ETHEREUM_ADDR, BOB_ETHEREUM_ADDR, createTestApiInstance } from '../../../../fixtures.js'
import chai, { expect } from 'chai'
import chaiResponseValidator from 'chai-openapi-response-validator'
import request from 'supertest'
import { STATUS_CODES } from '../../../../utils.js'
import { Balance, BalanceType, ChannelEntry, ChannelStatus, U256 } from '../../../../../../../lib/hoprd_hoprd.js'

let node = sinon.fake() as any
node.getTickets = sinon.fake.returns(['ticket'])
node.aggregateTickets = sinon.fake()

describe('POST /channels/{channelid}/tickets/aggregate', () => {
  let service: any

  const incoming = new ChannelEntry(
    ALICE_ETHEREUM_ADDR.clone(),
    BOB_ETHEREUM_ADDR.clone(),
    new Balance('1', BalanceType.HOPR),
    U256.one(),
    ChannelStatus.Closed,
    U256.one(),
    U256.one()
  )

  before(async function () {
    const loaded = await createTestApiInstance(node)

    service = loaded.service

    // @ts-ignore ESM / CommonJS compatibility issue
    chai.use(chaiResponseValidator.default(loaded.api.apiDoc))
  })

  it('should aggregate tickets successfully', async () => {
    const res = await request(service).post(`/api/v3/channels/${incoming.get_id().to_hex()}/tickets/aggregate`)
    expect(res.status).to.equal(204)
    expect(res).to.satisfyApiSpec
    expect(res.body).to.be.empty
  })

  it('should fail when no tickets to aggregate', async () => {
    node.getTickets = sinon.fake.returns([])
    const res = await request(service).post(`/api/v3/channels/${incoming.get_id().to_hex()}/tickets/aggregate`)
    expect(res.status).to.equal(404)
    expect(res).to.satisfyApiSpec
    expect(res.body).to.deep.equal({ status: STATUS_CODES.TICKETS_NOT_FOUND })
  })

  it('should validate channelid', async () => {
    const res = await request(service).post(`/api/v3/channels/invalidchannelid/tickets/aggregate`)
    expect(res.status).to.equal(400)
    expect(res).to.satisfyApiSpec
    expect(res.body).to.deep.equal({
      status: STATUS_CODES.INVALID_CHANNELID
    })
  })

  it('should fail when node call fails', async () => {
    node.getTickets = sinon.fake.returns(['ticket'])
    node.aggregateTickets = sinon.fake.throws('')

    const res = await request(service).post(`/api/v3/channels/${incoming.get_id().to_hex()}/tickets/aggregate`)
    expect(res.status).to.equal(422)
    expect(res).to.satisfyApiSpec
  })
})
