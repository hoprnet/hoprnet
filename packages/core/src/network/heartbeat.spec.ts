import Heartbeat from './heartbeat'
import NetworkPeerStore from './network-peers'
import assert from 'assert'
import { HEARTBEAT_INTERVAL, NETWORK_QUALITY_THRESHOLD } from '../constants'
import sinon from 'sinon'
import { fakePeerId } from '../test-utils'
import type PeerId from 'peer-id'

describe('unit test heartbeat', async () => {
  let heartbeat: Heartbeat
  let hangUp = sinon.fake.resolves(undefined)
  let peers: NetworkPeerStore
  let clock: any

  let interaction = {
    interact: sinon.fake.resolves(true)
  } as any

  beforeEach(() => {
    clock = sinon.useFakeTimers(Date.now())
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
    peers.register(fakePeerId(1))
    await heartbeat.__forTestOnly_checkNodes()
    assert(hangUp.notCalled)
    assert(interaction.interact.notCalled)
  })

  it('check nodes interacts with an old peer', async () => {
    peers.register(fakePeerId(2))
    clock.tick(HEARTBEAT_INTERVAL * 2)
    await heartbeat.__forTestOnly_checkNodes()
    assert(hangUp.notCalled, 'shouldnt call hangup')
    assert(interaction.interact.calledOnce, 'should call interact')
  })

  it('test heartbeat flow', async () => {
    let generateMock = (i: string | number) => {
      let id = fakePeerId(i)
      let peers = new NetworkPeerStore([])
      let heartbeat = new Heartbeat(peers, interaction, hangUp)
      return { peers, interaction, id, heartbeat }
    }
    let alice = generateMock(1)
    let bob = generateMock(2)
    let chris = generateMock(3)

    let dial = (source: any, dest: any) => {
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

    // Alice heartbeat, all available
    clock.tick(HEARTBEAT_INTERVAL * 2)
    await alice.heartbeat.__forTestOnly_checkNodes()
    clock.tick(HEARTBEAT_INTERVAL * 2)
    await alice.heartbeat.__forTestOnly_checkNodes()
    clock.tick(HEARTBEAT_INTERVAL * 2)
    await alice.heartbeat.__forTestOnly_checkNodes()
    clock.tick(HEARTBEAT_INTERVAL * 2)
    await alice.heartbeat.__forTestOnly_checkNodes()

    assert(alice.peers.qualityOf(bob.id) > NETWORK_QUALITY_THRESHOLD, 'bob is high q')
    assert(alice.peers.qualityOf(chris.id) > NETWORK_QUALITY_THRESHOLD, 'chris is high q')

    // Chris dies, alice heartbeats again
    alice.interaction.interact = sinon.fake((id: PeerId) => {
      if (id.equals(chris.id)) {
        return Promise.resolve(-1)
      }
      return Promise.resolve(0)
    })

    clock.tick(HEARTBEAT_INTERVAL * 2)
    await alice.heartbeat.__forTestOnly_checkNodes()
    assert(alice.peers.qualityOf(bob.id) > NETWORK_QUALITY_THRESHOLD, 'bob is still high q')
    assert(alice.peers.qualityOf(chris.id) <= NETWORK_QUALITY_THRESHOLD, 'chris is now low q')
  })
})
