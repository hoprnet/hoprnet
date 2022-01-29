import sinon from 'sinon'
import { State, type PersistedState } from './state'
import { log, mockChannelEntry, mockOpenChannel, mockPeerData, mockPublicKey, mockState } from './state.mock'
import proxyquire from 'proxyquire'
import fs from 'fs'
import { expect } from 'chai'
import BN from 'bn.js'

describe('cover traffic state', async function () {
  let mockPersistedState: PersistedState
  beforeEach(async function () {
    const existsSyncStub = sinon.stub(fs, 'existsSync').callsFake((_path) => false)
    const writeFileSyncStub = sinon.stub(fs, 'writeFileSync').callsFake((..._args) => null)
    const persistMock = proxyquire('./state', {
      fs: {
        existsSync: existsSyncStub,
        readFileSync: null,
        writeFileSync: writeFileSyncStub
      }
    })
    mockPersistedState = new persistMock.PersistedState((state: State) => {
      log(`State update: ${Object.keys(state.nodes).length} nodes, ${Object.keys(state.channels).length} channels`)
    }, './test/ct.json')
  })

  afterEach(function () {
    sinon.restore()
  })

  it('should get state', async function () {
    const state = mockPersistedState.get()
    expect(state).to.deep.equal({
      nodes: {},
      channels: {},
      ctChannels: [],
      messageFails: {},
      messageTotalSuccess: 0,
      block: new BN('0')
    })
  })

  it('should set state', async function () {
    mockPersistedState.set(mockState)
    const state = mockPersistedState.get()
    expect(state).to.deep.equal(mockState)
  })

  it('should set a new channel', async function () {
    mockPersistedState.setChannel(mockChannelEntry)
    const state = mockPersistedState.get()
    expect(state.channels[mockChannelEntry.getId().toHex()]).to.deep.equal({
      channel: mockChannelEntry,
      sendAttempts: 0,
      forwardAttempts: 0
    })
  })

  it('should set a new node', async function () {
    mockPersistedState.setNode(mockPeerData)
    const state = mockPersistedState.get()
    expect(state.nodes[mockPeerData.id.toB58String()]).to.deep.equal({
      id: mockPeerData.id,
      multiaddrs: mockPeerData.multiaddrs,
      pub: mockPublicKey
    })
  })

  it('should set a new CT channel', async function () {
    mockPersistedState.setCTChannels([mockOpenChannel])
    const state = mockPersistedState.get()
    expect(state.ctChannels).to.deep.equal([mockOpenChannel])
  })

  it('should find channels from PARTY_A', async function () {
    mockPersistedState.setChannel(mockChannelEntry)
    const channelEntries = mockPersistedState.findChannelsFrom(mockChannelEntry.source)
    expect(channelEntries).to.deep.equal([mockChannelEntry])
  })

  it('should set block number', async function () {
    const newBlockNumber = mockState.block.addn(4)
    mockPersistedState.setBlock(newBlockNumber)
    const state = mockPersistedState.get()
    expect(state.block.toString()).to.equal(newBlockNumber.toString())
  })

  it('should find channel between source and destination', async function () {
    mockPersistedState.setChannel(mockChannelEntry)
    const channelEntry = mockPersistedState.findChannel(mockChannelEntry.source, mockChannelEntry.destination)
    expect(channelEntry).to.deep.equal(mockChannelEntry)
  })

  it('should increase sendAttempts', async function () {
    mockPersistedState.setChannel(mockChannelEntry)
    mockPersistedState.incrementSent(mockChannelEntry.source, mockChannelEntry.destination)
    const state = mockPersistedState.get()
    expect(state.channels[mockChannelEntry.getId().toHex()]).to.deep.equal({
      channel: mockChannelEntry,
      sendAttempts: 1,
      forwardAttempts: 0
    })
  })
  it('should not increase sendAttempts for non existing channels', async function () {
    mockPersistedState.incrementSent(mockChannelEntry.source, mockChannelEntry.destination)
    const state = mockPersistedState.get()
    expect(state.channels).to.deep.equal({})
  })

  it('should increase forwardAttempts', async function () {
    mockPersistedState.setChannel(mockChannelEntry)
    mockPersistedState.incrementForwards(mockChannelEntry.source, mockChannelEntry.destination)
    const state = mockPersistedState.get()
    expect(state.channels[mockChannelEntry.getId().toHex()]).to.deep.equal({
      channel: mockChannelEntry,
      sendAttempts: 0,
      forwardAttempts: 1
    })
  })
  it('should not increase forwardAttempts for non existing channels', async function () {
    mockPersistedState.incrementForwards(mockChannelEntry.source, mockChannelEntry.destination)
    const state = mockPersistedState.get()
    expect(state.channels).to.deep.equal({})
  })

  it('should count channels', async function () {
    mockPersistedState.setChannel(mockChannelEntry)
    const num = mockPersistedState.openChannelCount()
    expect(num).to.equal(1)
  })

  it('should count channels, when no channels exists', async function () {
    const num = mockPersistedState.openChannelCount()
    expect(num).to.equal(0)
  })

  it('should count messageTotalSuccess', async function () {
    const num = mockPersistedState.messageTotalSuccess()
    expect(num).to.equal(0)
  })
  it('should increase messageTotalSuccess', async function () {
    mockPersistedState.incrementMessageTotalSuccess()
    const num = mockPersistedState.messageTotalSuccess()
    expect(num).to.equal(1)
  })
  it('should count messageFails', async function () {
    const num = mockPersistedState.messageFails(mockChannelEntry.destination)
    expect(num).to.equal(0)
  })
  it('should increase messageFails', async function () {
    mockPersistedState.incrementMessageFails(mockChannelEntry.destination)
    const num = mockPersistedState.messageFails(mockChannelEntry.destination)
    expect(num).to.equal(1)
  })
  it('should reset messageFails', async function () {
    mockPersistedState.incrementMessageFails(mockChannelEntry.destination)
    mockPersistedState.incrementMessageFails(mockChannelEntry.destination)
    mockPersistedState.resetMessageFails(mockChannelEntry.destination)
    const num = mockPersistedState.messageFails(mockChannelEntry.destination)
    expect(num).to.equal(0)
  })
})
