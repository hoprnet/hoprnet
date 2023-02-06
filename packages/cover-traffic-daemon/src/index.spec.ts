import assert from 'assert'
import Hopr, { type HoprOptions } from '@hoprnet/hopr-core'
import { debug, PublicKey, wait, HoprDB, privKeyToPeerId } from '@hoprnet/hopr-utils'
import { PersistedState } from './state.js'
import { CoverTrafficStrategy } from './strategy.js'
import { sampleOptions } from '@hoprnet/hopr-core'
import HoprCoreEthereum, { createConnectorMock } from '@hoprnet/hopr-core-ethereum'

const namespace = 'hopr:test:cover-traffic'
const log = debug(namespace)

const privateKey = '0xcb1e5d91d46eb54a477a7eefec9c87a1575e3e5384d38f990f19c09aa8ddd332'
const mockPeerId = privKeyToPeerId(privateKey)

describe('cover-traffic daemon', async function () {
  let node: Hopr, data: PersistedState

  beforeEach(async function () {
    const connectorMock = createConnectorMock(mockPeerId)
    log('Mocked chain', connectorMock)
    node = new Hopr(mockPeerId, HoprDB.createMock(), {
      ...sampleOptions,
      testing: {
        // Do not use real libp2p instance to keep setup simple
        useMockedLibp2p: true
      }
    } as HoprOptions)

    await node.start()
  })

  afterEach(async function () {
    await node.stop()
    await HoprCoreEthereum.getInstance().stop()
  })

  it('should run and stop properly', async function () {
    assert(node instanceof Hopr)
    log('starting stubbed hopr node')
    await node.start()
    log('completed stubbed hopr node, starting cover-traffic strategy')
    node.setChannelStrategy(new CoverTrafficStrategy(PublicKey.fromPeerId(mockPeerId), node, data))
    log('completed strategy, waiting for 200 ms w/o crashing')
    await wait(1000)
  })
})
