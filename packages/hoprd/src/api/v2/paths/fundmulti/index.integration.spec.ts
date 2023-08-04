import request from 'supertest'
import chaiResponseValidator from 'chai-openapi-response-validator'
import chai, { expect } from 'chai'
import {
  createTestApiInstance,
  ALICE_PEER_ID,
  BOB_PEER_ID,
  ALICE_NATIVE_ADDR,
  INVALID_PEER_ID
} from '../../fixtures.js'
import { Balance, BalanceType } from '@hoprnet/hopr-utils'
import { STATUS_CODES } from '../../utils.js'

let node = {} as any
node.getId = () => ALICE_PEER_ID
node.getEthereumAddress = () => ALICE_NATIVE_ADDR
node.getNativeBalance = () => new Balance('10', BalanceType.Native)
node.getBalance = () => new Balance('5', BalanceType.HOPR)

node.fundChannel = async () => 'testReceipt'

describe('POST /fundmulti', () => {
  let service: any
  before(async function () {
    const loaded = await createTestApiInstance(node)

    service = loaded.service
    // @ts-ignore ESM / CommonJS compatibility issue
    chai.use(chaiResponseValidator.default(loaded.api.apiDoc))
  })

  it('should fund two channels', async () => {
    const res = await request(service).post('/api/v2/fundmulti').send({
      peerId: BOB_PEER_ID.toString(),
      outgoingAmount: '3',
      incomingAmount: '2'
    })
    expect(res).to.satisfyApiSpec
    expect(res.body).to.deep.equal({
      receipt: 'testReceipt'
    })
    expect(res.status).to.equal(201)
  })

  it('should fail on invalid peerId', async () => {
    const res = await request(service).post('/api/v2/fundmulti').send({
      peerId: INVALID_PEER_ID,
      outgoingAmount: '3',
      incomingAmount: '2'
    })
    expect(res.status).to.equal(400)
    expect(res).to.satisfyApiSpec
    expect(res.body).to.deep.equal({
      status: STATUS_CODES.INVALID_PEERID
    })
  })

  it('should fail on invalid amount', async () => {
    const res = await request(service).post('/api/v2/fundmulti').send({
      peerId: BOB_PEER_ID.toString(),
      outgoingAmount: '3',
      incomingAmount: 'abc'
    })
    expect(res.status).to.equal(400)
    expect(res).to.satisfyApiSpec
    expect(res.body).to.deep.equal({
      status: STATUS_CODES.INVALID_AMOUNT
    })
  })

  it('should fail when out of balance', async () => {
    const res = await request(service).post('/api/v2/fundmulti').send({
      peerId: BOB_PEER_ID.toString(),
      outgoingAmount: '8',
      incomingAmount: '3'
    })
    expect(res.status).to.equal(403)
    expect(res).to.satisfyApiSpec
    expect(res.body).to.deep.equal({
      status: STATUS_CODES.NOT_ENOUGH_BALANCE
    })
  })
})
