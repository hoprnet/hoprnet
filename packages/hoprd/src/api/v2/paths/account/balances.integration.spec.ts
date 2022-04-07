import request from 'supertest'
import sinon from 'sinon'
import chaiResponseValidator from 'chai-openapi-response-validator'
import chai, { expect } from 'chai'
import { createTestApiInstance } from '../../fixtures'
import { Balance, NativeBalance } from '@hoprnet/hopr-utils'
import BN from 'bn.js'

let node = sinon.fake() as any
const { api, service } = createTestApiInstance(node)
chai.use(chaiResponseValidator(api.apiDoc))

describe('GET /account/balances', () => {
  it('should get balance', async () => {
    const nativeBalance = new NativeBalance(new BN(10))
    const balance = new Balance(new BN(1))
    node.getNativeBalance = sinon.fake.returns(nativeBalance)
    node.getBalance = sinon.fake.returns(balance)

    const res = await request(service).get('/api/v2/account/balances')
    expect(res.status).to.equal(200)
    expect(res).to.satisfyApiSpec
    expect(res.body).to.deep.equal({
      native: nativeBalance.toString(),
      hopr: balance.toString()
    })
  })

  it('should return 422 when either of balances node calls fail', async () => {
    node.getBalance = sinon.fake.throws('')
    node.getNativeBalance = sinon.fake.returns(new Balance(new BN(10)))
    const res = await request(service).get('/api/v2/account/balances')
    expect(res.status).to.equal(422)
    expect(res).to.satisfyApiSpec

    node.getBalance = sinon.fake.returns(new Balance(new BN(10)))
    node.getNativeBalance = sinon.fake.throws('')

    const res2 = await request(service).get('/api/v2/account/balances')
    expect(res2.status).to.equal(422)
    expect(res2).to.satisfyApiSpec
  })
})
