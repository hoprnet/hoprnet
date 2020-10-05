import PeerInfo from 'peer-info'
import PeerId from 'peer-id'

// @ts-ignore
import libp2p = require('libp2p')
// @ts-ignore
import TCP = require('libp2p-tcp')
// @ts-ignore
import MPLEX = require('libp2p-mplex')
// @ts-ignore
import SECIO = require('libp2p-secio')

import Debug from 'debug'
import chalk from 'chalk'

import Hopr from '../..'
import HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import { Heartbeat, HEARTBEAT_TIMEOUT } from './heartbeat'

import assert from 'assert'
import Multiaddr from 'multiaddr'

import { EventEmitter } from 'events'
import { durations } from '@hoprnet/hopr-utils'

describe('check heartbeat mechanism', function () {
  async function generateNode(options?: { timeoutIntentionally?: boolean }): Promise<Hopr<HoprCoreConnector>> {
    const node = (await libp2p.create({
      peerInfo: await PeerInfo.create(await PeerId.create({ keyType: 'secp256k1' })),
      modules: {
        transport: [TCP],
        streamMuxer: [MPLEX],
        connEncryption: [SECIO],
      },
    })) as Hopr<HoprCoreConnector>

    node.peerInfo.multiaddrs.add(Multiaddr('/ip4/0.0.0.0/tcp/0'))

    await node.start()

    node.peerRouting.findPeer = (_: PeerId): Promise<never> => {
      return Promise.reject(Error('not implemented'))
    }

    node.interactions = {
      network: {
        heartbeat: new Heartbeat(node, options),
      },
    } as Hopr<HoprCoreConnector>['interactions']

    node.network = {
      heartbeat: new EventEmitter(),
    } as Hopr<HoprCoreConnector>['network']

    return (node as unknown) as Hopr<HoprCoreConnector>
  }

  it('should dispatch a heartbeat', async function () {
    const [Alice, Bob] = await Promise.all([
      /* prettier-ignore */
      generateNode(),
      generateNode(),
    ])

    await new Promise((resolve) => setTimeout(resolve, 100))

    await Alice.dial(Bob.peerInfo)

    await Promise.all([
      new Promise((resolve) => {
        Bob.network.heartbeat.once('beat', (peerId: PeerId) => {
          assert(peerId.isEqual(Alice.peerInfo.id), 'connection must come from Alice')
          resolve()
        })
      }),
      Alice.interactions.network.heartbeat.interact(Bob.peerInfo.id),
    ])

    await Promise.all([Alice.stop(), Bob.stop()])
  })

  it('should trigger a heartbeat timeout', async function () {
    const [Alice, Bob] = await Promise.all([
      /* prettier-ignore */
      generateNode(),
      generateNode({ timeoutIntentionally: true }),
    ])
    await new Promise((resolve) => setTimeout(resolve, 100))
    await Alice.dial(Bob.peerInfo)
    let errorThrown = false
    let before = Date.now()
    try {
      await Alice.interactions.network.heartbeat.interact(Bob.peerInfo.id)
    } catch (err) {
      errorThrown = true
    }

    assert(errorThrown && Date.now() - before >= HEARTBEAT_TIMEOUT, 'Should reach a timeout')

    await Promise.all([Alice.stop(), Bob.stop()])
  })
})
