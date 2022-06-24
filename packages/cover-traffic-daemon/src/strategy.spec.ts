import sinon from 'sinon'
import { State, type PersistedState } from './state.js'
import { log, mockChannelEntry, TestingPersistedState } from './state.mock.js'
import { expect } from 'chai'
import { default as Hopr, type ChannelStrategyInterface } from '@hoprnet/hopr-core'
import { CoverTrafficStrategy } from './strategy.js'

describe('cover traffic strategy', async function () {
  let mockCoverTrafficStrategy: ChannelStrategyInterface
  beforeEach(async function () {
    // have a mock state
    const mockPersistedState: PersistedState = new TestingPersistedState((state: State) => {
      log(`State update: ${Object.keys(state.nodes).length} nodes, ${Object.keys(state.channels).length} channels`)
    }, './test/ct.json')
    const mockHoprNode = sinon.createStubInstance<Hopr>(Hopr)
    mockHoprNode.sendMessage.resolves()

    mockCoverTrafficStrategy = new CoverTrafficStrategy(mockChannelEntry.source, mockHoprNode, mockPersistedState)
  })

  afterEach(function () {
    sinon.restore()
  })

  it('should create a strategy', async function () {
    expect(mockCoverTrafficStrategy.name).to.equal('covertraffic')
  })
  it('should have a 10 s tick interval', async function () {
    expect(mockCoverTrafficStrategy.tickInterval).to.equal(10000)
  })
})
