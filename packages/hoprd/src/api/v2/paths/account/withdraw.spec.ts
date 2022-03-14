import assert from 'assert'
import BN from 'bn.js'
import sinon from 'sinon'
import { Balance, NativeBalance, PublicKey } from '@hoprnet/hopr-utils'
import { STATUS_CODES } from '../../utils'
import { withdraw } from './withdraw'
import { ALICE_PEER_ID } from '../../fixtures'

let node = sinon.fake() as any
node.withdraw = sinon.fake.returns('receipt')
node.getNativeBalance = sinon.fake.returns(Promise.resolve(new NativeBalance(new BN('10'))))
node.getBalance = sinon.fake.returns(Promise.resolve(new Balance(new BN('10'))))

describe('withdraw', () => {
  const ALICE_ETH_ADDRESS = PublicKey.fromPeerId(ALICE_PEER_ID).toAddress()

  it('should withdraw HOPR successfuly', async () => {
    const receipt = await withdraw(node, 'hopr', ALICE_ETH_ADDRESS.toString(), '1')
    assert.equal(receipt, 'receipt')
  })

  it('should withdraw NATIVE successfuly', async () => {
    const receipt = await withdraw(node, 'native', ALICE_ETH_ADDRESS.toString(), '1')
    assert.equal(receipt, 'receipt')
  })

  it('should reject on invalid arguments', async () => {
    assert.rejects(
      () => {
        return withdraw(node, 'invalidCurrency' as unknown as 'native', ALICE_ETH_ADDRESS.toString(), '1')
      },
      (err: Error) => {
        return err.message.includes(STATUS_CODES.INVALID_CURRENCY)
      }
    )
    assert.rejects(
      () => {
        return withdraw(node, 'hopr', '0x00', '1')
      },
      (err: Error) => {
        return err.message.includes(STATUS_CODES.INVALID_ADDRESS)
      }
    )
    assert.rejects(
      () => {
        return withdraw(node, 'native', ALICE_ETH_ADDRESS.toString(), 'abc')
      },
      (err: Error) => {
        return err.message.includes(STATUS_CODES.INVALID_AMOUNT)
      }
    )
  })

  it('should return error when withdrawing more than balance or address incorrect', async () => {
    assert.rejects(
      () => {
        return withdraw(node, 'native', ALICE_ETH_ADDRESS.toString(), '100')
      },
      (err: Error) => {
        return err.message.includes(STATUS_CODES.NOT_ENOUGH_BALANCE)
      }
    )
  })
})
