import request from 'supertest'
import sinon from 'sinon'
import chaiResponseValidator from 'chai-openapi-response-validator'
import chai, { expect } from 'chai'
import { createTestApiInstance, testAlias, testPeerId } from '../../fixtures'
import { STATUS_CODES } from '../../utils'

let node = sinon.fake() as any

const { api, service } = createTestApiInstance(node)
chai.use(chaiResponseValidator(api.apiDoc))

describe('GET /aliases/{alias}', () => {
  it('should successfuly get alias', async () => {
    await request(service).post('/api/v2/aliases').send({
      peerId: testPeerId,
      alias: testAlias
    })

    const res = await request(service).get(`/api/v2/aliases/${testAlias}`)
    expect(res.status).to.equal(200)
    expect(res).to.satisfyApiSpec
    expect(res.body).to.deep.equal({
      peerId: testPeerId
    })
  })
  it('should return 404 on invalid peerId', async () => {
    const res = await request(service).get(`/api/v2/aliases/nonExistingAlias`)

    expect(res.status).to.equal(404)
    expect(res).to.satisfyApiSpec
    expect(res.body).to.deep.equal({ status: STATUS_CODES.PEERID_NOT_FOUND })
  })
})

describe('DELETE /aliases/{alias}', () => {
  it('should remove alias successfuly', async () => {
    await request(service).post('/api/v2/aliases').send({
      peerId: testPeerId,
      alias: testAlias
    })

    const res = await request(service).delete(`/api/v2/aliases/${testAlias}`)

    expect(res.status).to.equal(204)
    expect(res).to.satisfyApiSpec
    expect(res.body).to.be.empty
  })
})
