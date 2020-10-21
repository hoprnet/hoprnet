import PeerId from 'peer-id'
import libp2p from 'libp2p'
import type {Connection} from 'libp2p'
// @ts-ignore
import TCP = require('libp2p-tcp')
// @ts-ignore
import MPLEX = require('libp2p-mplex')
// @ts-ignore
import SECIO = require('libp2p-secio')
import { Heartbeat as HeartbeatInteraction } from '../interactions/network/heartbeat'
import Heartbeat from './heartbeat'
import NetworkPeerStore from './network-peers'
import { Network } from './index'

import assert from 'assert'
import Multiaddr from 'multiaddr'
import { LibP2P } from '..'
import { Interactions } from '../interactions'

type Mocks = {
  node: LibP2P
  network: Network
  interactions: Interactions<any>
}

describe('check heartbeat mechanism', function () {
  async function generateMocks(
    options?: { timeoutIntentionally: boolean },
    addr = '/ip4/0.0.0.0/tcp/0'
  ): Promise<Mocks> {
    const node = await libp2p.create({
      peerId: await PeerId.create({ keyType: 'secp256k1' }),
      modules: {
        transport: [TCP],
        streamMuxer: [MPLEX],
        connEncryption: [SECIO]
      }
    })

    node.multiaddrs.add(Multiaddr(addr))
    node.hangUp = async (_id) => {} // Need to override this in tests.

    await node.start()

    const interactions = {
      network: {
        heartbeat: new HeartbeatInteraction(node, (remotePeer) => network.heartbeat.emit('beat', remotePeer))
      }
    } as Interactions<any>

    const network = new Network(node, interactions, {} as any, { crawl: options })

    node.getConnectedPeers = () => node._network.networkPeers.peers.map((x) => x.id)
    node.on('peer:connect', (connection: Connection) => node.peerStore.addressBook.put(connection.remotePeer.id))
    return {
      node,
      interactions,
      network
    }
  }

  it('should initialise the heartbeat module and start the heartbeat functionality', async function () {
    const [Alice, Bob, Chris] = await Promise.all([generateMocks(), generateMocks(), generateMocks()])

    await Alice.node.dial(Bob.node.peerId)

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

    await Alice.node.dial(Chris.node.peerId)

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
