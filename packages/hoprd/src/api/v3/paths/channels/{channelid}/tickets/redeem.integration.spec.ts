import request from 'supertest'
import sinon from 'sinon'
import chaiResponseValidator from 'chai-openapi-response-validator'
import chai, { expect } from 'chai'
import {
  createTestApiInstance,
  ALICE_ETHEREUM_ADDR,
  BOB_ETHEREUM_ADDR,
  CHARLIE_ETHEREUM_ADDR
} from '../../../../fixtures.js'
import { Balance, BalanceType, U256, ChannelStatus, ChannelEntry } from '@hoprnet/hopr-utils'

import { STATUS_CODES } from '../../../../utils.js'

let node = sinon.fake() as any
node.getTickets = sinon.fake.returns(['ticket'])
node.redeemTicketsInChannel = sinon.fake()

describe('POST /channels/{channelid}/tickets/redeem', () => {
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
  const outgoing = new ChannelEntry(
    BOB_ETHEREUM_ADDR.clone(),
    ALICE_ETHEREUM_ADDR.clone(),
    new Balance('2', BalanceType.HOPR),
    new U256('2'),
    ChannelStatus.Closed,
    new U256('2'),
    new U256('2')
  )
  const otherChannel = new ChannelEntry(
    BOB_ETHEREUM_ADDR.clone(),
    CHARLIE_ETHEREUM_ADDR.clone(),
    new Balance('3', BalanceType.HOPR),
    new U256('3'),
    ChannelStatus.Open,
    new U256('3'),
    new U256('3')
  )
  node.getChannelsFrom = async () => [outgoing]
  node.getChannelsTo = async () => [incoming]
  node.getAllChannels = async () => [incoming, outgoing, otherChannel]

  before(async function () {
    const loaded = await createTestApiInstance(node)

    service = loaded.service

    // @ts-ignore ESM / CommonJS compatibility issue
    chai.use(chaiResponseValidator.default(loaded.api.apiDoc))
  })

  it('should redeem tickets successfully', async () => {
    const res = await request(service).post(`/api/v3/channels/${incoming.get_id().to_hex()}/tickets/redeem`)
    expect(res.status).to.equal(204)
    expect(res).to.satisfyApiSpec
    expect(res.body).to.be.empty
  })

  it('should fail when no tickets to redeem', async () => {
    node.getTickets = sinon.fake.returns([])
    const res = await request(service).post(`/api/v3/channels/${incoming.get_id().to_hex()}/tickets/redeem`)
    expect(res.status).to.equal(404)
    expect(res).to.satisfyApiSpec
    expect(res.body).to.deep.equal({ status: STATUS_CODES.TICKETS_NOT_FOUND })
  })

  it('should validate channelid', async () => {
    const res = await request(service).post(`/api/v3/channels/invalidchannelid/tickets/redeem`)
    expect(res.status).to.equal(400)
    expect(res).to.satisfyApiSpec
    expect(res.body).to.deep.equal({
      status: STATUS_CODES.INVALID_CHANNELID
    })
  })

  it('should fail when node call fails', async () => {
    node.getTickets = sinon.fake.throws('')

    const res = await request(service).post(`/api/v3/channels/${incoming.get_id().to_hex()}/tickets/redeem`)
    expect(res.status).to.equal(422)
    expect(res).to.satisfyApiSpec
  })
})
