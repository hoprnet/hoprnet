import assert from 'assert'
import libp2p from 'libp2p'

// @ts-ignore
import MPLEX = require('libp2p-mplex')
// @ts-ignore
import KadDHT = require('libp2p-kad-dht')
// @ts-ignore
import SECIO = require('libp2p-secio')

import PeerId from 'peer-id'

import { Handler } from 'libp2p'

import TCP from '.'
import Multiaddr from 'multiaddr'
import pipe from 'it-pipe'

import { u8aEquals } from '@hoprnet/hopr-utils'

import { randomBytes } from 'crypto'
import { RELAY_CIRCUIT_TIMEOUT } from './constants'
import { connectionHelper } from '../../test-utils'

const TEST_PROTOCOL = `/test/0.0.1`

describe('should create a socket and connect to it', function () {
  jest.setTimeout(RELAY_CIRCUIT_TIMEOUT * 3)

  async function generateNode(
    options: {
      id: number
      ipv4?: boolean
      ipv6?: boolean
      useWebRTC?: boolean
      startNode?: boolean
      failIntentionallyOnWebRTC?: boolean
      timeoutIntentionallyOnWebRTC?: Promise<void>
      answerIntentionallyWithIncorrectMessages?: boolean
    },
    bootstrap?: Multiaddr 
  ): Promise<libp2p> {
    const peerId = await PeerId.create({ keyType: 'secp256k1' })
    const addresses = []

    if (options.ipv4) {
      addresses.push(
        Multiaddr(`/ip4/127.0.0.1/tcp/${9090 + 2 * options.id}`).encapsulate(`/p2p/${peerId.toB58String()}`)
      )
    }

    if (options.ipv6) {
      addresses.push(
        Multiaddr(`/ip6/::1/tcp/${9090 + 2 * options.id + 1}`).encapsulate(`/p2p/${peerId.toB58String()}`)
      )
    }

    const node = new libp2p({
      peerId, addresses: {listen: addresses},
      modules: {
        transport: [TCP],
        streamMuxer: [MPLEX],
        connEncryption: [SECIO],
        dht: KadDHT
      },
      config: {
        transport: {
          TCP: {
            useWebRTC: options.useWebRTC,
            bootstrapServers: [bootstrap],
            failIntentionallyOnWebRTC: options.failIntentionallyOnWebRTC,
            timeoutIntentionallyOnWebRTC: options.timeoutIntentionallyOnWebRTC,
            answerIntentionallyWithIncorrectMessages: options.answerIntentionallyWithIncorrectMessages
          }
        },
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

    node.handle([TEST_PROTOCOL], (handler: Handler) => {
      pipe(
        handler.stream,
        // echoing msg
        handler.stream
      )
    })

    await node.start()

    return node
  }

  // it('should establish a direct connection between two nodes', async function () {
  //   const [sender, counterparty] = await Promise.all([
  //     generateNode({ id: 0, ipv4: true, useWebRTC: false }),
  //     generateNode({ id: 1, ipv4: true, useWebRTC: false }),
  //   ])

  //   const { stream }: { stream: Stream } = await sender.dialProtocol(
  //     Multiaddr(`/ip4/127.0.0.1/tcp/9092/p2p/${counterparty.peerInfo.id.toB58String()}`),
  //     TEST_PROTOCOL
  //   )

  //   let msgReceived = false
  //   const testMessage = randomBytes(123)

  //   await pipe(
  //     [testMessage],
  //     stream,
  //     async (source: AsyncIterable<Uint8Array>) => {
  //       for await (const msg of source) {
  //         assert(u8aEquals(msg.slice(), testMessage), 'sent message and received message must be identical')
  //         msgReceived = true
  //         return
  //       }
  //     }
  //   )

  //   assert(msgReceived, `Message must be received by counterparty.`)

  //   await Promise.all([
  //     sender.hangUp(new PeerInfo(counterparty.peerInfo.id)),
  //     counterparty.hangUp(new PeerInfo(sender.peerInfo.id)),
  //   ])

  //   // Try with abort controller
  //   // const abort = new AbortController()

  //   // setTimeout(() => abort.abort(), 300)

  //   // try {
  //   //   await sender.dialProtocol(
  //   //     Multiaddr(`/ip4/127.0.0.1/tcp/9092/p2p/${counterparty.peerInfo.id.toB58String()}`),
  //   //     TEST_PROTOCOL,
  //   //     { signal: abort.signal }
  //   //   )
  //   // } catch (err) {
  //   //   if (err.type !== 'aborted') {
  //   //     throw err
  //   //   }
  //   // }

  //   // await Promise.all([
  //   //   sender.hangUp(new PeerInfo(counterparty.peerInfo.id)),
  //   //   counterparty.hangUp(new PeerInfo(sender.peerInfo.id)),
  //   // ])

  //   await Promise.all([
  //     sender.stop(),
  //     counterparty.stop(),
  //   ])
  // })

  // it('should establish a direct connection between two nodes over IPv6', async function () {
  //   const [sender, counterparty] = await Promise.all([
  //     generateNode({ id: 0, ipv6: true, useWebRTC: false }),
  //     generateNode({ id: 1, ipv6: true, useWebRTC: false }),
  //   ])

  //   const { stream }: { stream: Stream } = await sender.dialProtocol(
  //     Multiaddr(`/ip6/::1/tcp/9093/p2p/${counterparty.peerInfo.id.toB58String()}`),
  //     TEST_PROTOCOL
  //   )

  //   let msgReceived = false
  //   const testMessage = randomBytes(123)

  //   await pipe(
  //     [testMessage],
  //     stream,
  //     async (source: AsyncIterable<Uint8Array>) => {
  //       for await (const msg of source) {
  //         assert(u8aEquals(msg.slice(), testMessage), 'sent message and received message must be identical')
  //         msgReceived = true
  //         return
  //       }
  //     }
  //   )

  //   assert(msgReceived, `Message must be received by counterparty.`)

  //   // await Promise.all([
  //   //   sender.hangUp(new PeerInfo(counterparty.peerInfo.id)),
  //   //   counterparty.hangUp(new PeerInfo(sender.peerInfo.id)),
  //   // ])

  //   // Try with abort controller
  //   // const abort = new AbortController()

  //   // setTimeout(() => {
  //   //   setImmediate(() => abort.abort())
  //   // }, 300)

  //   // try {
  //   //   await sender.dialProtocol(
  //   //     Multiaddr(`/ip6/::1/tcp/9093/p2p/${counterparty.peerInfo.id.toB58String()}`),
  //   //     TEST_PROTOCOL,
  //   //     { signal: abort.signal }
  //   //   )
  //   // } catch (err) {
  //   //   if (err.type !== 'aborted') {
  //   //     throw err
  //   //   }
  //   // }

  //   // await Promise.all([
  //   //   sender.hangUp(new PeerInfo(counterparty.peerInfo.id)),
  //   //   counterparty.hangUp(new PeerInfo(sender.peerInfo.id)),
  //   // ])

  //   await Promise.all([sender.stop(), counterparty.stop()])
  // })

  // it('must not establish a connection to a non-existing node', async function () {
  //   const [sender, fakeCounterparty] = await Promise.all([
  //     generateNode({ id: 0, ipv4: true, useWebRTC: false }),
  //     privKeyToPeerId(randomBytes(32)),
  //   ])

  //   let errThrown = false
  //   try {
  //     await sender.dialProtocol(
  //       Multiaddr(`/ip4/127.0.0.1/tcp/9094/p2p/${fakeCounterparty.toB58String()}`),
  //       TEST_PROTOCOL
  //     )
  //   } catch (err) {
  //     errThrown = true
  //   }

  //   assert(errThrown, `Must throw error in case other node node is not reachable`)

  //   await Promise.all([sender.stop()])
  // })

  // it('must not establish a relayed connection to a non-existing node', async function () {
  //   jest.setTimeout(RELAY_CIRCUIT_TIMEOUT * 2)

  //   const relay = await generateNode({ id: 2, ipv4: true })

  //   const [sender, fakeCounterparty] = await Promise.all([
  //     generateNode({ id: 0, ipv4: true, useWebRTC: false }, relay.peerInfo),
  //     privKeyToPeerId(randomBytes(32)),
  //   ])

  //   connectionHelper([sender, relay])

  //   let errThrown = false
  //   const INVALID_PORT = 9999
  //   try {
  //     await sender.dialProtocol(
  //       Multiaddr(`/ip4/127.0.0.1/tcp/${INVALID_PORT}/p2p/${fakeCounterparty.toB58String()}`),
  //       TEST_PROTOCOL
  //     )
  //   } catch (err) {
  //     errThrown = true
  //   }

  //   assert(errThrown, `Must throw error in case other node node is not reachable`)

  //   await Promise.all([
  //     sender.stop(),
  //     relay.stop(),
  //   ])
  // })

  // it('must not establish a relayed connection to an offline node', async function () {
  //   const relay = await generateNode({ id: 2, ipv4: true })

  //   const [sender, offlineCounterparty] = await Promise.all([
  //     generateNode({ id: 0, ipv4: true, useWebRTC: false }, relay.peerInfo),
  //     generateNode({ id: 1, ipv4: true, useWebRTC: false }, relay.peerInfo),
  //   ])

  //   connectionHelper([sender, relay])
  //   connectionHelper([relay, offlineCounterparty])

  //   await offlineCounterparty.stop()

  //   let errThrown = false
  //   const INVALID_PORT = 9999

  //   const now = Date.now()
  //   try {
  //     await sender.dialProtocol(
  //       Multiaddr(`/ip4/127.0.0.1/tcp/${INVALID_PORT}/p2p/${offlineCounterparty.peerInfo.id.toB58String()}`),
  //       TEST_PROTOCOL
  //     )
  //   } catch (err) {
  //     errThrown = true
  //   }

  //   assert(errThrown, `Must throw error in case other node is not reachable`)

  //   await Promise.all([
  //     sender.stop(),
  //     relay.stop(),
  //   ])
  // })

  it('should set up a relayed connection and upgrade to WebRTC', async function () {
    const relay = await generateNode({ id: 2, ipv4: true })
    const [sender, counterparty] = await Promise.all([
      generateNode({ id: 0, ipv4: true }, relay.multiaddrs[0]),
      generateNode({ id: 1, ipv4: true }, relay.multiaddrs[0])
    ])
    connectionHelper([sender, relay])
    connectionHelper([relay, counterparty])
    const INVALID_PORT = 8758
    // @ts-ignore
    const { stream }: { stream: Connection } = await sender.dialProtocol(
      Multiaddr(`/ip4/127.0.0.1/tcp/${INVALID_PORT}/p2p/${counterparty.peerId.toB58String()}`),
      TEST_PROTOCOL
    )
    let msgReceived = false
    const testMessage = randomBytes(33)
    await pipe([testMessage], stream, async (source: AsyncIterable<Uint8Array>) => {
      for await (const msg of source) {
        assert(u8aEquals(msg.slice(), testMessage), 'sent message and received message must be identical')
        msgReceived = true
        return
      }
    })

    await new Promise((resolve) => setTimeout(resolve, 250))
    stream.close()
    assert(msgReceived, `msg must be received`)
    // await Promise.all([
    //   sender.hangUp(new PeerInfo(counterparty.peerInfo.id)),
    //   counterparty.hangUp(new PeerInfo(sender.peerInfo.id)),
    // ])
    // // Try with abort controller
    // const abort = new AbortController()
    // abort.abort()
    // try {
    //   await sender.dialProtocol(
    //     Multiaddr(`/ip4/127.0.0.1/tcp/${INVALID_PORT}/p2p/${counterparty.peerInfo.id.toB58String()}`),
    //     TEST_PROTOCOL,
    //     { signal: abort.signal }
    //   )
    // } catch {}
    // await Promise.all([
    //   sender.hangUp(new PeerInfo(counterparty.peerInfo.id)),
    //   counterparty.hangUp(new PeerInfo(sender.peerInfo.id)),
    // ])
    await Promise.all([sender.stop(), counterparty.stop(), relay.stop()])
  })

  // it('should set up a relayed connection and fail while upgrading to WebRTC', async function () {
  //   const relay = await generateNode({ id: 2, ipv4: true, ipv6: true })

  //   const [sender, counterparty] = await Promise.all([
  //     generateNode({ id: 0, ipv4: true, failIntentionallyOnWebRTC: true }, relay.peerInfo),
  //     generateNode({ id: 1, ipv6: true, failIntentionallyOnWebRTC: true }, relay.peerInfo),
  //   ])

  //   connectionHelper([sender, relay])
  //   connectionHelper([relay, counterparty])

  //   const INVALID_PORT = 8758
  //   const conn = await sender.dialProtocol(
  //     Multiaddr(`/ip4/127.0.0.1/tcp/${INVALID_PORT}/p2p/${counterparty.peerInfo.id.toB58String()}`),
  //     TEST_PROTOCOL
  //   )

  //   let msgReceived = false

  //   const testMessage = randomBytes(123)
  //   await pipe(
  //     [testMessage],
  //     conn.stream,
  //     async (source: AsyncIterable<Uint8Array>) => {
  //       for await (const msg of source) {
  //         assert(u8aEquals(msg.slice(), testMessage), 'sent message and received message must be identical')
  //         msgReceived = true
  //         return
  //       }
  //     }
  //   )

  //   assert(msgReceived, `message must be received`)

  //   await Promise.all([
  //     sender.stop(),
  //     counterparty.stop(),
  //     relay.stop(),
  //   ])
  // })

  // it(
  //   'should set up a relayed connection and fail while upgrading to WebRTC due to falsy messages',
  //   async function () {
  //     const relay = await generateNode({ id: 2, ipv4: true, ipv6: true })

  //     const [sender, counterparty] = await Promise.all([
  //       generateNode({ id: 0, ipv4: true }, relay.peerInfo),
  //       generateNode({ id: 1, ipv6: true, answerIntentionallyWithIncorrectMessages: true }, relay.peerInfo),
  //     ])

  //     connectionHelper([sender, relay])
  //     connectionHelper([relay, counterparty])

  //     const now = Date.now()
  //     const INVALID_PORT = 8758
  //     const conn = await sender.dialProtocol(
  //       Multiaddr(`/ip4/127.0.0.1/tcp/${INVALID_PORT}/p2p/${counterparty.peerInfo.id.toB58String()}`),
  //       TEST_PROTOCOL
  //     )

  //     assert(Date.now() - now >= WEBRTC_TIMEOUT, `Connection should not get established before WebRTC timeout.`)

  //     let msgReceived = false

  //     const testMessage = randomBytes(123)
  //     await pipe(
  //       [testMessage],
  //       conn.stream,
  //       async (source: AsyncIterable<Uint8Array>) => {
  //         for await (const msg of source) {
  //           assert(u8aEquals(msg.slice(), testMessage), 'sent message and received message must be identical')
  //           msgReceived = true
  //           return
  //         }
  //       }
  //     )

  //     assert(msgReceived, `message must be received`)

  //     await Promise.all([
  //       sender.stop(),
  //       counterparty.stop(),
  //       relay.stop(),
  //     ])
  //   },
  //   durations.seconds(20)
  // )

  // it('should set up a relayed connection and timeout while upgrading to WebRTC', async function () {
  //   const relay = await generateNode({ id: 2, ipv4: true, ipv6: true })

  //   const now = Date.now()

  //   const [sender, counterparty] = await Promise.all([
  //     generateNode(
  //       {
  //         id: 0,
  //         ipv4: true,
  //         timeoutIntentionallyOnWebRTC: new Promise((resolve) => setTimeout(resolve, WEBRTC_TIMEOUT)),
  //       },
  //       relay.peerInfo
  //     ),
  //     generateNode(
  //       {
  //         id: 1,
  //         ipv6: true,
  //         timeoutIntentionallyOnWebRTC: new Promise((resolve) => setTimeout(resolve, WEBRTC_TIMEOUT)),
  //       },
  //       relay.peerInfo
  //     ),
  //   ])

  //   connectionHelper([sender, relay])
  //   connectionHelper([relay, counterparty])

  //   const INVALID_PORT = 8758
  //   const conn = await sender.dialProtocol(
  //     Multiaddr(`/ip4/127.0.0.1/tcp/${INVALID_PORT}/p2p/${counterparty.peerInfo.id.toB58String()}`),
  //     TEST_PROTOCOL
  //   )

  //   assert(Date.now() - now >= WEBRTC_TIMEOUT, `Connection must not succeed before WebRTC timeout`)

  //   let msgReceived = false

  //   const testMessage = randomBytes(123)
  //   await pipe(
  //     [testMessage],
  //     conn.stream,
  //     async (source: AsyncIterable<Uint8Array>) => {
  //       for await (const msg of source) {
  //         assert(u8aEquals(msg.slice(), testMessage), 'sent message and received message must be identical')
  //         msgReceived = true
  //         return
  //       }
  //     }
  //   )

  //   assert(msgReceived, `message must be received`)

  //   await Promise.all([sender.stop(), counterparty.stop(), relay.stop()])
  // })

  // it('should set up a relayed connection and upgrade to WebRTC and keep connected even if the relay goes offline', async function () {
  //   const relay = await generateNode({ id: 2, ipv4: true, ipv6: true })

  //   const [sender, counterparty] = await Promise.all([
  //     generateNode({ id: 0, ipv4: true }, relay.peerInfo),
  //     generateNode({ id: 1, ipv6: true }, relay.peerInfo),
  //   ])

  //   connectionHelper([sender, relay])
  //   connectionHelper([relay, counterparty])

  //   const INVALID_PORT = 8758
  //   const conn = await sender.dialProtocol(
  //     Multiaddr(`/ip4/127.0.0.1/tcp/${INVALID_PORT}/p2p/${counterparty.peerInfo.id.toB58String()}`),
  //     TEST_PROTOCOL
  //   )

  //   let msgReceived = false
  //   const firstMessage = randomBytes(33)
  //   const secondMessage = randomBytes(1337)
  //   const thirdMessage = randomBytes(41)

  //   let relayStopped = false

  //   await pipe(
  //     (async function * () {
  //       yield firstMessage
  //       await new Promise<void>(resolve => setTimeout(resolve, 500)),
  //       yield secondMessage
  //       await new Promise<void>(resolve => setTimeout(resolve, 500)),
  //       yield thirdMessage
  //     })(),
  //     conn.stream,
  //     async (source: AsyncIterable<Uint8Array>) => {
  //       let index = 0
  //       for await (const msg of source) {
  //         if (index == 0) {
  //           assert(u8aEquals(msg.slice(), firstMessage), 'sent message and received message must be identical')
  //           index++
  //         } else if (index == 1) {
  //           assert(u8aEquals(msg.slice(), secondMessage), 'sent message and received message must be identical')
  //           index++
  //           setImmediate(() => {
  //             relay.stop.call(relay).then(() => {
  //               relayStopped = true
  //             })
  //           })
  //         } else if (index == 2) {
  //           assert(u8aEquals(msg.slice(), thirdMessage), 'sent message and received message must be identical')
  //           index++
  //           msgReceived = true
  //         }
  //       }
  //       return
  //     }
  //   )

  //   assert(relayStopped, `Relay node must have been shut down`)

  //   assert(msgReceived, `msg must be received`)

  //   await Promise.all([
  //     sender.hangUp(new PeerInfo(counterparty.peerInfo.id)),
  //     counterparty.hangUp(new PeerInfo(sender.peerInfo.id)),
  //   ])

  //   await Promise.all([
  //     sender.stop(),
  //     counterparty.stop(),
  //   ])
  // })

  // it('should set up a relayed connection with p2p Multiaddr and upgrade to WebRTC', async function () {
  //   const relay = await generateNode({ id: 2, ipv4: true, ipv6: true })

  //   const [sender, counterparty] = await Promise.all([
  //     generateNode({ id: 0, ipv4: true }, relay.peerInfo),
  //     generateNode({ id: 1, ipv6: true }, relay.peerInfo),
  //   ])

  //   connectionHelper([sender, relay])
  //   connectionHelper([relay, counterparty])

  //   const INVALID_PORT = 8758
  //   const conn = await sender.dialProtocol(Multiaddr(`/p2p/${counterparty.peerInfo.id.toB58String()}`), TEST_PROTOCOL)

  //   let msgReceived = false
  //   const testMessage = randomBytes(33)

  //   await pipe(
  //     [testMessage],
  //     conn.stream,
  //     async (source: AsyncIterable<Uint8Array>) => {
  //       for await (const msg of source) {
  //         assert(u8aEquals(msg.slice(), testMessage), 'sent message and received message must be identical')
  //         msgReceived = true
  //         return
  //       }
  //     }
  //   )

  //   assert(msgReceived, `msg must be received`)

  //   await Promise.all([
  //     sender.stop(),
  //     counterparty.stop(),
  //     relay.stop(),
  //   ])
  // })

  // it('should set up a relayed connection with p2p Multiaddr with sender that does not support WebRTC', async function () {
  //   jest.setTimeout(15 * 1000)
  //   const relay = await generateNode({ id: 2, ipv4: true, ipv6: true })

  //   const now = Date.now()

  //   const [sender, counterparty] = await Promise.all([
  //     generateNode({ id: 0, ipv4: true, useWebRTC: false }, relay.peerInfo),
  //     generateNode({ id: 1, ipv4: true, useWebRTC: true }, relay.peerInfo),
  //   ])

  //   connectionHelper([sender, relay])
  //   connectionHelper([relay, counterparty])

  //   const INVALID_PORT = 8758
  //   const { stream }: { stream: Stream } = await sender.dialProtocol(
  //     Multiaddr(`/p2p/${counterparty.peerInfo.id.toB58String()}`),
  //     TEST_PROTOCOL
  //   )

  //   let msgReceived = false
  //   const testMessage = randomBytes(33)

  //   await pipe(
  //     [testMessage],
  //     stream,
  //     async (source: AsyncIterable<Uint8Array>) => {
  //       for await (const msg of source) {
  //         assert(u8aEquals(msg.slice(), testMessage), 'sent message and received message must be identical')
  //         msgReceived = true
  //         return
  //       }
  //     }
  //   )

  //   assert(msgReceived, `msg must be received`)

  //   // assert(Date.now() - now >= WEBRTC_TIMEOUT, `Connection should not be established before WebRTC timeout.`)

  //   await Promise.all([
  //     sender.stop(),
  //     counterparty.stop(),
  //     relay.stop(),
  //   ])
  // })

  // it('should set up a relayed connection with p2p Multiaddr with counterparty that does not support WebRTC', async function () {
  //   const relay = await generateNode({ id: 2, ipv4: true, ipv6: true })

  //   const now = Date.now()

  //   const [sender, counterparty] = await Promise.all([
  //     generateNode({ id: 0, ipv4: true, useWebRTC: true }, relay.peerInfo),
  //     generateNode({ id: 1, ipv4: true, useWebRTC: false }, relay.peerInfo),
  //   ])

  //   connectionHelper([sender, relay])
  //   connectionHelper([relay, counterparty])

  //   const INVALID_PORT = 8758
  //   const conn = await sender.dialProtocol(Multiaddr(`/p2p/${counterparty.peerInfo.id.toB58String()}`), TEST_PROTOCOL)

  //   let msgReceived = false
  //   const testMessage = randomBytes(33)

  //   await pipe(
  //     [testMessage],
  //     conn.stream,
  //     async (source: AsyncIterable<Uint8Array>) => {
  //       for await (const msg of source) {
  //         assert(u8aEquals(msg.slice(), testMessage), 'sent message and received message must be identical')
  //         msgReceived = true
  //         return
  //       }
  //     }
  //   )

  //   assert(msgReceived, `msg must be received`)

  //   // assert(Date.now() - now >= WEBRTC_TIMEOUT, `Connection should not be established before WebRTC timeout.`)

  //   await Promise.all([
  //     sender.stop(),
  //     counterparty.stop(),
  //     relay.stop(),
  //   ])
  // })

  // it('should accept TCP addresses and p2p addresses', async function () {
  //   const node = await generateNode({ id: 0 })
  //   const TransportModule = new TCP({ upgrader: node.upgrader, libp2p: node })

  //   const p2pMultiaddr = Multiaddr(`/p2p/${node.peerInfo.id.toB58String()}`)
  //   let filteredMultiaddr = TransportModule.filter([p2pMultiaddr])
  //   assert(filteredMultiaddr.length == 1 && filteredMultiaddr[0].equals(p2pMultiaddr))

  //   const ip4Multiaddr = Multiaddr(`/ip4/0.0.0.0/tcp/0/p2p/${node.peerInfo.id.toB58String()}`)
  //   filteredMultiaddr = TransportModule.filter([ip4Multiaddr])
  //   assert(filteredMultiaddr.length == 1 && filteredMultiaddr[0].equals(ip4Multiaddr))

  //   const ip6Multiaddr = Multiaddr(`/ip6/::1/tcp/0/p2p/${node.peerInfo.id.toB58String()}`)
  //   filteredMultiaddr = TransportModule.filter([ip6Multiaddr])
  //   assert(filteredMultiaddr.length == 1 && filteredMultiaddr[0].equals(ip6Multiaddr))
  // })

  // it('should establish connections to the relay and then connect "directly" via a p2p address', async function () {
  //   const relay = await generateNode({ id: 2, ipv4: true, ipv6: true })

  //   const [sender, counterparty] = await Promise.all([
  //     generateNode({ id: 0, ipv4: true }, relay.peerInfo),
  //     generateNode({ id: 1, ipv6: true }, relay.peerInfo),
  //   ])

  //   await sender.dial(relay.peerInfo)
  //   await counterparty.dial(relay.peerInfo)

  //   await sender.dial(Multiaddr(`/p2p/${counterparty.peerInfo.id.toB58String()}`))

  //   await Promise.all([relay.stop(), sender.stop(), counterparty.stop()])
  // })

  it('should set up a relayed connection and exchange messages', async function () {
    const relay = await generateNode({ id: 2, ipv4: true })

    const [sender, counterparty] = await Promise.all([
      generateNode({ id: 0, ipv4: true, useWebRTC: false }, relay.multiaddrs[0]),
      generateNode({ id: 1, ipv4: true, useWebRTC: false }, relay.multiaddrs[0])
    ])

    connectionHelper([sender, relay])
    connectionHelper([relay, counterparty])

    const INVALID_PORT = 8758
    // @ts-ignore
    const { stream }: { stream: Connection } = await sender.dialProtocol(
      Multiaddr(`/ip4/127.0.0.1/tcp/${INVALID_PORT}/p2p/${counterparty.peerId.toB58String()}`),
      TEST_PROTOCOL
    )

    const testMessage = new TextEncoder().encode('12356')

    let msgReceived = false
    await pipe([testMessage], stream, async (source: AsyncIterable<Uint8Array>) => {
      for await (const msg of source) {
        console.log(`receiving relayed connection. message`, new TextDecoder().decode(msg.slice()))
        if (u8aEquals(msg.slice(), testMessage)) {
          msgReceived = true

          return
        }
      }
    })

    stream.close()

    assert(msgReceived, `Message must be received by counterparty`)

    // await Promise.all([
    //   sender.hangUp(new PeerInfo(counterparty.peerInfo.id)),
    //   counterparty.hangUp(new PeerInfo(sender.peerInfo.id)),
    // ])

    // // Try with abort controller
    // const abort = new AbortController()

    // setTimeout(() => setImmediate(() => abort.abort()), 300)

    // try {
    //   await sender.dialProtocol(
    //     Multiaddr(`/ip4/127.0.0.1/tcp/${INVALID_PORT}/p2p/${counterparty.peerInfo.id.toB58String()}`),
    //     TEST_PROTOCOL,
    //     { signal: abort.signal }
    //   )
    // } catch (err) {
    //   if (err.type !== 'aborted') {
    //     throw err
    //   }
    // }

    // await Promise.all([
    //   sender.hangUp(new PeerInfo(counterparty.peerInfo.id)),
    //   counterparty.hangUp(new PeerInfo(sender.peerInfo.id)),
    // ])

    await Promise.all([sender.stop(), counterparty.stop(), relay.stop()])
  })

  // it('should set up a relayed connection, exchange messages, then reconnect with a different address and exchange messages', async function () {
  //   const relay = await generateNode({ id: 2, ipv4: true, ipv6: true })

  //   let [sender, counterparty] = await Promise.all([
  //     generateNode({ id: 0, ipv4: true, useWebRTC: false }, relay.peerInfo),
  //     generateNode({ id: 1, ipv6: true, useWebRTC: false }, relay.peerInfo),
  //   ])

  //   connectionHelper([sender, relay])
  //   connectionHelper([relay, counterparty])

  //   const INVALID_PORT = 8758

  //   const { stream }: { stream: Stream } = await sender.dialProtocol(
  //     Multiaddr(`/ip4/127.0.0.1/tcp/${INVALID_PORT}/p2p/${counterparty.peerInfo.id.toB58String()}`),
  //     TEST_PROTOCOL
  //   )

  //   const testMessage = randomBytes(37)

  //   let msgReceived = false
  //   await pipe(
  //     [testMessage],
  //     stream,
  //     async (source: AsyncIterable<Uint8Array>) => {
  //       for await (const msg of source) {
  //         if (u8aEquals(msg.slice(), testMessage)) {
  //           msgReceived = true

  //           return
  //         }
  //       }
  //     }
  //   )

  //   assert(msgReceived, `Message must be received by counterparty`)

  //   await counterparty.stop()

  //   // Regenerate with **DIFFERENT** ip:port
  //   counterparty = await generateNode({ id: 1, ipv4: true, useWebRTC: false }, relay.peerInfo)

  //   await counterparty.dial(relay.peerInfo)

  //   const secondMessage = randomBytes(43)

  //   const { stream: secondStream }: { stream: Stream } = await sender.dialProtocol(
  //     Multiaddr(`/ip4/127.0.0.1/tcp/${INVALID_PORT}/p2p/${counterparty.peerInfo.id.toB58String()}`),
  //     TEST_PROTOCOL
  //   )

  //   let secondMessageReceived = false
  //   await pipe(
  //     [secondMessage],
  //     secondStream,
  //     async (source: AsyncIterable<Uint8Array>) => {
  //       for await (const msg of source) {
  //         if (u8aEquals(msg.slice(), secondMessage)) {
  //           secondMessageReceived = true

  //           return
  //         }
  //       }
  //     }
  //   )

  //   assert(secondMessageReceived, `counterparty should receive message after reconnecting to relay node`)

  //   await Promise.all([
  //     sender.stop(),
  //     counterparty.stop(),
  //     relay.stop(),
  //   ])
  // })
})
