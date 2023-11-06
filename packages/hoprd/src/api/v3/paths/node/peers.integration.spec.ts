import request from 'supertest'
import sinon from 'sinon'
import chaiResponseValidator from 'chai-openapi-response-validator'
import chai, { expect } from 'chai'
import { AccountEntry, PeerOrigin, PeerStatus, peer_metadata_protocol_version_name, Hopr } from '@hoprnet/hopr-utils'

import {
  createTestApiInstance,
  ALICE_PEER_ID,
  ALICE_MULTI_ADDR,
  ALICE_ACCOUNT_ENTRY,
  BOB_PEER_ID,
  BOB_MULTI_ADDR,
  BOB_ACCOUNT_ENTRY,
  CHARLIE_PEER_ID,
  CHARLIE_ACCOUNT_ENTRY
} from '../../fixtures.js'
import { STATUS_CODES } from '../../utils.js'

const meta: Map<string, string> = new Map([[peer_metadata_protocol_version_name(), '1.2.3']])

const ALICE_ENTRY = PeerStatus.build(
  ALICE_PEER_ID.toString(),
  PeerOrigin.Initialization,
  false,
  BigInt(1646410980793),
  1.0,
  BigInt(10),
  BigInt(10),
  0,
  meta,
  10
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
  meta,
  10
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
  meta,
  10
)

function toJsonDict(account: AccountEntry, peer: PeerStatus, isNew: boolean, multiaddr: string | undefined) {
  return {
    peerId: peer.peer_id(),
    peerAddress: account.chain_addr.to_string(),
    multiAddr: multiaddr ? multiaddr.toString() : '',
    heartbeats: {
      sent: Number(peer.heartbeats_sent),
      success: Number(peer.heartbeats_succeeded)
    },
    lastSeen: Number(peer.last_seen),
    quality: peer.quality(),
    backoff: peer.backoff,
    isNew: isNew,
    reportedVersion: peer.metadata().get(peer_metadata_protocol_version_name()) ?? 'unknown'
  }
}

// sinon.fake always attempts to deserialize types, but it deserializes BigInt as a string
;(BigInt.prototype as any).toJSON = function () {
  return Number(this)
}

const ALICE_PEER_INFO = toJsonDict(ALICE_ACCOUNT_ENTRY, ALICE_ENTRY, false, ALICE_MULTI_ADDR.toString())
const BOB_PEER_INFO = toJsonDict(BOB_ACCOUNT_ENTRY, BOB_ENTRY, true, BOB_MULTI_ADDR.toString())
const CHARLIE_PEER_INFO = toJsonDict(CHARLIE_ACCOUNT_ENTRY, CHARLIE_ENTRY, false, undefined)

let node = sinon.fake() as any as Hopr
node.getConnectedPeers = sinon.fake.resolves([
  ALICE_PEER_ID.toString(),
  BOB_PEER_ID.toString(),
  CHARLIE_PEER_ID.toString()
])
node.getAccountsAnnouncedOnChain = async () => {
  return [ALICE_ACCOUNT_ENTRY, BOB_ACCOUNT_ENTRY]
}

node.getPeerInfo = async (peer: string) => {
  switch (peer) {
    case ALICE_PEER_ID.toString():
      return ALICE_ENTRY
    case BOB_PEER_ID.toString():
      return BOB_ENTRY
    case CHARLIE_PEER_ID.toString():
      return CHARLIE_ENTRY
  }
}

node.peerIdToChainKey = async (peer: string) => {
  switch (peer) {
    case ALICE_PEER_ID.toString():
      return ALICE_ACCOUNT_ENTRY.chain_addr
    case BOB_PEER_ID.toString():
      return BOB_ACCOUNT_ENTRY.chain_addr
    case CHARLIE_PEER_ID.toString():
      return CHARLIE_ACCOUNT_ENTRY.chain_addr
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
