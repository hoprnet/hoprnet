import request from 'supertest'
import sinon from 'sinon'
import chaiResponseValidator from 'chai-openapi-response-validator'
import chai, { expect } from 'chai'
import { PublicKey } from '@hoprnet/hopr-utils'
import { createTestApiInstance, ALICE_PEER_ID } from '../../fixtures'

let node = sinon.fake() as any
const { api, service } = createTestApiInstance(node)
chai.use(chaiResponseValidator(api.apiDoc))

describe('GET /account/addresses', () => {
  const ALICE_ETH_ADDRESS = PublicKey.fromPeerId(ALICE_PEER_ID).toAddress()

  it('should return addresses', async () => {
    node.getEthereumAddress = sinon.fake.returns(ALICE_ETH_ADDRESS)
    node.getId = sinon.fake.returns(ALICE_PEER_ID)

    const res = await request(service).get('/api/v2/account/addresses')
    expect(res.status).to.equal(200)
    expect(res).to.satisfyApiSpec
    expect(res.body).to.deep.equal({
      native: ALICE_ETH_ADDRESS.toString(),
      hopr: ALICE_PEER_ID.toB58String(),
      nativeAddress: ALICE_ETH_ADDRESS.toString(),
      hoprAddress: ALICE_PEER_ID.toB58String()
    })
  })
})
