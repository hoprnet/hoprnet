import request from 'supertest'
import sinon from 'sinon'
import chaiResponseValidator from 'chai-openapi-response-validator'
import chai, { expect } from 'chai'
import { createTestApiInstance, ALICE_PEER_ID } from '../../fixtures'
import { Balance, NativeBalance, PublicKey } from '@hoprnet/hopr-utils'
import BN from 'bn.js'
import { STATUS_CODES } from '../../utils'

let node = sinon.fake() as any
node.withdraw = sinon.fake.returns('receipt')
node.getNativeBalance = sinon.fake.returns(Promise.resolve(new NativeBalance(new BN('10'))))
node.getBalance = sinon.fake.returns(Promise.resolve(new Balance(new BN('10'))))

const { api, service } = createTestApiInstance(node)
chai.use(chaiResponseValidator(api.apiDoc))

describe('POST /account/withdraw', () => {
  const ALICE_ETH_ADDRESS = PublicKey.fromPeerId(ALICE_PEER_ID).toAddress()

  it('should withdraw NATIVE successfuly', async () => {
    const res = await request(service).post('/api/v2/account/withdraw').send({
      currency: 'NATIVE',
      amount: '1',
      recipient: ALICE_ETH_ADDRESS.toString()
    })
    expect(res.status).to.equal(200)
    expect(res).to.satisfyApiSpec
    expect(res.body).to.deep.equal({
      receipt: 'receipt'
    })
  })
  it('should withdraw HOPR successfuly', async () => {
    const res = await request(service).post('/api/v2/account/withdraw').send({
      currency: 'HOPR',
      amount: '1',
      recipient: ALICE_ETH_ADDRESS.toString()
    })
    expect(res.status).to.equal(200)
    expect(res).to.satisfyApiSpec
    expect(res.body).to.deep.equal({
      receipt: 'receipt'
    })
  })
  it('should return 400 on incorrect currency in body', async () => {
    const res = await request(service).post('/api/v2/account/withdraw').send({
      currency: 'invalidCurrency',
      amount: '1',
      recipient: ALICE_ETH_ADDRESS.toString()
    })
    expect(res.status).to.equal(400)
    expect(res).to.satisfyApiSpec
    expect(res.body).to.deep.equal({
      status: STATUS_CODES.INVALID_CURRENCY
    })
  })
  it('should return 400 on incorrect amount in body', async () => {
    const res = await request(service).post('/api/v2/account/withdraw').send({
      currency: 'NATIVE',
      amount: 'invalidAmount',
      recipient: ALICE_ETH_ADDRESS.toString()
    })
    expect(res.status).to.equal(400)
    expect(res).to.satisfyApiSpec
    expect(res.body).to.deep.equal({
      status: STATUS_CODES.INVALID_AMOUNT
    })
  })
  it('should return 400 on incorrect address in body', async () => {
    const res = await request(service).post('/api/v2/account/withdraw').send({
      currency: 'NATIVE',
      amount: '1',
      recipient: 'invalidAddress'
    })
    expect(res.status).to.equal(400)
    expect(res).to.satisfyApiSpec
    expect(res.body).to.deep.equal({
      status: STATUS_CODES.INVALID_ADDRESS
    })
  })

  it('should return 422 when withdrawing more than current balance', async () => {
    const res = await request(service).post('/api/v2/account/withdraw').send({
      currency: 'NATIVE',
      amount: '100000000000000000000000000000000000000000000000000000000000000',
      recipient: ALICE_ETH_ADDRESS.toString()
    })
    expect(res.status).to.equal(422)
    expect(res).to.satisfyApiSpec
    expect(res.body).to.deep.equal({
      status: STATUS_CODES.NOT_ENOUGH_BALANCE,
      error: STATUS_CODES.NOT_ENOUGH_BALANCE
    })
  })
})
