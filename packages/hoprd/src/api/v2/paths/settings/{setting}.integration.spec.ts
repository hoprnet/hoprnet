import request from 'supertest'
import sinon from 'sinon'
import chaiResponseValidator from 'chai-openapi-response-validator'
import chai, { expect } from 'chai'
import { createTestApiInstance, invalidTestPeerId, testPeerId } from '../../fixtures'
import { STATUS_CODES } from '../../utils'

let node = sinon.fake() as any
const { api, service } = createTestApiInstance(node)
chai.use(chaiResponseValidator(api.apiDoc))

describe('PUT /settings/{setting}', () => {
  it('should set setting successfuly', async () => {
    const res = await request(service)
      .post(`/api/v2/settings/includeRecipient`)
      .send({ key: 'includeRecipient', value: true })
    expect(res.status).to.equal(200)
    expect(res).to.satisfyApiSpec
    expect(res.body).to.be.empty
  })

  it('should return error when invalid setting key is provided', () => {
    it('should set setting successfuly', async () => {
      const res = await request(service).post(`/api/v2/settings/invalidKey`).send({ key: 'invalidKey', value: true })
      expect(res.status).to.equal(400)
      expect(res).to.satisfyApiSpec
      expect(res.body).to.deep.equal({ status: STATUS_CODES.INVALID_SETTING })
    })
  })

  // it('should throw error when invalid value provided ', () => {
  //   const stateOps = createTestMocks()

  //   assert.throws(
  //     () => setSetting(node, stateOps, 'includeRecipient', 'true'),
  //     (err: Error) => {
  //       return err.message.includes(STATUS_CODES.INVALID_SETTING_VALUE)
  //     }
  //   )
  //   assert.throws(
  //     () => setSetting(node, stateOps, 'strategy', 'abcd'),
  //     (err: Error) => {
  //       return err.message.includes(STATUS_CODES.INVALID_SETTING_VALUE)
  //     }
  //   )
  // })

  // it('should ping successfuly', async () => {
  //   node.ping = sinon.fake.returns({ latency: 10 })

  //   const res = await request(service).post(`/api/v2/node/ping`).send({ peerId: testPeerId })
  //   expect(res.status).to.equal(200)
  //   expect(res).to.satisfyApiSpec
  //   expect(res.body).to.deep.equal({ latency: 10 })
  // })

  // it('should return error on invalid peerId', async () => {
  //   node.ping = sinon.fake.returns({ latency: 10 })

  //   const res = await request(service).post(`/api/v2/node/ping`).send({ peerId: invalidTestPeerId })
  //   expect(res.status).to.equal(400)
  //   expect(res).to.satisfyApiSpec
  //   expect(res.body).to.deep.equal({ status: STATUS_CODES.INVALID_PEERID })
  // })

  // it('should return propper error on ping fail', async () => {
  //   node.ping = sinon.fake.throws('')

  //   const res = await request(service).post(`/api/v2/node/ping`).send({ peerId: testPeerId })
  //   expect(res.status).to.equal(422)
  //   expect(res).to.satisfyApiSpec
  //   expect(res.body.status).to.equal(STATUS_CODES.UNKNOWN_FAILURE)
  // })
})
