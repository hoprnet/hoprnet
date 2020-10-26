import Heartbeat from './heartbeat'
import NetworkPeerStore from './network-peers'
import PeerId from 'peer-id'
import assert from 'assert'
import { generateLibP2PMock } from '../test-utils'
import { Interactions } from '../interactions'
import { Network } from '../network'
import type {Connection} from 'libp2p'
import { Heartbeat as HeartbeatInteraction } from '../interactions/network/heartbeat'


async function generateMocks(
  options?: { timeoutIntentionally: boolean },
  addr = '/ip4/0.0.0.0/tcp/0'
) {
  const {node, address} = await generateLibP2PMock(addr)

  const interactions = {
    network: {
      heartbeat: new HeartbeatInteraction(node, (remotePeer) => network.heartbeat.emit('beat', remotePeer))
    }
  } as Interactions<any>

  const network = new Network(node, interactions, {} as any, { crawl: options })

  node.on('peer:connect', (connection: Connection) => node.peerStore.addressBook.add(connection.remotePeer, [connection.remoteAddr]))

  return {
    node, address, network, interactions
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
          assert(Alice.node.peerId.isEqual(peerId), `Incoming connection must come from Alice`)
          resolve()
        })
      }),
      Alice.interactions.network.heartbeat.interact(Bob.node.peerId)
    ])

    assert(
      !Chris.network.networkPeers.has(Alice.node.peerId),
      `Chris should not know about Alice in the beginning.`
    )

    await Alice.node.dial(Chris.address)

    // Check that the internal state is as expected
    assert(Alice.network.networkPeers.has(Chris.node.peerId), `Alice should know about Chris now.`)
    assert(Alice.network.networkPeers.has(Bob.node.peerId), `Alice should know about Bob now.`)
    assert(Chris.network.networkPeers.has(Alice.node.peerId), `Chris should know about Alice now.`)
    assert(Bob.network.networkPeers.has(Alice.node.peerId), `Bob should know about Alice now.`)

    // Simulate a node failure
    await Chris.node.stop()

    for (let i = 0; i < Alice.network.networkPeers.peers.length; i++) {
      Alice.network.networkPeers.peers[i].lastSeen = 0
    }

    // Check whether a node failure gets detected
    await Alice.network.heartbeat.checkNodes()

    assert(!Alice.network.networkPeers.has(Chris.node.peerId), `Alice should have removed Chris.`)

    await Promise.all([
      /* pretier-ignore */
      Alice.node.stop(),
      Bob.node.stop()
    ])
  })
})

describe('unit test heartbeat', () => {
  let heartbeat
  let hangUp = jest.fn(async () => {})
  let peers
  let interaction = {
    interact: jest.fn(() => {})
  } as any

  beforeEach(() => {
    peers = new NetworkPeerStore([])
    heartbeat = new Heartbeat(peers, interaction, hangUp)
  })

  it('check nodes is noop with empty store', async () => {
    await heartbeat.checkNodes()
    expect(hangUp.mock.calls.length).toBe(0)
    expect(interaction.interact.mock.calls.length).toBe(0)
  })

  it('check nodes is noop with only new peers', async () => {
    peers.push({
      id: PeerId.createFromB58String('16Uiu2HAmShu5QQs3LKEXjzmnqcT8E3YqyxKtVTurWYp8caM5jYJq'),
      lastSeen: Date.now()
    })
    await heartbeat.checkNodes()
    expect(hangUp.mock.calls.length).toBe(0)
    expect(interaction.interact.mock.calls.length).toBe(0)
  })

  it('check nodes interacts with an old peer', async () => {
    peers.push({ id: PeerId.createFromB58String('16Uiu2HAmShu5QQs3LKEXjzmnqcT8E3YqyxKtVTurWYp8caM5jYJw'), lastSeen: 0 })
    await heartbeat.checkNodes()
    expect(hangUp.mock.calls.length).toBe(0)
    expect(interaction.interact.mock.calls.length).toBe(1)
  })
})
