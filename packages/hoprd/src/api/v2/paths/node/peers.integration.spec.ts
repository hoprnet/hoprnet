import request from 'supertest'
import sinon from 'sinon'
import chaiResponseValidator from 'chai-openapi-response-validator'
import chai, { expect } from 'chai'
import {
  createTestApiInstance,
  ALICE_PEER_ID,
  ALICE_MULTI_ADDR,
  BOB_PEER_ID,
  BOB_MULTI_ADDR,
  CHARLIE_PEER_ID
} from '../../fixtures'
import { STATUS_CODES } from '../../utils'

const ALICE_ENTRY = {
  id: ALICE_PEER_ID,
  heartbeatsSent: 10,
  heartbeatsSuccess: 10,
  lastSeen: 1646410980793,
  backoff: 0,
  quality: 1
}
const ALICE_PEER_INFO = {
  peerId: ALICE_PEER_ID.toB58String(),
  multiAddr: ALICE_MULTI_ADDR.toString(),
  heartbeats: {
    sent: ALICE_ENTRY.heartbeatsSent,
    success: ALICE_ENTRY.heartbeatsSuccess
  },
  lastSeen: ALICE_ENTRY.lastSeen,
  quality: ALICE_ENTRY.quality,
  backoff: ALICE_ENTRY.backoff,
  isNew: false
}

const BOB_ENTRY = {
  id: BOB_PEER_ID,
  heartbeatsSent: 0,
  heartbeatsSuccess: 0,
  lastSeen: 1646410680793,
  backoff: 0,
  quality: 0.2
}
const BOB_PEER_INFO = {
  peerId: BOB_PEER_ID.toB58String(),
  multiAddr: BOB_MULTI_ADDR.toString(),
  heartbeats: {
    sent: BOB_ENTRY.heartbeatsSent,
    success: BOB_ENTRY.heartbeatsSuccess
  },
  lastSeen: BOB_ENTRY.lastSeen,
  quality: BOB_ENTRY.quality,
  backoff: BOB_ENTRY.backoff,
  isNew: true
}

const CHARLIE_ENTRY = {
  id: CHARLIE_PEER_ID,
  heartbeatsSent: 10,
  heartbeatsSuccess: 8,
  lastSeen: 1646410980993,
  backoff: 0,
  quality: 0.8
}
const CHARLIE_PEER_INFO = {
  peerId: CHARLIE_PEER_ID.toB58String(),
  heartbeats: {
    sent: CHARLIE_ENTRY.heartbeatsSent,
    success: CHARLIE_ENTRY.heartbeatsSuccess
  },
  lastSeen: CHARLIE_ENTRY.lastSeen,
  quality: CHARLIE_ENTRY.quality,
  backoff: CHARLIE_ENTRY.backoff,
  isNew: false
}

let node = sinon.fake() as any
node.getConnectedPeers = sinon.fake.returns([ALICE_PEER_ID, BOB_PEER_ID, CHARLIE_PEER_ID])
node.getAddressesAnnouncedOnChain = sinon.fake.resolves([ALICE_MULTI_ADDR, BOB_MULTI_ADDR])
node.getConnectionInfo = sinon.stub()
// we must use `sinon.match.has` as passing the plain PeerId in `withArgs` fails to work
node.getConnectionInfo.withArgs(sinon.match.has('_idB58String', ALICE_PEER_ID.toB58String())).returns(ALICE_ENTRY)
node.getConnectionInfo.withArgs(sinon.match.has('_idB58String', BOB_PEER_ID.toB58String())).returns(BOB_ENTRY)
node.getConnectionInfo.withArgs(sinon.match.has('_idB58String', CHARLIE_PEER_ID.toB58String())).returns(CHARLIE_ENTRY)

const { api, service } = createTestApiInstance(node)
chai.use(chaiResponseValidator(api.apiDoc))

describe('GET /node/peers', function () {
  it('should return invalid quality when quality is not a number', async function () {
    const res = await request(service).get(`/api/v2/node/peers?quality=abc`).send()
    expect(res.status).to.equal(400)
    expect(res).to.satisfyApiSpec
    expect(res.body.status).to.equal(STATUS_CODES.INVALID_QUALITY)
  })

  it('should return invalid quality when quality is greater than 1', async function () {
    const res = await request(service).get(`/api/v2/node/peers?quality=2`).send()
    expect(res.status).to.equal(400)
    expect(res).to.satisfyApiSpec
    expect(res.body.status).to.equal(STATUS_CODES.INVALID_QUALITY)
  })

  it('should return invalid quality when quality is less than 0', async function () {
    const res = await request(service).get(`/api/v2/node/peers?quality=-1`).send()
    expect(res.status).to.equal(400)
    expect(res).to.satisfyApiSpec
    expect(res.body.status).to.equal(STATUS_CODES.INVALID_QUALITY)
  })

  it('should resolve with all data', async function () {
    const res = await request(service).get(`/api/v2/node/peers`).send()

    expect(res.status).to.equal(200)
    expect(res).to.satisfyApiSpec
    expect(res.body).to.deep.equal({
      connected: [ALICE_PEER_INFO, BOB_PEER_INFO, CHARLIE_PEER_INFO],
      announced: [ALICE_PEER_INFO, BOB_PEER_INFO]
    })
  })

  it('should resolve with active nodes', async function () {
    const res = await request(service).get(`/api/v2/node/peers?quality=0.5`).send()

    expect(res.status).to.equal(200)
    expect(res).to.satisfyApiSpec
    expect(res.body).to.deep.equal({
      connected: [ALICE_PEER_INFO, CHARLIE_PEER_INFO],
      announced: [ALICE_PEER_INFO]
    })
  })
})
