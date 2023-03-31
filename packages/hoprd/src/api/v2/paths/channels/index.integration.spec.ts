import request from 'supertest'
import sinon from 'sinon'
import chaiResponseValidator from 'chai-openapi-response-validator'
import chai, { expect } from 'chai'
import {
  createTestApiInstance,
  ALICE_PEER_ID,
  BOB_PEER_ID,
  CHARLIE_PEER_ID,
  ALICE_NATIVE_ADDR,
  INVALID_PEER_ID
} from '../../fixtures.js'
import { Balance, ChannelEntry, NativeBalance, PublicKey, UINT256, Hash, ChannelStatus } from '@hoprnet/hopr-utils'
import BN from 'bn.js'
import { STATUS_CODES } from '../../utils.js'

let node = sinon.fake() as any
node.getId = sinon.fake.returns(ALICE_PEER_ID)
node.getEthereumAddress = sinon.fake.returns(ALICE_NATIVE_ADDR)
node.getNativeBalance = sinon.fake.returns(new NativeBalance(new BN(10)))
node.getBalance = sinon.fake.returns(new Balance(new BN(1)))

const CHANNEL_ID = ChannelEntry.createMock().getId()

node.openChannel = sinon.fake.returns(
  Promise.resolve({
    channelId: CHANNEL_ID,
    receipt: 'testReceipt'
  })
)

describe('GET /channels', function () {
  const incoming = new ChannelEntry(
    PublicKey.fromPeerId(ALICE_PEER_ID),
    PublicKey.fromPeerId(BOB_PEER_ID),
    new Balance(new BN(1)),
    Hash.create(),
    new UINT256(new BN(1)),
    new UINT256(new BN(1)),
    ChannelStatus.Closed,
    new UINT256(new BN(1)),
    new UINT256(new BN(1))
  )
  const outgoing = new ChannelEntry(
    PublicKey.fromPeerId(BOB_PEER_ID),
    PublicKey.fromPeerId(ALICE_PEER_ID),
    new Balance(new BN(2)),
    Hash.create(),
    new UINT256(new BN(2)),
    new UINT256(new BN(2)),
    ChannelStatus.Closed,
    new UINT256(new BN(2)),
    new UINT256(new BN(2))
  )
  const otherChannel = new ChannelEntry(
    PublicKey.fromPeerId(BOB_PEER_ID),
    PublicKey.fromPeerId(CHARLIE_PEER_ID),
    new Balance(new BN(3)),
    Hash.create(),
    new UINT256(new BN(3)),
    new UINT256(new BN(3)),
    ChannelStatus.WaitingForCommitment,
    new UINT256(new BN(3)),
    new UINT256(new BN(3))
  )
  node.getChannelsFrom = sinon.fake.returns(Promise.resolve([outgoing]))
  node.getChannelsTo = sinon.fake.returns(Promise.resolve([incoming]))
  node.getAllChannels = sinon.fake.returns(Promise.resolve([incoming, outgoing, otherChannel]))

  let service: any
  before(async function () {
    const loaded = await createTestApiInstance(node)

    service = loaded.service

    // @ts-ignore ESM / CommonJS compatibility issue
    chai.use(chaiResponseValidator.default(loaded.api.apiDoc))
  })

  it('should get channels list including closed', async function () {
    const res = await request(service).get('/api/v2/channels?includingClosed=true')
    expect(res.status).to.equal(200)
    expect(res).to.satisfyApiSpec
    expect(res.body.incoming.length).to.be.equal(1)
    expect(res.body.outgoing.length).to.be.equal(1)
    // expect(res.body.all.length).to.be.equal(0)
    expect(res.body.incoming[0].channelId).to.deep.equal(incoming.getId().toHex())
    expect(res.body.outgoing[0].channelId).to.deep.equal(outgoing.getId().toHex())
  })
  it('should get channels list excluding closed', async function () {
    const res = await request(service).get('/api/v2/channels')
    expect(res.status).to.equal(200)
    expect(res).to.satisfyApiSpec
    expect(res.body.incoming.length).to.be.equal(0)
    expect(res.body.outgoing.length).to.be.equal(0)
    // expect(res.body.all.length).to.be.equal(0)
  })
  it('should get all the channels', async function () {
    const res = await request(service).get('/api/v2/channels?fullTopology=true')
    expect(res.status).to.equal(200)
    expect(res).to.satisfyApiSpec
    expect(res.body.incoming.length).to.be.equal(0)
    expect(res.body.outgoing.length).to.be.equal(0)
    expect(res.body.all.length).to.be.equal(3)
    expect(res.body.all[0].channelId).to.deep.equal(incoming.getId().toHex())
    expect(res.body.all[1].channelId).to.deep.equal(outgoing.getId().toHex())
    expect(res.body.all[2].channelId).to.deep.equal(otherChannel.getId().toHex())
  })
})

describe('POST /channels', () => {
  let service: any
  before(async function () {
    const loaded = await createTestApiInstance(node)

    service = loaded.service
  })

  it('should open channel', async () => {
    const res = await request(service).post('/api/v2/channels').send({
      peerId: ALICE_PEER_ID.toString(),
      amount: '1'
    })
    expect(res.status).to.equal(201)
    expect(res).to.satisfyApiSpec
    expect(res.body).to.deep.equal({
      channelId: CHANNEL_ID.toHex(),
      receipt: 'testReceipt'
    })
  })

  it('should fail on invalid peerId', async () => {
    const res = await request(service).post('/api/v2/channels').send({
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
    const res = await request(service).post('/api/v2/channels').send({
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
    const res = await request(service).post('/api/v2/channels').send({
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
    node.openChannel = sinon.fake.throws('Channel is already opened')

    const res = await request(service).post('/api/v2/channels').send({
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
