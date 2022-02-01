import assert from 'assert'
import BN from 'bn.js'
import sinon from 'sinon'
import { Balance, NativeBalance } from '@hoprnet/hopr-utils'
import { STATUS_CODES } from '../'
import { withdraw } from './withdraw'
import { testEthAddress } from '../fixtures'

let node = sinon.fake() as any
node.withdraw = sinon.fake.returns('receipt')
node.getNativeBalance = sinon.fake.returns(Promise.resolve(new NativeBalance(new BN('10'))))
node.getBalance = sinon.fake.returns(Promise.resolve(new Balance(new BN('10'))))

describe('withdraw', () => {
  it('should withdraw HOPR successfuly', async () => {
    const receipt = await withdraw(node, 'HOPR', testEthAddress, '1')
    assert.equal(receipt, 'receipt')
  })

  it('should withdraw NATIVE successfuly', async () => {
    const receipt = await withdraw(node, 'NATIVE', testEthAddress, '1')
    assert.equal(receipt, 'receipt')
  })

  it('should reject on invalid arguments', async () => {
    assert.rejects(
      () => {
        return withdraw(node, 'invalidCurrency' as unknown as 'NATIVE', testEthAddress, '1')
      },
      (err: Error) => {
        return err.message.includes(STATUS_CODES.INVALID_CURRENCY)
      }
    )
    assert.rejects(
      () => {
        return withdraw(node, 'HOPR', '0x00', '1')
      },
      (err: Error) => {
        return err.message.includes(STATUS_CODES.INVALID_ADDRESS)
      }
    )
    assert.rejects(
      () => {
        return withdraw(node, 'NATIVE', testEthAddress, 'abc')
      },
      (err: Error) => {
        return err.message.includes(STATUS_CODES.INVALID_AMOUNT)
      }
    )
  })

  it('should return error when withdrawing more than balance or address incorrect', async () => {
    assert.rejects(
      () => {
        return withdraw(node, 'NATIVE', testEthAddress, '100')
      },
      (err: Error) => {
        return err.message.includes(STATUS_CODES.NOT_ENOUGH_BALANCE)
      }
    )
  })
})
