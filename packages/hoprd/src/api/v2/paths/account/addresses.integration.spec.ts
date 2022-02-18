import request from 'supertest'
import sinon from 'sinon'
import { Address } from '@hoprnet/hopr-utils'
import PeerId from 'peer-id'
import chaiResponseValidator from 'chai-openapi-response-validator'
import chai, { expect } from 'chai'
import { createTestApiInstance, testEthAddress, testPeerId } from '../../fixtures'

let node = sinon.fake() as any
node.getEthereumAddress = sinon.fake.returns(Address.fromString(testEthAddress))
node.getId = sinon.fake.returns(PeerId.createFromB58String(testPeerId))

const { api, service } = createTestApiInstance(node)
chai.use(chaiResponseValidator(api.apiDoc))

describe('GET /account/addresses', () => {
  it('should return addresses', async () => {
    const res = await request(service).get('/api/v2/account/addresses')
    expect(res.status).to.equal(200)
    expect(res).to.satisfyApiSpec
    expect(res.body).to.deep.equal({
      native: testEthAddress,
      hopr: testPeerId,
      nativeAddress: testEthAddress,
      hoprAddress: testPeerId
    })
  })
})
