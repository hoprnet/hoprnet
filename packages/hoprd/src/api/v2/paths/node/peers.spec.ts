import sinon from 'sinon'
import assert from 'assert'
import PeerId from 'peer-id'
import { getPeers } from './peers'
import { STATUS_CODES } from '../../utils'
import {
  ALICE_PEER_ID,
  ALICE_MULTI_ADDR,
  BOB_PEER_ID,
  BOB_MULTI_ADDR,
  testPeerIdInstance as CHARLIE_PEER_ID
} from '../../fixtures'

const ALICE_ENTRY = {
  id: ALICE_PEER_ID,
  heartbeatsSent: 10,
  heartbeatsSuccess: 10,
  lastSeen: 1646410980793,
  backoff: 0,
  lastTen: 1
}
const BOB_ENTRY = {
  id: BOB_PEER_ID,
  heartbeatsSent: 10,
  heartbeatsSuccess: 2,
  lastSeen: 1646410680793,
  backoff: 0,
  lastTen: 0.1
}
const CHARLIE_ENTRY = {
  id: CHARLIE_PEER_ID,
  heartbeatsSent: 10,
  heartbeatsSuccess: 9,
  lastSeen: 1646410980993,
  backoff: 0,
  lastTen: 0.9
}

let node = sinon.fake() as any
node.getConnectedPeers = sinon.fake.returns([ALICE_PEER_ID, BOB_PEER_ID, CHARLIE_PEER_ID])
node.getAnnouncedAddresses = sinon.fake.resolves([ALICE_MULTI_ADDR, BOB_MULTI_ADDR])
node.getConnectionInfo = sinon.stub()
node.getConnectionInfo.withArgs(ALICE_PEER_ID).returns(ALICE_ENTRY)
node.getConnectionInfo.withArgs(BOB_PEER_ID).returns(BOB_ENTRY)
node.getConnectionInfo.withArgs(CHARLIE_PEER_ID).returns(CHARLIE_ENTRY)

describe('peers', function () {
  it('should throw on invalid quality', async function () {
    assert.rejects(
      () => getPeers(node, 10),
      (err: Error) => {
        return err.message.includes(STATUS_CODES.INVALID_QUALITY)
      }
    )
    assert.rejects(
      () => getPeers(node, undefined),
      (err: Error) => {
        return err.message.includes(STATUS_CODES.INVALID_QUALITY)
      }
    )
  })

  it('should resolve with all data', async function () {
    const peers = await getPeers(node, 0)
    assert.equal(peers.connected.length, 3, 'getPeers did not return correct amount of connected peers')
    assert.equal(peers.announced.length, 2, 'getPeers did not return correct amount of announced peers')
  })

  it('should resolve with active nodes', async function () {
    const peers = await getPeers(node, 0.5)
    assert.equal(peers.connected.length, 2, 'getPeers did not return correct amount of active connected peers')
    assert.equal(peers.announced.length, 1, 'getPeers did not return correct amount of active announced peers')
  })
})
