import request from 'supertest'
import sinon from 'sinon'
import chaiResponseValidator from 'chai-openapi-response-validator'
import chai, { expect } from 'chai'
import { PeerOrigin, PeerStatus, PEER_METADATA_PROTOCOL_VERSION } from '@hoprnet/hopr-core'

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
import type { Hopr } from '@hoprnet/hopr-core'

const meta: Map<string, string> = new Map([[PEER_METADATA_PROTOCOL_VERSION, '1.2.3']])

const ALICE_ENTRY = PeerStatus.build(
  ALICE_PEER_ID.toString(),
  PeerOrigin.Initialization,
  false,
  BigInt(1646410980793),
  1.0,
  BigInt(10),
  BigInt(10),
  0,
  meta
)

const BOB_ENTRY = PeerStatus.build(
  BOB_PEER_ID.toString(),
  PeerOrigin.Initialization,
  false,
  BigInt(1646410680793),
  0.2,
  BigInt(0),
  BigInt(0),
  0,
  meta
)

const CHARLIE_ENTRY = PeerStatus.build(
  CHARLIE_PEER_ID.toString(),
  PeerOrigin.Initialization,
  false,
  BigInt(1646410980993),
  0.8,
  BigInt(10),
  BigInt(8),
  0,
  meta
)

function toJsonDict(peer: PeerStatus, isNew: boolean, multiaddr: string | undefined) {
  if (multiaddr === undefined) {
    return {
      peerId: peer.peer_id(),
      heartbeats: {
        sent: Number(peer.heartbeats_sent),
        success: Number(peer.heartbeats_succeeded)
      },
      lastSeen: Number(peer.last_seen),
      quality: peer.quality,
      backoff: peer.backoff,
      isNew: isNew,
      reportedVersion: peer.metadata().get(PEER_METADATA_PROTOCOL_VERSION) ?? 'unknown'
    }
  } else {
    return {
      peerId: peer.peer_id(),
      multiAddr: multiaddr,
      heartbeats: {
        sent: Number(peer.heartbeats_sent),
        success: Number(peer.heartbeats_succeeded)
      },
      lastSeen: Number(peer.last_seen),
      quality: peer.quality,
      backoff: peer.backoff,
      isNew: isNew,
      reportedVersion: peer.metadata().get(PEER_METADATA_PROTOCOL_VERSION) ?? 'unknown'
    }
  }
}

// sinon.fake always attempts to deserialize types, but it deserializes BigInt as a string
;(BigInt.prototype as any).toJSON = function () {
  return Number(this)
}

const ALICE_PEER_INFO = toJsonDict(ALICE_ENTRY, false, ALICE_MULTI_ADDR.toString())
const BOB_PEER_INFO = toJsonDict(BOB_ENTRY, true, BOB_MULTI_ADDR.toString())
const CHARLIE_PEER_INFO = toJsonDict(CHARLIE_ENTRY, false, undefined)

let node = sinon.fake() as any as Hopr
node.getConnectedPeers = async () => {
  return [ALICE_PEER_ID, BOB_PEER_ID, CHARLIE_PEER_ID]
}
node.getAddressesAnnouncedOnChain = async function* () {
  yield ALICE_MULTI_ADDR
  yield BOB_MULTI_ADDR
}

node.getConnectionInfo = async (peer: PeerId) => {
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
    const res = await request(service).get(`/api/v3/node/peers?quality=abc`).send()
    expect(res.status).to.equal(400)
    expect(res).to.satisfyApiSpec
    expect(res.body.status).to.equal(STATUS_CODES.INVALID_QUALITY)
  })

  it('should return invalid quality when quality is greater than 1', async function () {
    const res = await request(service).get(`/api/v3/node/peers?quality=2`).send()
    expect(res.status).to.equal(400)
    expect(res).to.satisfyApiSpec
    expect(res.body.status).to.equal(STATUS_CODES.INVALID_QUALITY)
  })

  it('should return invalid quality when quality is less than 0', async function () {
    const res = await request(service).get(`/api/v3/node/peers?quality=-1`).send()
    expect(res.status).to.equal(400)
    expect(res).to.satisfyApiSpec
    expect(res.body.status).to.equal(STATUS_CODES.INVALID_QUALITY)
  })

  it('should resolve with all data', async function () {
    const res = await request(service).get(`/api/v3/node/peers`).send()

    expect(res.status).to.equal(200)
    expect(res).to.satisfyApiSpec
    expect(res.body).to.deep.equal({
      connected: [ALICE_PEER_INFO, BOB_PEER_INFO, CHARLIE_PEER_INFO],
      announced: [ALICE_PEER_INFO, BOB_PEER_INFO]
    })
  })

  it('should resolve with active nodes', async function () {
    const res = await request(service).get(`/api/v3/node/peers?quality=0.5`).send()

    expect(res.status).to.equal(200)
    expect(res).to.satisfyApiSpec
    expect(res.body).to.deep.equal({
      connected: [ALICE_PEER_INFO, CHARLIE_PEER_INFO],
      announced: [ALICE_PEER_INFO]
    })
  })
})
