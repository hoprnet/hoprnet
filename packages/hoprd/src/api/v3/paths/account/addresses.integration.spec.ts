import request from 'supertest'
import sinon from 'sinon'
import chaiResponseValidator from 'chai-openapi-response-validator'
import chai, { expect } from 'chai'
import { PublicKey } from '@hoprnet/hopr-utils'
import { createTestApiInstance, ALICE_PEER_ID } from '../../fixtures.js'

let node = sinon.fake() as any

describe('GET /account/addresses', () => {
  const ALICE_ETH_ADDRESS = () => PublicKey.from_peerid_str(ALICE_PEER_ID.toString()).to_address()

  let service: any
  before(async function () {
    const loaded = await createTestApiInstance(node)

    service = loaded.service

    // @ts-ignore ESM / CommonJS compatibility issue
    chai.use(chaiResponseValidator.default(loaded.api.apiDoc))
  })

  it('should return addresses', async () => {
    node.getEthereumAddress = sinon.fake.returns(ALICE_ETH_ADDRESS())
    node.getId = sinon.fake.returns(ALICE_PEER_ID)

    const res = await request(service).get('/api/v3/account/addresses')
    expect(res.status).to.equal(200)
    expect(res).to.satisfyApiSpec
    expect(res.body).to.deep.equal({
      native: ALICE_ETH_ADDRESS().to_string(),
      hopr: ALICE_PEER_ID.toString(),
      nativeAddress: ALICE_ETH_ADDRESS().to_string(),
      hoprAddress: ALICE_PEER_ID.toString()
    })
  })
})
