import LibP2P from 'libp2p'
import Hopr from '@hoprnet/hopr-core'
import { debug, PublicKey, wait } from '@hoprnet/hopr-utils'
import sinon from 'sinon'
import { PersistedState } from './state'
import { CoverTrafficStrategy } from './strategy'
import { sampleOptions } from './mocks/core'
import { dbMock } from './mocks/db'
import { sampleData } from './mocks/state'
import { chainMock } from './mocks/chain'
import { libp2pMock } from './mocks/libp2p'
import { mockPeerId } from './mocks/constants'

const namespace = 'hopr:test:cover-traffic'
const log = debug(namespace)

describe('cover-traffic daemon', async function () {
  let node: Hopr, data: PersistedState

  beforeEach(function () {
    function stubLibp2p() {
      sinon.stub(LibP2P, 'create').callsFake(() => {
        log('libp2p stub started')
        return Promise.resolve(libp2pMock)
      })
    }

    data = sampleData
    stubLibp2p()
    node = new Hopr(mockPeerId, dbMock, chainMock, sampleOptions)
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
