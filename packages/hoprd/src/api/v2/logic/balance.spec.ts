import { Balance, NativeBalance } from '@hoprnet/hopr-utils'
import assert from 'assert'
import BN from 'bn.js'
import sinon from 'sinon'
import { Balances, getBalances } from './balance'

let node = sinon.fake() as any

describe('getBalances', () => {
  it('should get balance', async () => {
    node.getBalance = sinon.fake.returns(new NativeBalance(new BN(1)))
    node.getNative = sinon.fake.returns(new Balance(new BN(10)))

    const { native, hopr } = (await getBalances(node)) as Balances
    assert.equal(native, '1')
    assert.equal(hopr, '10')
  })

  it('should return error when either of balances node calls fail', async () => {
    node.getBalance = sinon.fake.throws('')
    node.getNative = sinon.fake.returns(new Balance(new BN(10)))
    const err = (await getBalances(node)) as Error
    assert.equal(err.message, 'failure')
    node.getBalance = sinon.fake.returns(new Balance(new BN(10)))
    node.getNative = sinon.fake.throws('')
    const err2 = (await getBalances(node)) as Error
    assert.equal(err2.message, 'failure')
  })
})
