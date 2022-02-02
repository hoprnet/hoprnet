import { Balance, NativeBalance } from '@hoprnet/hopr-utils'
import assert from 'assert'
import BN from 'bn.js'
import sinon from 'sinon'
import { getBalances } from './balance'

let node = sinon.fake() as any

describe('getBalances', () => {
  it('should get balance', async () => {
    node.getNativeBalance = sinon.fake.returns(new NativeBalance(new BN(10)))
    node.getBalance = sinon.fake.returns(new Balance(new BN(1)))

    const { native, hopr } = await getBalances(node)
    assert.equal(native.toBN().toString(), '10')
    assert.equal(hopr.toBN().toString(), '1')
  })

  it('should throw error when either of balances node calls fail', async () => {
    node.getBalance = sinon.fake.throws('')
    node.getNativeBalance = sinon.fake.returns(new Balance(new BN(10)))

    try {
      await getBalances(node)
      assert.fail('Expected failure did not throw')
    } catch (err) {
      assert.throws(
        () => {
          throw err
        },
        Error,
        'failure'
      )
    }
    node.getBalance = sinon.fake.returns(new Balance(new BN(10)))
    node.getNativeBalance = sinon.fake.throws('')

    try {
      await getBalances(node)
      assert.fail('Expected failure did not throw')
    } catch (err) {
      assert.throws(
        () => {
          throw err
        },
        Error,
        'failure'
      )
    }
  })
})
