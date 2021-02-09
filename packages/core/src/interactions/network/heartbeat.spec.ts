import PeerId from 'peer-id'

import { Heartbeat } from './heartbeat'
import assert from 'assert'
import { EventEmitter } from 'events'
import * as constants from '../../constants'
import { generateLibP2PMock } from '../../test-utils'

// @ts-ignore
constants.HEARTBEAT_TIMEOUT = 300

describe('check heartbeat mechanism', function () {
  async function generateNode(options?: { timeoutIntentionally?: boolean }) {
    const { node, address } = await generateLibP2PMock()

    node.peerRouting.findPeer = (_: PeerId) => Promise.reject(Error('not implemented'))

    await node.start()

    const network = {
      heartbeat: new EventEmitter()
    }

    const interactions = {
      network: {
        heartbeat: new Heartbeat(node, (remotePeer) => network.heartbeat.emit('beat', remotePeer), options)
      }
    }

    return {
      node,
      network,
      interactions,
      address
    }
  }

  it('should dispatch a heartbeat', async function () {
    const [Alice, Bob] = await Promise.all([generateNode(), generateNode()])

    await Alice.node.dial(Bob.address)

    await Promise.all([
      new Promise<void>((resolve) => {
        Bob.network.heartbeat.once('beat', (peerId: PeerId) => {
          assert(peerId.equals(Alice.node.peerId), 'connection must come from Alice')
          resolve()
        })
      }),
      Alice.interactions.network.heartbeat.interact(Bob.node.peerId)
    ])

    await Promise.all([Alice.node.stop(), Bob.node.stop()])
  })

  it('should trigger a heartbeat timeout', async function () {
    const [Alice, Bob] = await Promise.all([generateNode(), generateNode({ timeoutIntentionally: true })])

    await Alice.node.dial(Bob.address)
    let before = Date.now()

    assert((await Alice.interactions.network.heartbeat.interact(Bob.node.peerId)) < 0, 'Ping must fail')

    assert(
      Date.now() - before >= constants.HEARTBEAT_TIMEOUT,
      `Should reach a timeout, ${Date.now() - before} ${constants.HEARTBEAT_TIMEOUT}`
    )

    await Promise.all([Alice.node.stop(), Bob.node.stop()])
  })
})
