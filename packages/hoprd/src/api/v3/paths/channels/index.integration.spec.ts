import request from 'supertest'
import sinon from 'sinon'
import chaiResponseValidator from 'chai-openapi-response-validator'
import chai, { expect } from 'chai'

import { Balance, ChannelEntry, BalanceType, U256, ChannelStatus } from '@hoprnet/hopr-utils'

import {
  createTestApiInstance,
  ALICE_PEER_ID,
  ALICE_ETHEREUM_ADDR,
  ALICE_ACCOUNT_ENTRY,
  BOB_ETHEREUM_ADDR,
  CHARLIE_ETHEREUM_ADDR,
  channelEntryCreateMock
} from '../../fixtures.js'
import { STATUS_CODES } from '../../utils.js'

let node = sinon.fake as any
node.getId = sinon.fake.returns(ALICE_PEER_ID)
node.getEthereumAddress = sinon.fake.returns(ALICE_ETHEREUM_ADDR.clone())
node.getSafeBalance = sinon.fake.resolves(new Balance('10', BalanceType.HOPR))

const CHANNEL_ID = channelEntryCreateMock().get_id()

node.openChannel = sinon.fake.resolves({
  channelId: CHANNEL_ID,
  receipt: 'testReceipt'
})

describe('GET /channels', function () {
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
  node.getChannelsFrom = sinon.fake.resolves([outgoing])
  node.getChannelsTo = sinon.fake.resolves([incoming])
  node.getAllChannels = sinon.fake.resolves([incoming, outgoing, otherChannel])
  node.db = sinon.fake()
  node.db.get_account = sinon.fake.resolves(ALICE_ACCOUNT_ENTRY)

  let service: any

  before(async function () {
    const loaded = await createTestApiInstance(node)

    service = loaded.service

    // @ts-ignore ESM / CommonJS compatibility issue
    chai.use(chaiResponseValidator.default(loaded.api.apiDoc))
  })

  it('should get channels list including closed', async function () {
    const res = await request(service).get('/api/v3/channels?includingClosed=true')
    expect(res.status).to.equal(200)
    expect(res).to.satisfyApiSpec
    expect(res.body.incoming.length).to.be.equal(1)
    expect(res.body.outgoing.length).to.be.equal(1)
    // expect(res.body.all.length).to.be.equal(0)
    expect(res.body.incoming[0].id).to.deep.equal(incoming.get_id().to_hex())
    expect(res.body.outgoing[0].id).to.deep.equal(outgoing.get_id().to_hex())
  })

  it('should get channels list excluding closed', async function () {
    const res = await request(service).get('/api/v3/channels')
    expect(res.status).to.equal(200)
    expect(res).to.satisfyApiSpec
    expect(res.body.incoming.length).to.be.equal(0)
    expect(res.body.outgoing.length).to.be.equal(0)
    // expect(res.body.all.length).to.be.equal(0)
  })

  it('should get all the channels', async function () {
    const res = await request(service).get('/api/v3/channels?fullTopology=true')
    expect(res.status).to.equal(200)
    expect(res).to.satisfyApiSpec
    expect(res.body.incoming.length).to.be.equal(0)
    expect(res.body.outgoing.length).to.be.equal(0)
    expect(res.body.all.length).to.be.equal(3)
    expect(res.body.all[0].channelId).to.deep.equal(incoming.get_id().to_hex())
    expect(res.body.all[1].channelId).to.deep.equal(outgoing.get_id().to_hex())
    expect(res.body.all[2].channelId).to.deep.equal(otherChannel.get_id().to_hex())
  })
})

describe('POST /channels', () => {
  let service: any
  before(async function () {
    const loaded = await createTestApiInstance(node)

    service = loaded.service

    // @ts-ignore ESM / CommonJS compatibility issue
    chai.use(chaiResponseValidator.default(loaded.api.apiDoc))
  })

  it('should open channel', async () => {
    const res = await request(service).post('/api/v3/channels').send({
      peerAddress: ALICE_ETHEREUM_ADDR.to_string(),
      amount: '1'
    })
    expect(res.status).to.equal(201)
    expect(res).to.satisfyApiSpec
    expect(res.body).to.deep.equal({
      channelId: CHANNEL_ID.to_hex(),
      transactionReceipt: 'testReceipt'
    })
  })

  it('should fail on invalid counterparty address', async () => {
    const res = await request(service).post('/api/v3/channels').send({
      peerAddress: 'invalid address',
      amount: '1'
    })
    expect(res.status).to.equal(400)
    expect(res).to.satisfyApiSpec
    expect(res.body).to.deep.equal({
      status: STATUS_CODES.INVALID_ADDRESS
    })
  })

  it('should fail on invalid amountToFund', async () => {
    const res = await request(service).post('/api/v3/channels').send({
      peerAddress: ALICE_ETHEREUM_ADDR.to_string(),
      amount: 'abc'
    })
    expect(res.status).to.equal(400)
    expect(res).to.satisfyApiSpec
    expect(res.body).to.deep.equal({
      status: STATUS_CODES.INVALID_AMOUNT
    })
  })

  it('should fail when out of balance', async () => {
    const res = await request(service).post('/api/v3/channels').send({
      peerAddress: ALICE_ETHEREUM_ADDR.to_string(),
      amount: '10000000'
    })
    expect(res.status).to.equal(403)
    expect(res).to.satisfyApiSpec
    expect(res.body).to.deep.equal({
      status: STATUS_CODES.NOT_ENOUGH_BALANCE
    })
  })

  it('should fail when channel is already open', async () => {
    node.openChannel = () => {
      throw Error('Channel is already opened')
    }

    const res = await request(service).post('/api/v3/channels').send({
      peerAddress: ALICE_ETHEREUM_ADDR.to_string(),
      amount: '1'
    })
    expect(res.status).to.equal(409)
    expect(res).to.satisfyApiSpec
    expect(res.body).to.deep.equal({
      status: STATUS_CODES.CHANNEL_ALREADY_OPEN
    })
  })
})
