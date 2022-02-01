import assert from 'assert'
import sinon from 'sinon'
import { listChannels } from './list'
import { ChannelEntry } from '@hoprnet/hopr-utils'
import { testPeerIdInstance } from '../../fixtures'

let node = sinon.fake() as any
node.getId = sinon.fake.returns(testPeerIdInstance)

describe('listChannels', function () {
  const testChannel = ChannelEntry.createMock()
  node.getChannelsFrom = sinon.fake.returns(Promise.resolve([testChannel]))
  node.getChannelsTo = sinon.fake.returns(Promise.resolve([testChannel]))

  it('should get channels list including closed', async function () {
    const { incoming, outgoing } = await listChannels(node, true)
    assert.equal(incoming.length, 1)
    assert.equal(outgoing.length, 1)
  })
  it('should get channels list excluding closed', async function () {
    const { incoming, outgoing } = await listChannels(node, false)

    assert.equal(incoming.length, 0)
    assert.equal(outgoing.length, 0)
  })
})
