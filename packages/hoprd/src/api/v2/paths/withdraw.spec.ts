import axios from 'axios'
import assert from 'assert'

// assuming tests are running on alice node
// assuming node balance is not empty

const baseUrl = `http://localhost:${process.env.API_PORT}/api/v2`

const aliceEthWallet = '0x07c97c4f845b4698D79036239153bB381bc72ad3'

const performTestsUsing = (currencyToTest: 'native' | 'hopr') => {
  describe('withdraw', () => {
    const withdraw = (currency: string, amount: string, recipient) => {
      return axios
        .post(`${baseUrl}/withdraw`, {
          currency,
          amount,
          recipient
        })
        .catch(({ response }) => response)
    }

    it('POST - should withdraw successfuly with correct parameters', async () => {
      const res1 = await withdraw(currencyToTest, '0.001', aliceEthWallet)
      assert.equal(res1.status, 200)
      assert.equal(res1.data.status, 'success')
      assert('receipt' in res1.data)
    }).timeout(2000)
    it('POST - should fail when invalid amount or currency', async () => {
      const res1 = await withdraw(currencyToTest, '0.001abcd', aliceEthWallet)
      assert.equal(res1.status, 400)
      assert.equal(res1.data.status, 'incorrectAmount')
      const res2 = await withdraw(currencyToTest + 'abcd', '0.001', aliceEthWallet)
      assert.equal(res2.status, 400)
      assert.equal(res2.data.status, 'incorrectCurrency')
    })
    it('POST - should fail when withdrawing more than balance or address incorrect', async () => {
      const res1 = await withdraw(currencyToTest, '100000000000000', aliceEthWallet)
      assert.equal(res1.status, 500)
      assert.equal(res1.data.status, 'failure')
      const res2 = await withdraw(currencyToTest, '0.001', aliceEthWallet + 'abcd')
      assert.equal(res2.status, 500)
      assert.equal(res2.data.status, 'failure')
    }).timeout(2000)
  })
}

performTestsUsing('native')
performTestsUsing('hopr')
