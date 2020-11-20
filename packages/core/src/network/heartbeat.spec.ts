import Heartbeat from './heartbeat'
import NetworkPeerStore from './network-peers'
import PeerId from 'peer-id'
import assert from 'assert'
import { HEARTBEAT_REFRESH } from '../constants'
// @ts-ignore
import sinon from 'sinon'
import { fakePeerId } from '../test-utils'

describe('unit test heartbeat', async () => {
  let heartbeat
  let hangUp = sinon.fake.resolves(undefined)
  let peers: NetworkPeerStore
  let clock

  let interaction = {
    interact: sinon.fake.resolves(true)
  } as any

  beforeEach(() => {
    clock = sinon.useFakeTimers(Date.now());
    peers = new NetworkPeerStore([])
    heartbeat = new Heartbeat(peers, interaction, hangUp)
  })

  afterEach(() => {
    clock.restore()
  })

  it('check nodes is noop with empty store', async () => {
    await heartbeat.__forTestOnly_checkNodes()
    assert(hangUp.notCalled, 'hangup not called')
    assert(interaction.interact.notCalled, 'interact not called')
  })

  it('check nodes is noop with only new peers', async () => {
    peers.register(PeerId.createFromB58String('16Uiu2HAmShu5QQs3LKEXjzmnqcT8E3YqyxKtVTurWYp8caM5jYJq'))
    await heartbeat.__forTestOnly_checkNodes()
    assert(hangUp.notCalled)
    assert(interaction.interact.notCalled)
  })

  it('check nodes interacts with an old peer', async () => {
    peers.register(PeerId.createFromB58String('16Uiu2HAmShu5QQs3LKEXjzmnqcT8E3YqyxKtVTurWYp8caM5jYJw'))
    clock.tick(HEARTBEAT_REFRESH * 2)
    await heartbeat.__forTestOnly_checkNodes()
    assert(hangUp.notCalled, 'shouldnt call hangup')
    assert(interaction.interact.calledOnce, 'should call interact')
  })

  it('test heartbeat flow', async() => {
    let generateMock = (i) => {
      let id = fakePeerId(i) 
      let peers = new NetworkPeerStore([])
      let heartbeat = new Heartbeat(peers, interaction, hangUp)
      return {peers, interaction, id, heartbeat}
    }
    let alice = generateMock(1)
    let bob = generateMock(2)
    let chris = generateMock(3)

    let dial = (source, dest) => {
      source.peers.register(dest.id)
      dest.peers.register(source.id)
    }
    // Setup base state
    dial(bob, alice)
    assert(!chris.peers.has(alice.id), `Chris should not know about Alice in the beginning.`)
    dial(chris, alice)
    assert(alice.peers.has(chris.id), `Alice should know about Chris now.`)
    assert(alice.peers.has(bob.id), `Alice should know about Bob now.`)
    assert(chris.peers.has(alice.id), `Chris should know about Alice now.`)
    assert(bob.peers.has(alice.id), `Bob should know about Alice now.`)

    console.log(">>> setup")
    // Alice heartbeat, all available
    //let heartbeatPromise = alice.heartbeat.__forTestOnly_checkNodes()
    clock.tick(HEARTBEAT_REFRESH * 2)
    //await heartbeatPromise
    console.log(">>> heartbeat")

    // Chris dies, alice heartbeats again
    //chris.stop

    //TODO simulate wait for it to be oldest
    // Check whether a node failure gets detected
    // TODO await Alice.network.heartbeat.checkNodes()
    // TODO assert(!Alice.network.networkPeers.has(Chris.node.peerId), `Alice should have removed Chris.`)
    return
  })
})
