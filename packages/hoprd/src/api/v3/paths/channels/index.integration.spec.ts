import request from 'supertest'
import chaiResponseValidator from 'chai-openapi-response-validator'
import chai, { expect } from 'chai'
import {
  createTestApiInstance,
  ALICE_PEER_ID,
  BOB_PEER_ID,
  CHARLIE_PEER_ID,
  ALICE_NATIVE_ADDR,
  INVALID_PEER_ID,
  channelEntryCreateMock
} from '../../fixtures.js'
import { Balance, ChannelEntry, BalanceType, PublicKey, U256, ChannelStatus } from '@hoprnet/hopr-utils'

import { STATUS_CODES } from '../../utils.js'

let node = {} as any
node.getId = () => ALICE_PEER_ID
node.getEthereumAddress = () => ALICE_NATIVE_ADDR
node.getNativeBalance = () => new Balance('10', BalanceType.Native)
node.getBalance = () => new Balance('1', BalanceType.HOPR)

const CHANNEL_ID = channelEntryCreateMock().get_id()

node.openChannel = async () => ({
  channelId: CHANNEL_ID,
  receipt: 'testReceipt'
})

describe('GET /channels', function () {
  const incoming = new ChannelEntry(
    PublicKey.from_peerid_str(ALICE_PEER_ID.toString()).to_address(),
    PublicKey.from_peerid_str(BOB_PEER_ID.toString()).to_address(),
    new Balance('1', BalanceType.HOPR),
    U256.one(),
    ChannelStatus.Closed,
    U256.one(),
    U256.one()
  )
  const outgoing = new ChannelEntry(
    PublicKey.from_peerid_str(BOB_PEER_ID.toString()).to_address(),
    PublicKey.from_peerid_str(ALICE_PEER_ID.toString()).to_address(),
    new Balance('2', BalanceType.HOPR),
    new U256('2'),
    ChannelStatus.Closed,
    new U256('2'),
    new U256('2')
  )
  const otherChannel = new ChannelEntry(
    PublicKey.from_peerid_str(BOB_PEER_ID.toString()).to_address(),
    PublicKey.from_peerid_str(CHARLIE_PEER_ID.toString()).to_address(),
    new Balance('3', BalanceType.HOPR),
    new U256('3'),
    ChannelStatus.Open,
    new U256('3'),
    new U256('3')
  )
  node.getChannelsFrom = async () => [outgoing]
  node.getChannelsTo = async () => [incoming]
  node.getAllChannels = async () => [incoming, outgoing, otherChannel]

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
    expect(res.body.incoming[0].channelId).to.deep.equal(incoming.get_id().to_hex())
    expect(res.body.outgoing[0].channelId).to.deep.equal(outgoing.get_id().to_hex())
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
  })

  it('should open channel', async () => {
    const res = await request(service).post('/api/v3/channels').send({
      peerId: ALICE_PEER_ID.toString(),
      amount: '1'
    })
    expect(res.status).to.equal(201)
    expect(res).to.satisfyApiSpec
    expect(res.body).to.deep.equal({
      channelId: CHANNEL_ID.to_hex(),
      receipt: 'testReceipt'
    })
  })

  it('should fail on invalid peerId', async () => {
    const res = await request(service).post('/api/v3/channels').send({
      peerId: INVALID_PEER_ID,
      amount: '1'
    })
    expect(res.status).to.equal(400)
    expect(res).to.satisfyApiSpec
    expect(res.body).to.deep.equal({
      status: STATUS_CODES.INVALID_PEERID
    })
  })

  it('should fail on invalid amountToFund', async () => {
    const res = await request(service).post('/api/v3/channels').send({
      peerId: ALICE_PEER_ID.toString(),
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
      peerId: ALICE_PEER_ID.toString(),
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
      peerId: ALICE_PEER_ID.toString(),
      amount: '1'
    })
    expect(res.status).to.equal(409)
    expect(res).to.satisfyApiSpec
    expect(res.body).to.deep.equal({
      status: STATUS_CODES.CHANNEL_ALREADY_OPEN
    })
  })
})
