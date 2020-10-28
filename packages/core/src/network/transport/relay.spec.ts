import libp2p from 'libp2p'
// @ts-ignore
import MPLEX = require('libp2p-mplex')
// @ts-ignore
import KadDHT = require('libp2p-kad-dht')
// @ts-ignore
import SECIO = require('libp2p-secio')
// @ts-ignore
import TCP from 'libp2p-tcp'

import { Handler, MultiaddrConnection } from 'libp2p'
import Multiaddr from 'multiaddr'
import pipe from 'it-pipe'
import Relay from './relay'
import { randomBytes } from 'crypto'
import { privKeyToPeerId } from '../../utils'
import { getAddress } from '../../test-utils'
import assert from 'assert'

const TEST_PROTOCOL = `/test/0.0.1`

const privKeys = [randomBytes(32), randomBytes(32), randomBytes(32)]

async function generateNode(options: {
  id: number
  ipv4?: boolean
  ipv6?: boolean
  connHandler?: (conn: MultiaddrConnection) => void
}): Promise<{ node: libp2p; relay: Relay }> {
  const peerId = await privKeyToPeerId(privKeys[options.id])
  let addresses = []

  if (options.ipv4) {
    addresses = [Multiaddr(`/ip4/127.0.0.1/tcp/${9490 + 2 * options.id}`).encapsulate(`/p2p/${peerId.toB58String()}`)]
  }

  if (options.ipv6) {
    addresses = [Multiaddr(`/ip6/::1/tcp/${9490 + 2 * options.id + 1}`).encapsulate(`/p2p/${peerId.toB58String()}`)]
  }

  const node = new libp2p({
    peerId,
    addresses: { listen: addresses },
    modules: {
      transport: [TCP],
      streamMuxer: [MPLEX],
      connEncryption: [SECIO],
      dht: KadDHT
    },
    config: {
      dht: {
        enabled: false
      },
      relay: {
        enabled: false
      },
      peerDiscovery: {
        autoDial: false
      }
    }
  })

  let relay = new Relay(node, options.connHandler)

  node.handle([TEST_PROTOCOL], (handler: Handler) => {
    pipe(
      handler.stream,
      // echoing msg
      handler.stream
    )
  })

  await node.start()

  return {
    node,
    relay
  }
}

describe('should create a socket and connect to it', function () {
  it('should create a node and echo a single message', async function () {
    let [sender, relayer, counterparty] = await Promise.all([
      generateNode({ id: 0, ipv4: true }),
      generateNode({ id: 1, ipv4: true }),
      generateNode({
        id: 2,
        ipv4: true,
        connHandler: (conn: MultiaddrConnection) => {
          pipe(
            conn,
            (source: AsyncIterable<Uint8Array>) => {
              return (async function* () {
                for await (const msg of source) {
                  console.log(`echoing <${new TextDecoder().decode(msg.slice())}>`)
                  yield msg
                }
              })()
            },
            conn
          )
        }
      })
    ])

    // Make sure that the nodes know each other
    await Promise.all([sender.node.dial(getAddress(relayer.node)), counterparty.node.dial(getAddress(relayer.node))])

    let conn = await sender.relay.establishRelayedConnection(
      Multiaddr(`/p2p/${counterparty.node.peerId.toB58String()}`),
      relayer.node.multiaddrs
    )

    await pipe(
      (async function* () {
        yield new TextEncoder().encode(`first message`)

        await new Promise((resolve) => setTimeout(resolve, 100))

        yield new TextEncoder().encode(`second message`)
      })(),
      conn,
      async (source: AsyncIterable<Uint8Array>) => {
        setTimeout(() => setImmediate(() => conn.close()), 300)
        for await (const msg of source) {
          console.log(`received <${new TextDecoder().decode(msg.slice())}>`)
        }
      }
    )

    await counterparty.node.stop()

    await new Promise((resolve) => setTimeout(resolve, 200))

    counterparty = await generateNode({
      id: 2,
      ipv4: true,
      connHandler: (conn: MultiaddrConnection) => {
        pipe(
          conn,
          (source: AsyncIterable<Uint8Array>) => {
            return (async function* () {
              for await (const msg of source) {
                console.log(`echoing <${new TextDecoder().decode(msg.slice())}>`)
                yield msg
              }
            })()
          },
          conn
        )
      }
    })

    await counterparty.node.dial(getAddress(relayer.node))
    conn = await sender.relay.establishRelayedConnection(
      Multiaddr(`/p2p/${counterparty.node.peerId.toB58String()}`),
      relayer.node.multiaddrs
    )

    await pipe(
      (async function* () {
        yield new TextEncoder().encode(`first message`)
        await new Promise((resolve) => setTimeout(resolve, 100))
        yield new TextEncoder().encode(`second message`)
      })(),
      conn,
      async (source: AsyncIterable<Uint8Array>) => {
        setTimeout(() => setImmediate(() => conn.close()), 300)
        for await (const msg of source) {
          console.log(`received <${new TextDecoder().decode(msg.slice())}>`)
        }
      }
    )

    await Promise.all([sender, relayer, counterparty].map((x) => x.node.stop()))
    await new Promise((resolve) => setTimeout(resolve, 200))
  })

  it('should create a node and exchange messages', async function () {
    // let testMessagesEchoed = false
    // let testMessagesReplied = false
    // let thirdBatchEchoed = false
    // let waitingForSecondBatch = defer<void>()
    // let [sender, relay, counterparty] = await Promise.all([
    //   generateNode({ id: 0, ipv4: true }),
    //   generateNode({ id: 1, ipv4: true }),
    //   generateNode({
    //     id: 2,
    //     ipv4: true,
    //     connHandler: (handler: Handler & { counterparty: PeerId }) => {
    //       pipe(
    //         handler.stream,
    //         (source: AsyncIterable<Uint8Array>) => {
    //           return (async function* () {
    //             for (let i = 0; i < 2; i++) {
    //               let msg = (await source[Symbol.asyncIterator]().next()).value?.slice()
    //               assert(msg[0] == i + 1, 'Counterparty must receive all test messages in the right order')
    //               yield msg
    //             }
    //             testMessagesEchoed = true
    //             await new Promise((resolve) => setTimeout(resolve, 100))
    //             yield new Uint8Array([3])
    //             yield new Uint8Array([4])
    //             for (let i = 0; i < 2; i++) {
    //               let msg = (await source[Symbol.asyncIterator]().next()).value?.slice()
    //               assert(msg[0] == i + 5, 'Counterparty must receive all test messages in the right order')
    //             }
    //             thirdBatchEchoed = true
    //             waitingForSecondBatch.resolve()
    //           })()
    //         },
    //         handler.stream
    //       )
    //     },
    //   }),
    // ])
    // // Make sure that the nodes know each other
    // await Promise.all([sender.dial(relay.peerInfo), counterparty.dial(relay.peerInfo)])
    // const { stream } = await sender.relay.establishRelayedConnection(
    //   Multiaddr(`/p2p/${counterparty.peerInfo.id.toB58String()}`),
    //   [relay.peerInfo]
    // )
    // await pipe(
    //   (async function * () {
    //     yield new Uint8Array([1])
    //     yield new Uint8Array([2])
    //   })(),
    //   stream,
    //   async (source: AsyncIterable<Uint8Array>) => {
    //     for (let i = 0; i < 2; i++) {
    //       let msg = (await source[Symbol.asyncIterator]().next()).value?.slice()
    //       assert(msg[0] == i + 1, 'Sender must receive echoed messages')
    //     }
    //     testMessagesReplied = true
    //   }
    // )
    // await sender.stop()
    // const waiting = defer()
    // sender = await generateNode({
    //   id: 0,
    //   ipv4: true,
    //   connHandler: async (handler: Handler & { counterparty: PeerId }) => {
    //     pipe(
    //       handler.stream,
    //       async (source: AsyncIterable<Uint8Array>) => {
    //         for (let i = 0; i < 2; i++) {
    //           let msg = (await source[Symbol.asyncIterator]().next()).value?.slice()
    //           assert(msg[0] == i + 3, `Sender must receive all test messages in the right order.`)
    //         }
    //         waiting.resolve()
    //       }
    //     )
    //     await new Promise((resolve) => setTimeout(resolve, 100))
    //     pipe(
    //       (async function* () {
    //         yield new Uint8Array([5])
    //         yield new Uint8Array([6])
    //       })(),
    //       handler.stream
    //     )
    //   },
    // })
    // await sender.dial(relay.peerInfo)
    // await Promise.all([waiting.promise, waitingForSecondBatch.promise])
    // assert(testMessagesEchoed && testMessagesReplied && thirdBatchEchoed)
    // await Promise.all([sender.stop(), relay.stop(), counterparty.stop()])
  })

  it('should create a node and exchange messages', async function () {
    // let i = 1
    // let firstBatchEchoed = false
    // let secondBatchEchoed = false
    // let thirdBatchEchoed = false
    // let messagesReceived = false
    // const waiting = defer<void>()
    // let [sender, relay, counterparty] = await Promise.all([
    //   generateNode({ id: 0, ipv4: true }),
    //   generateNode({ id: 1, ipv4: true }),
    //   generateNode({
    //     id: 2,
    //     ipv4: true,
    //     connHandler: (handler: Handler & { counterparty: PeerId }) => {
    //       pipe(
    //         handler.stream,
    //         (source: AsyncIterable<Uint8Array>) => {
    //           return (async function* () {
    //             let i = 1
    //             for await (const msg of source) {
    //               if (u8aEquals(msg.slice(), new Uint8Array([1]))) {
    //                 i++
    //               } else if (i == 2 && u8aEquals(msg.slice(), new Uint8Array([2]))) {
    //                 firstBatchEchoed = true
    //               }
    //               yield msg
    //             }
    //           })()
    //         },
    //         handler.stream
    //       )
    //     },
    //   }),
    // ])
    // await Promise.all([sender.dial(relay.peerInfo), counterparty.dial(relay.peerInfo)])
    // const { stream } = await sender.relay.establishRelayedConnection(
    //   Multiaddr(`/p2p/${counterparty.peerInfo.id.toB58String()}`),
    //   [relay.peerInfo]
    // )
    // pipe(
    //   (async function* () {
    //     yield new Uint8Array([i++])
    //     await new Promise(resolve => setTimeout(resolve, 500))
    //     yield new Uint8Array([i++])
    //     await new Promise(resolve => setTimeout(resolve, 500))
    //     // yield new Uint8Array([i++])
    //     await counterparty.stop()
    //     counterparty = await generateNode({
    //       id: 2,
    //       ipv4: true,
    //       connHandler: (handler: Handler & { counterparty: PeerId }) => {
    //         pipe(
    //           handler.stream,
    //           (source: AsyncIterable<Uint8Array>) => {
    //             return (async function * () {
    //               let i = 1
    //               for await (const msg of source) {
    //                 if (u8aEquals(msg.slice(), new Uint8Array([3]))) {
    //                   i++
    //                 } else if (i == 2 && u8aEquals(msg.slice(), new Uint8Array([4]))) {
    //                   secondBatchEchoed = true
    //                 }
    //                 yield msg
    //               }
    //             })()
    //           },
    //           handler.stream
    //         )
    //       },
    //     })
    //     await counterparty.dial(relay.peerInfo)
    //     yield new Uint8Array([i++])
    //     yield new Uint8Array([i++])
    //     await new Promise(resolve => setTimeout(resolve, 500))
    //     await counterparty.stop()
    //     counterparty = await generateNode({
    //       id: 2,
    //       ipv4: true,
    //       connHandler: (handler: Handler & { counterparty: PeerId }) => {
    //         pipe(
    //           handler.stream,
    //           (source: AsyncIterable<Uint8Array>) => {
    //             return (async function * () {
    //               let i = 1
    //               for await (const msg of source) {
    //                 if (u8aEquals(msg.slice(), new Uint8Array([5]))) {
    //                   i++
    //                 } else if (i == 2 && u8aEquals(msg.slice(), new Uint8Array([6]))) {
    //                   thirdBatchEchoed = true
    //                 }
    //                 yield msg
    //               }
    //             })()
    //           },
    //           handler.stream
    //         )
    //       },
    //     })
    //     await counterparty.dial(relay.peerInfo)
    //     yield new Uint8Array([i++])
    //     yield new Uint8Array([i++])
    //     await new Promise(resolve => setTimeout(resolve, 500))
    //     return
    //   })(),
    //   stream,
    //   async (source: AsyncIterable<Uint8Array>) => {
    //     let i = 1
    //     for await (const msg of source) {
    //       if (u8aEquals(msg.slice(), new Uint8Array([1]))) {
    //         i++
    //       } else if (i == 2 && u8aEquals(msg.slice(), new Uint8Array([2]))) {
    //         i++
    //       } else if (i == 3 && u8aEquals(msg.slice(), new Uint8Array([3]))) {
    //         i++
    //       } else if (i == 4 && u8aEquals(msg.slice(), new Uint8Array([4]))) {
    //         i++
    //       } else if (i == 5 && u8aEquals(msg.slice(), new Uint8Array([5]))) {
    //         i++
    //       } else if (i == 6 && u8aEquals(msg.slice(), new Uint8Array([6]))) {
    //         messagesReceived = true
    //         waiting.resolve()
    //       }
    //     }
    //   }
    // )
    // await waiting.promise
    // assert(
    //   firstBatchEchoed && secondBatchEchoed && thirdBatchEchoed,
    //   'restarted counterparty must echo all messages that are sent to it.'
    // )
    // assert(messagesReceived, 'senders must receive all echoed messages - even if the counterparty went offline')
    // await Promise.all([sender.stop(), relay.stop(), counterparty.stop()])
  })

  it(`should connect to unknown nodes using DHT queries`, async function () {
    // let [sender, relay, counterparty] = await Promise.all([
    //   generateNode({ id: 0, ipv4: true }),
    //   generateNode({ id: 1, ipv4: true }),
    //   generateNode({ id: 2, ipv4: true }),
    // ])
    // let relayQueried = false
    // let counterpartyQueried = false
    // const findPeer = async (id: PeerId): Promise<PeerInfo> => {
    //   if (id.equals(sender.peerInfo.id)) {
    //     return Promise.resolve(sender.peerInfo)
    //   } else if (id.equals(relay.peerInfo.id)) {
    //     relayQueried = true
    //     return Promise.resolve(relay.peerInfo)
    //   } else if (id.equals(counterparty.peerInfo.id)) {
    //     counterpartyQueried = true
    //     return Promise.resolve(counterparty.peerInfo)
    //   } else {
    //     throw Error(`unknonw node`)
    //   }
    // }
    // sender.relay._dht = { peerRouting: { findPeer } }
    // relay.relay._dht = { peerRouting: { findPeer } }
    // await sender.relay.establishRelayedConnection(Multiaddr(`/p2p/${counterparty.peerInfo.id.toB58String()}`), [
    //   new PeerInfo(relay.peerInfo.id),
    // ])
    // assert(relayQueried, `Sender must have queried DHT for relay node`)
    // assert(counterpartyQueried, `Relay node must have queried DHT for counterparty`)
    // await Promise.all([sender.stop(), relay.stop(), counterparty.stop()])
  })

  it('should not use itself as relay node', async function () {
    let [sender, counterparty] = await Promise.all([
      generateNode({ id: 0, ipv4: true }),
      generateNode({ id: 2, ipv4: true })
    ])
    let errThrown = false
    try {
      await sender.relay.establishRelayedConnection(
        Multiaddr(`/p2p/${counterparty.node.peerId.toB58String()}`),
        sender.node.multiaddrs
      )
    } catch (err) {
      errThrown = true
    }
    assert(errThrown, `Must throw an error if there is no other opportunity than calling ourself`)
  })
})
