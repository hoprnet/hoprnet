import LibP2P from 'libp2p'
import Hopr from '@hoprnet/hopr-core'
import { debug, PublicKey, wait, dbMock, privKeyToPeerId } from '@hoprnet/hopr-utils'
import sinon from 'sinon'
import { PersistedState } from './state'
import { CoverTrafficStrategy } from './strategy'
import { sampleData } from './state.mock'
import { sampleOptions, createLibp2pMock } from '@hoprnet/hopr-core'
import { createConnectorMock } from '@hoprnet/hopr-core-ethereum'

const namespace = 'hopr:test:cover-traffic'
const log = debug(namespace)

const privateKey = '0xcb1e5d91d46eb54a477a7eefec9c87a1575e3e5384d38f990f19c09aa8ddd332'
const mockPeerId = privKeyToPeerId(privateKey)

describe('cover-traffic daemon', async function () {
  let node: Hopr, data: PersistedState

  beforeEach(function () {
    function stubLibp2p() {
      sinon.stub(LibP2P, 'create').callsFake(() => {
        log('libp2p stub started')
        return Promise.resolve(createLibp2pMock(mockPeerId))
      })
    }
    data = sampleData
    stubLibp2p()
    const connectorMock = createConnectorMock(mockPeerId)
    log('Mocked chain', connectorMock)
    node = new Hopr(mockPeerId, dbMock, connectorMock, sampleOptions)
  })

  afterEach(function () {
    sinon.restore()
  })

  it('should run and stop properly', async function () {
    log('starting stubbed hopr node')
    await node.start()
    log('completed stubbed hopr node, starting cover-traffic strategy')
    node.setChannelStrategy(new CoverTrafficStrategy(PublicKey.fromPeerId(mockPeerId), node, data))
    log('completed strategy, waiting for 200 ms w/o crashing')
    await wait(200)
    log('Starting node stop process')
    await node.stop()
    log('Stopped node succesfully')
  })
})
