import PeerInfo from 'peer-info'
import PeerId from 'peer-id'
import libp2p from 'libp2p'
// @ts-ignore
import TCP = require('libp2p-tcp')
// @ts-ignore
import MPLEX = require('libp2p-mplex')
// @ts-ignore
import SECIO = require('libp2p-secio')

import Hopr from '..'
import HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import { Heartbeat as HeartbeatInteraction } from '../interactions/network/heartbeat'

import Heartbeat from './heartbeat'
import NetworkPeerStore from './peerStore'
import { Network } from './index'

import assert from 'assert'
import Multiaddr from 'multiaddr'

describe('check heartbeat mechanism', function () {
  async function generateNode(): Promise<Hopr<HoprCoreConnector>> {
    const node = (await libp2p.create({
      peerInfo: await PeerInfo.create(await PeerId.create({ keyType: 'secp256k1' })),
      modules: {
        transport: [TCP],
        streamMuxer: [MPLEX],
        connEncryption: [SECIO]
      }
    })) as Hopr<HoprCoreConnector>

    node.peerInfo.multiaddrs.add(Multiaddr('/ip4/0.0.0.0/tcp/0'))
    node.hangUp = async (id) => {}

    node.interactions = {
      network: {
        heartbeat: new HeartbeatInteraction(node)
      }
    } as Hopr<HoprCoreConnector>['interactions']

    node.network = new Network(node, node.interactions, {} as any)
    node.peerRouting.findPeer = (_: PeerId) => Promise.reject(Error('not implemented'))

    await node.start()

    return (node as unknown) as Hopr<HoprCoreConnector>
  }

  it('should initialise the heartbeat module and start the heartbeat functionality', async function () {
    const [Alice, Bob, Chris] = await Promise.all([
      /* prettier-ignore */
      generateNode(),
      generateNode(),
      generateNode()
    ])

    await new Promise((resolve) => setTimeout(resolve, 100))

    await Alice.dial(Bob.peerInfo)

    // Check whether our event listener is triggered by heartbeat interactions
    await Promise.all([
      new Promise(async (resolve) => {
        Bob.network.heartbeat.once('beat', (peerId: PeerId) => {
          assert(Alice.peerInfo.id.isEqual(peerId), `Incoming connection must come from Alice`)
          resolve()
        })
      }),
      Alice.interactions.network.heartbeat.interact(Bob.peerInfo.id)
    ])

    assert(
      !Chris.network.peerStore.has(Alice.peerInfo.id.toB58String()),
      `Chris should not know about Alice in the beginning.`
    )

    await Alice.dial(Chris.peerInfo)

    // Check that the internal state is as expected
    assert(Alice.network.peerStore.has(Chris.peerInfo.id.toB58String()), `Alice should know about Chris now.`)
    assert(Alice.network.peerStore.has(Bob.peerInfo.id.toB58String()), `Alice should know about Bob now.`)
    assert(Chris.network.peerStore.has(Alice.peerInfo.id.toB58String()), `Chris should know about Alice now.`)
    assert(Bob.network.peerStore.has(Alice.peerInfo.id.toB58String()), `Bob should know about Alice now.`)

    // Simulate a node failure
    await Chris.stop()

    for (let i = 0; i < Alice.network.peerStore.peers.length; i++) {
      Alice.network.peerStore.peers[i].lastSeen = 0
    }

    // Check whether a node failure gets detected
    await Alice.network.heartbeat.checkNodes()

    assert(!Alice.network.peerStore.has(Chris.peerInfo.id.toB58String()), `Alice should have removed Chris.`)

    await Promise.all([
      /* pretier-ignore */
      Alice.stop(),
      Bob.stop()
    ])
  })
})

describe('unit test heartbeat', () => {
  let heartbeat
  let hangUp = jest.fn(async () => {})
  let peers
  let interaction = {
    interact: jest.fn(() => {}),
  } as any

  beforeEach(() => {
    const empty = [][Symbol.iterator]()
    ;(peers = new NetworkPeerStore(empty)), (heartbeat = new Heartbeat(peers, interaction, hangUp))
  })

  it('check nodes is noop with empty store', async () => {
    await heartbeat.checkNodes()
    expect(hangUp.mock.calls.length).toBe(0)
    expect(interaction.interact.mock.calls.length).toBe(0)
  })

  it('check nodes is noop with only new peers', async () => {
    peers.push({ id: 'a', lastSeen: Date.now() })
    await heartbeat.checkNodes()
    expect(hangUp.mock.calls.length).toBe(0)
    expect(interaction.interact.mock.calls.length).toBe(0)
  })

  it('check nodes interacts with an old peer', async () => {
    peers.push({ id: '16Uiu2HAmShu5QQs3LKEXjzmnqcT8E3YqyxKtVTurWYp8caM5jYJw', lastSeen: 0 })
    await heartbeat.checkNodes()
    expect(hangUp.mock.calls.length).toBe(0)
    expect(interaction.interact.mock.calls.length).toBe(1)
  })
})
