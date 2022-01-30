import assert from 'assert'
import sinon from 'sinon'
import { withdraw } from './withdraw'

const aliceEthWallet = '0x07c97c4f845b4698D79036239153bB381bc72ad3'

let node = sinon.fake() as any

const performTestsUsing = (currencyToTest: 'native' | 'hopr') => {
  describe('withdraw', () => {
    it('should withdraw successfuly', async () => {
      node.withdraw = sinon.fake.returns('receipt')
      const receipt = await withdraw({
        node,
        rawAmount: '0.1',
        rawCurrency: currencyToTest,
        rawRecipient: aliceEthWallet
      })
      assert.equal(receipt, 'receipt')
    })
    it('should return error when invalid amount or currency', async () => {
      node.withdraw = sinon.fake.returns('receipt')
      const err1 = (await withdraw({
        node,
        rawAmount: '0.1',
        rawCurrency: currencyToTest + 'abcd',
        rawRecipient: aliceEthWallet
      })) as Error
      assert.equal(err1.message, 'incorrectCurrency')
      const err2 = (await withdraw({
        node,
        rawAmount: '0.1abcd',
        rawCurrency: currencyToTest,
        rawRecipient: aliceEthWallet
      })) as Error
      assert.equal(err2.message, 'incorrectAmount')
    })
    it('should return error when withdrawing more than balance or address incorrect', async () => {
      node.withdraw = sinon.fake.returns('receipt')
      const err1 = (await withdraw({
        node,
        rawAmount: '10000000000',
        rawCurrency: currencyToTest + 'abcd',
        rawRecipient: aliceEthWallet
      })) as Error
      assert.equal(err1.message, 'incorrectCurrency')
    })
    it('should return error when node call fails', async () => {
      node.withdraw = sinon.fake.throws('')
      const err1 = (await withdraw({
        node,
        rawAmount: '10000000000',
        rawCurrency: currencyToTest + 'abcd',
        rawRecipient: aliceEthWallet
      })) as Error
      assert.equal(err1.message, 'failure')
    })
  })
}

performTestsUsing('native')
performTestsUsing('hopr')
