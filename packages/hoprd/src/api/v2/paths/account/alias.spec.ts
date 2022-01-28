import axios from 'axios'
import assert from 'assert'

// assuming tests are running on alice node
const baseUrl = `http://localhost:${process.env.API_PORT}/api/v2`

const bobPeerId = '16Uiu2HAmRFjDov6sbcZeppbnNFFTdx5hFoBzr8csBgevtKUex8y9'
const alicePeerId = '16Uiu2HAmUsJwbECMroQUC29LQZZWsYpYZx1oaM1H9DBoZHLkYn12'
const invalidPeerId = 'definetly not a valid peerId'

describe('alias', () => {
  const testAliases = ['testAlias1', 'testAlias2']

  const setAlias = (alias: string, peerId: string) => {
    return axios
      .post(`${baseUrl}/account/alias`, {
        alias,
        peerId
      })
      .catch(({ response }) => response)
  }
  const getAlias = (peerId: string) => {
    return axios.get(`${baseUrl}/account/alias?peerId=${peerId}`).catch(({ response }) => response)
  }

  it('POST - should set alias successfuly with correct parameters', async () => {
    const res1 = await setAlias(testAliases[0], bobPeerId)
    assert.equal(res1.status, 200)
    assert.deepEqual(res1.data, { status: 'success' })
    const res2 = await setAlias(testAliases[1], bobPeerId)
    assert.equal(res2.status, 200)
    assert.deepEqual(res2.data, { status: 'success' })
  })
  it('POST - should fail on invalid peerId', async () => {
    const { status, data } = await setAlias('testAlias', invalidPeerId)
    assert.equal(status, 400)
    assert.deepEqual(data, { status: 'invalidPeerId' })
  })
  it('POST - should fail on incomplete body', async () => {
    const res1 = await setAlias('', bobPeerId)
    assert.equal(res1.status, 400)
    assert.deepEqual(res1.data, { status: 'missingBodyfields' })
    const res2 = await setAlias('testAlias', '')
    assert.equal(res2.status, 400)
    assert.deepEqual(res2.data, { status: 'missingBodyfields' })
  })

  it('GET - should successfuly fetch aliases', async () => {
    const { status, data } = await getAlias(bobPeerId)
    assert.equal(status, 200)
    assert.deepEqual(data, { status: 'success', aliases: testAliases })
  })
  it('GET - should fail on missing peerId', async () => {
    const { status, data } = await getAlias('')
    assert.equal(status, 400)
    assert.deepEqual(data, { status: 'noPeerIdProvided' })
  })
  it('GET - should fail when no alias found', async () => {
    const { status, data } = await getAlias(alicePeerId)
    assert.equal(status, 404)
    assert.deepEqual(data, { status: 'aliasNotFound' })
  })
})
