import request from 'supertest'
import sinon from 'sinon'
import chaiResponseValidator from 'chai-openapi-response-validator'
import chai, { expect } from 'chai'
import { createTestApiInstance, ALICE_PEER_ID, INVALID_PEER_ID } from '../../fixtures.js'
import { STATUS_CODES } from '../../utils.js'
import { PEER_METADATA_PROTOCOL_VERSION } from '@hoprnet/hopr-core'

let node = sinon.fake() as any

describe('POST /node/ping', () => {
  let service: any
  before(async function () {
    const loaded = await createTestApiInstance(node)

    service = loaded.service

    // @ts-ignore ESM / CommonJS compatibility issue
    chai.use(chaiResponseValidator.default(loaded.api.apiDoc))
  })

  it('should ping successfuly', async () => {
    let meta = new Map<string, string>()
    meta.set(PEER_METADATA_PROTOCOL_VERSION, '1.2.3')

    node.ping = sinon.fake.returns({ latency: 10 })
    node.getConnectionInfo = sinon.fake.returns({ metadata: () => meta })

    const res = await request(service).post(`/api/v2/node/ping`).send({ peerId: ALICE_PEER_ID.toString() })
    expect(res.status).to.equal(200)
    expect(res).to.satisfyApiSpec
    expect(res.body).to.deep.equal({ latency: 10, reportedVersion: '1.2.3' })
  })

  it('should return error on invalid peerId', async () => {
    node.ping = sinon.fake.returns({ latency: 10 })

    const res = await request(service).post(`/api/v2/node/ping`).send({ peerId: INVALID_PEER_ID })
    expect(res.status).to.equal(400)
    expect(res).to.satisfyApiSpec
    expect(res.body).to.deep.equal({ status: STATUS_CODES.INVALID_PEERID })
  })

  it('should return proper error on ping fail', async () => {
    node.ping = sinon.fake.throws('')

    const res = await request(service).post(`/api/v2/node/ping`).send({ peerId: ALICE_PEER_ID.toString() })
    expect(res.status).to.equal(422)
    expect(res).to.satisfyApiSpec
    expect(res.body.status).to.equal(STATUS_CODES.UNKNOWN_FAILURE)
  })
})
