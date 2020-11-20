import Heartbeat from './heartbeat'
import NetworkPeerStore from './network-peers'
import PeerId from 'peer-id'
import assert from 'assert'
import { generateLibP2PMock } from '../test-utils'
import { Interactions } from '../interactions'
import { Network } from '../network'
import type { Connection } from 'libp2p'
import { Heartbeat as HeartbeatInteraction } from '../interactions/network/heartbeat'
import debug from 'debug'
// @ts-ignore
import sinon from 'sinon'

const log = debug('hopr:heartbeat-tests')

async function generateMocks(options?: { timeoutIntentionally: boolean }, addr = '/ip4/0.0.0.0/tcp/0') {
  const { node, address } = await generateLibP2PMock(addr)

  node.hangUp = async (_id) => {} // Need to override this as we don't have real conns

  const interactions = {
    network: {
      heartbeat: new HeartbeatInteraction(node, (remotePeer) => network.heartbeat.emit('beat', remotePeer))
    }
  } as Interactions<any>

  const network = new Network(node, interactions, {} as any, { crawl: options })

  node.connectionManager.on('peer:connect', (connection: Connection) => {
    log('> Connection from', connection.remotePeer)
    node.peerStore.addressBook.add(connection.remotePeer, [connection.remoteAddr])
    network.networkPeers.register(connection.remotePeer)
  })

  return {
    node,
    address,
    network,
    interactions
  }
}

describe('check heartbeat mechanism', function () {
  it('should initialise the heartbeat module and start the heartbeat functionality', async function () {
    const [Alice, Bob, Chris] = await Promise.all([generateMocks(), generateMocks(), generateMocks()])

    await Alice.node.dial(Bob.address)

    // Check whether our event listener is triggered by heartbeat interactions
    await Promise.all([
      new Promise(async (resolve) => {
        Bob.network.heartbeat.once('beat', (peerId: PeerId) => {
          log('bob heartbeat from alice')
          assert(Alice.node.peerId.isEqual(peerId), `Incoming connection must come from Alice`)
          resolve()
        })
      }),
      Alice.interactions.network.heartbeat.interact(Bob.node.peerId)
    ])

    assert(!Chris.network.networkPeers.has(Alice.node.peerId), `Chris should not know about Alice in the beginning.`)

    await Alice.node.dial(Chris.address)

    // Check that the internal state is as asserted
    assert(Alice.network.networkPeers.has(Chris.node.peerId), `Alice should know about Chris now.`)
    assert(Alice.network.networkPeers.has(Bob.node.peerId), `Alice should know about Bob now.`)
    assert(Chris.network.networkPeers.has(Alice.node.peerId), `Chris should know about Alice now.`)
    assert(Bob.network.networkPeers.has(Alice.node.peerId), `Bob should know about Alice now.`)

    // Simulate a node failure
    await Chris.node.stop()

    //TODO simulate wait for it to be oldest
    // Check whether a node failure gets detected
    // TODO await Alice.network.heartbeat.checkNodes()
    // TODO assert(!Alice.network.networkPeers.has(Chris.node.peerId), `Alice should have removed Chris.`)

    await Promise.all([
      /* pretier-ignore */
      Alice.node.stop(),
      Bob.node.stop()
    ])
  })
})

describe('unit test heartbeat', () => {
  let heartbeat
  let hangUp = sinon.fake()
  let peers: NetworkPeerStore
  let interaction = {
    interact: sinon.fake()
  } as any

  beforeEach(() => {
    peers = new NetworkPeerStore([])
    heartbeat = new Heartbeat(peers, interaction, hangUp)
  })

  it('check nodes is noop with empty store', async () => {
    await heartbeat.checkNodes()
    assert(hangUp.notCalled, 'hangup not called')
    assert(interaction.interact.notCalled, 'interact not called')
  })

  it('check nodes is noop with only new peers', async () => {
    peers.register(PeerId.createFromB58String('16Uiu2HAmShu5QQs3LKEXjzmnqcT8E3YqyxKtVTurWYp8caM5jYJq'))
    await heartbeat.checkNodes()
    assert(hangUp.notCalled)
    assert(interaction.interact.notCalled)
  })

  it('check nodes interacts with an old peer', async () => {
    // TODO
    /*
    peers.register(PeerId.createFromB58String('16Uiu2HAmShu5QQs3LKEXjzmnqcT8E3YqyxKtVTurWYp8caM5jYJw'))
    await heartbeat.checkNodes()
    assert(hangUp.notCalled)
    assert(interaction.interact.calledOnce)
    */
  })
})
