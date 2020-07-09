// @ts-ignore
import libp2p = require('libp2p')

// @ts-ignore
import MPLEX = require('libp2p-mplex')
// @ts-ignore
import KadDHT = require('libp2p-kad-dht')
// @ts-ignore
import SECIO = require('libp2p-secio')
// @ts-ignore
import TCP from 'libp2p-tcp'

import PeerId from 'peer-id'

import { Handler } from './types'

import Multiaddr from 'multiaddr'
import PeerInfo from 'peer-info'
import pipe from 'it-pipe'

import Relay from './relay'
import { randomBytes } from 'crypto'

import { privKeyToPeerId } from '../../utils'
import { durations, u8aEquals } from '@hoprnet/hopr-utils'
import { assert } from 'console'

const TEST_PROTOCOL = `/test/0.0.1`

const privKeys = [randomBytes(32), randomBytes(32), randomBytes(32)]

describe('should create a socket and connect to it', function () {
  async function generateNode(options: {
    id: number
    ipv4?: boolean
    ipv6?: boolean
    connHandler?: (conn: Handler & { counterparty: PeerId }) => void
  }): Promise<libp2p> {
    const peerInfo = new PeerInfo(await privKeyToPeerId(privKeys[options.id]))

    if (options.ipv4) {
      peerInfo.multiaddrs.add(
        Multiaddr(`/ip4/127.0.0.1/tcp/${9090 + 2 * options.id}`).encapsulate(`/p2p/${peerInfo.id.toB58String()}`)
      )
    }

    if (options.ipv6) {
      peerInfo.multiaddrs.add(
        Multiaddr(`/ip6/::1/tcp/${9090 + 2 * options.id + 1}`).encapsulate(`/p2p/${peerInfo.id.toB58String()}`)
      )
    }

    const node = new libp2p({
      peerInfo,
      modules: {
        transport: [TCP],
        streamMuxer: [MPLEX],
        connEncryption: [SECIO],
        dht: KadDHT,
      },
      config: {
        dht: {
          enabled: false,
        },
        relay: {
          enabled: false,
        },
        peerDiscovery: {
          autoDial: false,
        },
      },
    })

    node.relay = new Relay(node, options.connHandler)

    node.handle(TEST_PROTOCOL, (handler: Handler) => {
      pipe(
        /* prettier-ignore */
        handler.stream,
        // echoing msg
        handler.stream
      )
    })

    await node.start()

    return node
  }

  it('should create a node and exchange messages', async function () {
    this.timeout(durations.seconds(5))

    let i = 1

    let firstBatchEchoed = false
    let secondBatchEchoed = false
    let thirdBatchEchoed = false

    let messagesReceived = false

    let [sender, relay, counterparty] = await Promise.all([
      generateNode({ id: 0, ipv4: true }),
      generateNode({ id: 1, ipv4: true }),
      generateNode({
        id: 2,
        ipv4: true,
        connHandler: (handler: Handler & { counterparty: PeerId }) => {
          pipe(
            /* prettier-ignore */
            handler.stream,
            (source: AsyncIterable<Uint8Array>) => {
              return (async function* () {
                let i = 1
                for await (const msg of source) {
                  // console.log(`echoing 0`, msg)
                  if (u8aEquals(msg.slice(), new Uint8Array([1]))) {
                    i++
                  } else if (i == 2 && u8aEquals(msg.slice(), new Uint8Array([2]))) {
                    firstBatchEchoed = true
                  }

                  yield msg
                }
              })()
            },
            handler.stream
          )
        },
      }),
    ])

    await Promise.all([sender.dial(relay.peerInfo), counterparty.dial(relay.peerInfo)])

    const { stream } = await sender.relay.establishRelayedConnection(
      Multiaddr(`/p2p/${counterparty.peerInfo.id.toB58String()}`),
      [relay.peerInfo]
    )

    const pipePromise = pipe(
      // prettier-ignore
      (async function* () {
        yield new Uint8Array([i++])

        await new Promise(resolve => setTimeout(resolve, 500))

        yield new Uint8Array([i++])

        await new Promise(resolve => setTimeout(resolve, 500))

        // yield new Uint8Array([i++])

        await counterparty.stop()

        counterparty = await generateNode({
          id: 2,
          ipv4: true,
          connHandler: (handler: Handler & { counterparty: PeerId }) => {
            pipe(
              /* prettier-ignore */
              handler.stream,
              (source: AsyncIterable<Uint8Array>) => {
                return (async function * () {
                  let i = 1
                  for await (const msg of source) {
                    console.log(`echoing 1st`, msg)
                    if (u8aEquals(msg.slice(), new Uint8Array([3]))) {
                      i++
                    } else if (i == 2 && u8aEquals(msg.slice(), new Uint8Array([4]))) {
                      secondBatchEchoed = true
                    }
                    
                    yield msg
                  }
                })()
              },
              handler.stream
            )
          },
        })

        await counterparty.dial(relay.peerInfo)

        yield new Uint8Array([i++])
        yield new Uint8Array([i++])

        await new Promise(resolve => setTimeout(resolve, 500))

        // yield new Uint8Array([i++])

        await counterparty.stop()

        counterparty = await generateNode({
          id: 2,
          ipv4: true,
          connHandler: (handler: Handler & { counterparty: PeerId }) => {
            pipe(
              /* prettier-ignore */
              handler.stream,
              (source: AsyncIterable<Uint8Array>) => {
                return (async function * () {
                  let i = 1
                  for await (const msg of source) {
                    if (u8aEquals(msg.slice(), new Uint8Array([5]))) {
                      i++
                    } else if (i == 2 && u8aEquals(msg.slice(), new Uint8Array([6]))) {
                      thirdBatchEchoed = true
                    }
                    console.log(`echoing 2nd`, msg)
                    yield msg
                  }
                })()
              },
              handler.stream
            )
          },
        })

        await counterparty.dial(relay.peerInfo)

        yield new Uint8Array([i++])

        await new Promise(resolve => setTimeout(resolve, 500))

        yield new Uint8Array([i++])
        return
      })(),
      stream,
      async (source: AsyncIterable<Uint8Array>) => {
        let i = 1
        for await (const msg of source) {
          if (u8aEquals(msg.slice(), new Uint8Array([1]))) {
            i++
          } else if (i == 2 && u8aEquals(msg.slice(), new Uint8Array([2]))) {
            i++
          } else if (i == 3 && u8aEquals(msg.slice(), new Uint8Array([3]))) {
            i++
          } else if (i == 4 && u8aEquals(msg.slice(), new Uint8Array([4]))) {
            i++
          } else if (i == 5 && u8aEquals(msg.slice(), new Uint8Array([5]))) {
            i++
          } else if (i == 6 && u8aEquals(msg.slice(), new Uint8Array([6]))) {
            messagesReceived = true
          }
        }
      }
    )

    await new Promise((resolve) => setTimeout(resolve, durations.seconds(4)))

    assert(
      firstBatchEchoed && secondBatchEchoed && thirdBatchEchoed,
      'restarted counterparty must echo all messages that are sent to it.'
    )

    assert(messagesReceived, 'senders must receive all echoed messages - even if the counterparty went offline')

    await Promise.all([sender.stop(), relay.stop(), counterparty.stop()])
  })
})

/**
 * Informs each node about the others existence.
 * @param nodes Hopr nodes
 */
function connectionHelper(nodes: libp2p[]) {
  for (let i = 0; i < nodes.length; i++) {
    for (let j = i + 1; j < nodes.length; j++) {
      nodes[i].peerStore.put(nodes[j].peerInfo)
      nodes[j].peerStore.put(nodes[i].peerInfo)
    }
  }
}
