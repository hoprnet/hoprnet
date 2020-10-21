import PeerInfo from 'peer-info'
import PeerId from 'peer-id'
import libp2p from 'libp2p'
// @ts-ignore
import TCP = require('libp2p-tcp')
// @ts-ignore
import MPLEX = require('libp2p-mplex')
// @ts-ignore
import SECIO = require('libp2p-secio')
import { Heartbeat } from './heartbeat'
import assert from 'assert'
import Multiaddr from 'multiaddr'
import { EventEmitter } from 'events'
import * as constants from '../../constants';

// @ts-ignore
constants.HEARTBEAT_TIMEOUT = 300

describe('check heartbeat mechanism', function () {
  async function generateNode(options?: { timeoutIntentionally?: boolean }) {
    const node = await libp2p.create({
      peerInfo: await PeerInfo.create(await PeerId.create({ keyType: 'secp256k1' })),
      modules: {
        transport: [TCP],
        streamMuxer: [MPLEX],
        connEncryption: [SECIO]
      }
    })
    node.peerInfo.multiaddrs.add(Multiaddr('/ip4/0.0.0.0/tcp/0'))
    node.peerRouting.findPeer = (_: PeerId) => Promise.reject(Error('not implemented'))

    await node.start()

    node.interactions = {
      network: {
        heartbeat: new Heartbeat(node, (remotePeer) => node.network.heartbeat.emit('beat', remotePeer),  options)
      }
    }

    node.network = {
      heartbeat: new EventEmitter()
    }

    return node
  }

  it('should dispatch a heartbeat', async function () {
    const [Alice, Bob] = await Promise.all([generateNode(), generateNode()])

    await Alice.dial(Bob.peerInfo)

    await Promise.all([
      new Promise((resolve) => {
        Bob.network.heartbeat.once('beat', (peerId: PeerId) => {
          assert(peerId.isEqual(Alice.peerInfo.id), 'connection must come from Alice')
          resolve()
        })
      }),
      Alice.interactions.network.heartbeat.interact(Bob.peerInfo.id)
    ])

    await Promise.all([Alice.stop(), Bob.stop()])
  })

  it('should trigger a heartbeat timeout', async function () {
    const [Alice, Bob] = await Promise.all([generateNode(), generateNode({ timeoutIntentionally: true })])

    await Alice.dial(Bob.peerInfo)
    let errorThrown = false
    let before = Date.now()
    try {
      await Alice.interactions.network.heartbeat.interact(Bob.peerInfo.id)
    } catch (err) {
      errorThrown = true
    }

    assert(errorThrown, 'Should throw an error')
    assert(Date.now() - before >= constants.HEARTBEAT_TIMEOUT, `Should reach a timeout, ${Date.now() - before} ${constants.HEARTBEAT_TIMEOUT}`)

    await Promise.all([Alice.stop(), Bob.stop()])
  })
})
