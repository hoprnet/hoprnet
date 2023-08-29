import request from 'supertest'
import sinon from 'sinon'
import chaiResponseValidator from 'chai-openapi-response-validator'
import chai, { expect } from 'chai'
import { PEER_METADATA_PROTOCOL_VERSION } from '@hoprnet/hopr-core'

import { createTestApiInstance, ALICE_PEER_ID, INVALID_PEER_ID } from '../../../fixtures.js'
import { STATUS_CODES } from '../../../utils.js'

let node = sinon.fake() as any

describe('POST /peers/{peerid}/ping', () => {
  let service: any
  before(async function () {
    const loaded = await createTestApiInstance(node)

    service = loaded.service

    // @ts-ignore ESM / CommonJS compatibility issue
    chai.use(chaiResponseValidator.default(loaded.api.apiDoc))
  })

  it('should ping successfully', async () => {
    let meta = new Map<string, string>()
    meta.set(PEER_METADATA_PROTOCOL_VERSION, '1.2.3')

    node.ping = sinon.fake.resolves({ latency: 10 })
    node.getConnectionInfo = sinon.fake.returns({ metadata: () => meta })

    const res = await request(service).post(`/api/v3/peers/${ALICE_PEER_ID.toString()}/ping`).send()
    expect(res.status).to.equal(200)
    expect(res).to.satisfyApiSpec
    expect(res.body).to.deep.equal({ latency: 10, reportedVersion: '1.2.3' })
  })

  it('should return error on invalid peerId', async () => {
    node.ping = sinon.fake.returns({ latency: 10 })

    const res = await request(service).post(`/api/v3/peers/${INVALID_PEER_ID}/ping`).send()
    expect(res.status).to.equal(400)
    expect(res).to.satisfyApiSpec
    expect(res.body).to.deep.equal({ status: STATUS_CODES.INVALID_PEERID })
  })

  it('should return proper error on ping fail', async () => {
    node.ping = sinon.fake.throws('')

    const res = await request(service).post(`/api/v3/peers/${ALICE_PEER_ID.toString()}/ping`).send()
    expect(res.status).to.equal(422)
    expect(res).to.satisfyApiSpec
    expect(res.body.status).to.equal(STATUS_CODES.UNKNOWN_FAILURE)
  })
})
