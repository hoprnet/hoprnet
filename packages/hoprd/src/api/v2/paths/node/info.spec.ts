import sinon from 'sinon'
import assert from 'assert'
import { getInfo } from './info'

let node = sinon.fake() as any

describe('getInfo', () => {
  it('should get info', async () => {
    node.environment = { id: 'hardhat-localhost' }
    node.smartContractInfo = sinon.fake.returns({
      network: 'a',
      hoprTokenAddress: 'b',
      hoprChannelsAddress: 'c',
      channelClosureSecs: 60
    })
    node.getAddressesAnnouncedToDHT = sinon.fake.returns([1, 2])
    node.getListeningAddresses = sinon.fake.returns([3, 4])
    const info = await getInfo(node)
    assert.deepEqual(info, {
      environment: 'hardhat-localhost',
      announcedAddress: ['1', '2'],
      listeningAddress: ['3', '4'],
      network: 'a',
      hoprToken: 'b',
      hoprChannels: 'c',
      channelClosurePeriod: 1
    })
  })
})
