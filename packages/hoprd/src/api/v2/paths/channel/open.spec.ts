import axios from 'axios'
import assert from 'assert'

// assuming tests are running on alice node
// assuming node balance is not empty

const baseUrl = `http://localhost:${process.env.API_PORT}/api/v2`

const bobPeerId = '16Uiu2HAmRFjDov6sbcZeppbnNFFTdx5hFoBzr8csBgevtKUex8y9'
const invalidPeerId = 'definetly not a valid peerId'

describe('open', () => {
  const openChannel = (peerId: string, amount: string) => {
    return axios
      .post(`${baseUrl}/channel/open`, {
        peerId,
        amount
      })
      .catch(({ response }) => response)
  }

  it('POST - should open channel successfuly with correct parameters', async () => {
    const res1 = await openChannel(bobPeerId, '0.01')
    assert.equal(res1.status, 200)
    assert.equal(res1.data.status, 'success')
    assert('channelId' in res1.data)
  })
  it('POST - should fail on invalid peerId or amountToFund', async () => {
    const res1 = await openChannel(bobPeerId, '0.01abcd')
    assert.equal(res1.status, 400)
    assert.equal(res1.data.status, 'invalidAmountToFund')
    const res2 = await openChannel(invalidPeerId, '0.01')
    assert.equal(res2.status, 400)
    assert.equal(res2.data.status, 'invalidPeerId')
  })
  it('POST - should fail when channel is already open', async () => {
    const res1 = await openChannel(bobPeerId, '0.01')
    assert.equal(res1.status, 403)
    assert.equal(res1.data.status, 'channelAlreadyOpen')
  })
  it('POST - should fail when amount to fund is bigger than current balance', async () => {
    const tokensRequired = '1000000000000000'
    const res1 = await openChannel(bobPeerId, tokensRequired)
    assert.equal(res1.status, 500)
    assert.equal(res1.data.status, 'notEnoughFunds')
    assert('tokensRequired' in res1.data)
    assert('currentBalance' in res1.data)
  })
})
