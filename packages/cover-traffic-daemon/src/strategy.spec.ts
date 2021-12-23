import sinon from 'sinon'
import { State, type PersistedState } from './state'
import { log, mockChannelEntry } from './state.mock'
import proxyquire from 'proxyquire'
import fs from 'fs'
import { expect } from 'chai'
import BN from 'bn.js'
import Hopr from '@hoprnet/hopr-core'
import { CoverTrafficStrategy } from './strategy'

describe('cover traffic strategy', async function () {
  let mockCoverTrafficStrategy
  beforeEach(async function () {
    // have a mock state
    const existsSyncStub = sinon.stub(fs, 'existsSync').callsFake((_path) => false)
    const writeFileSyncStub = sinon.stub(fs, 'writeFileSync').callsFake((..._args) => null)
    const persistMock = proxyquire('./state', {
      fs: {
        existsSync: existsSyncStub,
        readFileSync: null,
        writeFileSync: writeFileSyncStub
      }
    })
    const mockPersistedState: PersistedState = new persistMock.PersistedState((state: State) => {
      log(`State update: ${Object.keys(state.nodes).length} nodes, ${Object.keys(state.channels).length} channels`)
    }, './test/ct.json')

    const mockHoprNode = sinon.createStubInstance(Hopr)
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
