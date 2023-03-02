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
} from '../../fixtures.js'
import { STATUS_CODES } from '../../utils.js'
import type { PeerId } from '@libp2p/interface-peer-id'
import type Hopr from '@hoprnet/hopr-core'
import { PeerOrigin, PeerStatus } from '@hoprnet/hopr-core'

const ALICE_ENTRY = PeerStatus.build(
  ALICE_PEER_ID.toString(),
  PeerOrigin.Initialization,
  false,
  BigInt(1646410980793),
  1.0,
  BigInt(10),
  BigInt(10),
  0
)

const ALICE_PEER_INFO = {
  peerId: ALICE_PEER_ID.toString(),
  multiAddr: ALICE_MULTI_ADDR.toString(),
  heartbeats: {
    sent: ALICE_ENTRY.heartbeats_sent,
    success: ALICE_ENTRY.heartbeats_succeeded
  },
  lastSeen: ALICE_ENTRY.last_seen,
  quality: ALICE_ENTRY.quality,
  backoff: ALICE_ENTRY.backoff,
  isNew: false
}

const BOB_ENTRY = PeerStatus.build(
  BOB_PEER_ID.toString(),
  PeerOrigin.Initialization,
  false,
  BigInt(1646410680793),
  0.2,
  BigInt(0),
  BigInt(0),
  0
)

const BOB_PEER_INFO = {
  peerId: BOB_PEER_ID.toString(),
  multiAddr: BOB_MULTI_ADDR.toString(),
  heartbeats: {
    sent: BOB_ENTRY.heartbeats_sent,
    success: BOB_ENTRY.heartbeats_succeeded
  },
  lastSeen: BOB_ENTRY.last_seen,
  quality: BOB_ENTRY.quality,
  backoff: BOB_ENTRY.backoff,
  isNew: true
}

const CHARLIE_ENTRY = PeerStatus.build(
  CHARLIE_PEER_ID.toString(),
  PeerOrigin.Initialization,
  false,
  BigInt(1646410980993),
  0.8,
  BigInt(10),
  BigInt(8),
  0
)

const CHARLIE_PEER_INFO = {
  peerId: CHARLIE_PEER_ID.toString(),
  heartbeats: {
    sent: CHARLIE_ENTRY.heartbeats_sent,
    success: CHARLIE_ENTRY.heartbeats_succeeded
  },
  lastSeen: CHARLIE_ENTRY.last_seen,
  quality: CHARLIE_ENTRY.quality,
  backoff: CHARLIE_ENTRY.backoff,
  isNew: false
}

let node = sinon.fake() as any as Hopr
node.getConnectedPeers = sinon.fake.returns([ALICE_PEER_ID, BOB_PEER_ID, CHARLIE_PEER_ID])
node.getAddressesAnnouncedOnChain = async function* () {
  yield ALICE_MULTI_ADDR
  yield BOB_MULTI_ADDR
}

node.getConnectionInfo = (peer: PeerId) => {
  switch (peer.toString()) {
    case ALICE_PEER_ID.toString():
      return ALICE_ENTRY
    case BOB_PEER_ID.toString():
      return BOB_ENTRY
    case CHARLIE_PEER_ID.toString():
      return CHARLIE_ENTRY
  }
}

describe('GET /node/peers', function () {
  let service: any
  before(async function () {
    const loaded = await createTestApiInstance(node)

    service = loaded.service

    // @ts-ignore ESM / CommonJS compatibility issue
    chai.use(chaiResponseValidator.default(loaded.api.apiDoc))
  })

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
