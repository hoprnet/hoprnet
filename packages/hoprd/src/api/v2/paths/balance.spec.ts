import axios from 'axios'
import assert from 'assert'

// assuming tests are running on alice node
// assuming node balance is not empty

const baseUrl = `http://localhost:${process.env.API_PORT}/api/v2`

describe('balance', () => {
  const getBalance = () => {
    return axios.get(`${baseUrl}/balance`).catch(({ response }) => response)
  }

  it('POST - should fetch balance successfuly', async () => {
    const res1 = await getBalance()
    assert.equal(res1.status, 200)
    assert.equal(res1.data.status, 'success')
    assert('balances' in res1.data)
    assert('native' in res1.data.balances)
    assert('hopr' in res1.data.balances)
  })
})
