import assert from 'assert'
import sinon from 'sinon'
import { PublicKey } from '@hoprnet/hopr-utils'
import { getAddresses } from './addresses'
import { ALICE_PEER_ID } from '../../fixtures'

let node = sinon.fake() as any

describe('test address', function () {
  const ALICE_ETH_ADDRESS = PublicKey.fromPeerId(ALICE_PEER_ID).toAddress()

  it('should get addresses', async function () {
    node.getEthereumAddress = sinon.fake.returns(ALICE_ETH_ADDRESS)
    node.getId = sinon.fake.returns(ALICE_PEER_ID)

    const { native, hopr } = getAddresses(node)
    assert.equal(native, ALICE_ETH_ADDRESS.toString())
    assert.equal(hopr, ALICE_PEER_ID.toB58String())
  })
})
