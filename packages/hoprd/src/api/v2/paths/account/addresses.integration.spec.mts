import request from 'supertest'
import sinon from 'sinon'
import chaiResponseValidator from 'chai-openapi-response-validator'
import chai, { expect } from 'chai'
import { PublicKey } from '@hoprnet/hopr-utils'
import { createTestApiInstance, ALICE_PEER_ID } from '../../fixtures.mjs'

let node = sinon.fake() as any

describe.only('GET /account/addresses', () => {
  const ALICE_ETH_ADDRESS = PublicKey.fromPeerId(ALICE_PEER_ID).toAddress()

  let service: any
  before(async function () {
    const loaded = await createTestApiInstance(node)

    service = loaded.service

    // @ts-ignore ESM / CommonJS compatibility issue
    chai.use(chaiResponseValidator.default(loaded.api.apiDoc))
  })

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
