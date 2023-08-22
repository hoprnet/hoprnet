import sinon from 'sinon'
import assert from 'assert'

import { getPeerInfo } from './index.js'
import { ALICE_MULTI_ADDR, BOB_MULTI_ADDR, CHARLIE_PEER_ID } from '../../../fixtures.js'

let node = sinon.fake() as any
node.getAddressesAnnouncedToDHT = sinon.fake.resolves([ALICE_MULTI_ADDR, BOB_MULTI_ADDR])
node.getObservedAddresses = sinon.fake.returns([ALICE_MULTI_ADDR])

describe('get peer info', function () {
  it('should resolve with all data', async function () {
    const peers = await getPeerInfo(node, CHARLIE_PEER_ID)
    assert.equal(peers.announced.length, 2, 'getPeerInfo did not return correct amount of multiaddresses')
    assert.equal(peers.observed.length, 1, 'getPeerInfo did not return correct amount of multiaddresses')
  })
})
