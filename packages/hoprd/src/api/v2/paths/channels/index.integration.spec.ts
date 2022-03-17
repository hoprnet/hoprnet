import request from 'supertest'
import sinon from 'sinon'
import chaiResponseValidator from 'chai-openapi-response-validator'
import chai, { expect } from 'chai'
import { createTestApiInstance, ALICE_PEER_ID, INVALID_PEER_ID } from '../../fixtures'
import { Balance, ChannelEntry, NativeBalance } from '@hoprnet/hopr-utils'
import BN from 'bn.js'
import { STATUS_CODES } from '../../utils'

let node = sinon.fake() as any
node.getId = sinon.fake.returns(ALICE_PEER_ID)

const { api, service } = createTestApiInstance(node)
chai.use(chaiResponseValidator(api.apiDoc))

const CHANNEL_ID = ChannelEntry.createMock().getId()

describe('GET /channels', function () {
  const testChannel = ChannelEntry.createMock()
  node.getChannelsFrom = sinon.fake.returns(Promise.resolve([testChannel]))
  node.getChannelsTo = sinon.fake.returns(Promise.resolve([testChannel]))

  it('should get channels list including closed', async function () {
    const res = await request(service).get('/api/v2/channels?includingClosed=true')
    expect(res.status).to.equal(200)
    expect(res).to.satisfyApiSpec
    expect(res.body.incoming.length).to.be.equal(1)
    expect(res.body.outgoing.length).to.be.equal(1)
  })
  it('should get channels list excluding closed', async function () {
    const res = await request(service).get('/api/v2/channels')
    expect(res.status).to.equal(200)
    expect(res).to.satisfyApiSpec
    expect(res.body.incoming.length).to.be.equal(0)
    expect(res.body.outgoing.length).to.be.equal(0)
  })
})

node.getNativeBalance = sinon.fake.returns(new NativeBalance(new BN(10)))
node.getBalance = sinon.fake.returns(new Balance(new BN(1)))
node.openChannel = sinon.fake.returns(
  Promise.resolve({
    channelId: CHANNEL_ID,
    receipt: 'testReceipt'
  })
)

describe('POST /channels', () => {
  it('should open channel', async () => {
    const res = await request(service).post('/api/v2/channels').send({
      peerId: ALICE_PEER_ID.toB58String(),
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
      peerId: ALICE_PEER_ID.toB58String(),
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
      peerId: ALICE_PEER_ID.toB58String(),
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
      peerId: ALICE_PEER_ID.toB58String(),
      amount: '1'
    })
    expect(res.status).to.equal(403)
    expect(res).to.satisfyApiSpec
    expect(res.body).to.deep.equal({
      status: STATUS_CODES.CHANNEL_ALREADY_OPEN
    })
  })
})
