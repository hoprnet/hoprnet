/// <reference path="./@types/it-pair.ts" />
/// <reference path="./@types/libp2p.ts" />
/// <reference path="./@types/stream-to-it.ts" />

import { WebRTCConnection, WEBRTC_UPGRADE_TIMEOUT } from './webRTCConnection'
import Peer from 'simple-peer'

const wrtc = require('wrtc')

import { durations, u8aEquals } from '@hoprnet/hopr-utils'
import { RELAY_PAYLOAD_PREFIX } from './constants'
import { RelayContext } from './relayContext'
import { RelayConnection } from './relayConnection'
import type { Stream } from 'libp2p'
import assert from 'assert'

import Pair from 'it-pair'

import PeerId from 'peer-id'
import { EventEmitter } from 'events'

interface DebugMessage {
  messageId: number
  iteration: number
}

describe('test overwritable connection', function () {
  this.timeout(10000)
  let iteration = 0

  /**
   * Starts a duplex stream that sends numbered messages and checks
   * whether the received messages came from the same sender and in
   * expected order.
   * @param arg options
   */
  function getStream(arg: { usePrefix: boolean; designatedReceiverId?: number }): Stream {
    let _iteration = iteration

    let lastId = -1
    let receiver = arg.designatedReceiverId

    return {
      source: (async function* () {
        let i = 0
        let msg: Uint8Array

        let MSG_TIMEOUT = 10

        for (; i < WEBRTC_UPGRADE_TIMEOUT / MSG_TIMEOUT + 5; i++) {
          msg = new TextEncoder().encode(
            JSON.stringify({
              iteration: _iteration,
              messageId: i
            })
          )

          if (arg.usePrefix) {
            yield Uint8Array.from([...RELAY_PAYLOAD_PREFIX, ...msg])
          } else {
            yield msg
          }

          await new Promise<void>((resolve) => setTimeout(resolve, 10))
        }
      })(),
      sink: async (source: Stream['source']) => {
        let msg: Uint8Array

        for await (const _msg of source) {
          if (_msg != null) {
            if (arg.usePrefix) {
              msg = _msg.slice(1)
            } else {
              msg = _msg.slice()
            }

            let decoded = JSON.parse(new TextDecoder().decode(msg)) as DebugMessage
            assert(decoded.messageId == lastId + 1)

            if (receiver == null) {
              receiver = decoded.iteration
            } else {
              assert(receiver == decoded.iteration)
            }

            lastId = decoded.messageId
          } else {
            assert.fail(`received empty message`)
          }
        }
      }
    }
  }

  /**
   * Returns a SimplePeer instance that has mocks for all methods that are
   * required by the transport module.
   */
  function fakeWebRTC(): Peer.Instance {
    const peer = new EventEmitter()

    // @ts-ignore
    peer.connected = false

    // @ts-ignore
    peer.destroyed = false

    // @ts-ignore
    peer.signal = () => {}

    // @ts-ignore
    peer.destroy = () => {
      // @ts-ignore
      peer.destroyed = true
    }

    return peer as Peer.Instance
  }

  it('establish a WebRTC connection', async function () {
    // Sample two parties
    const [partyA, partyB] = await Promise.all(
      Array.from({ length: 2 }).map(() => PeerId.create({ keyType: 'secp256k1' }))
    )

    const connectionA = Pair()
    const connectionB = Pair()

    // Get WebRTC instances
    const PeerA = new Peer({ wrtc, initiator: true, trickle: true })
    const PeerB = new Peer({ wrtc, trickle: true })

    let cutConnection = false
    // Simulated partyA
    const ctxA = new WebRTCConnection({
      conn: new RelayConnection({
        stream: {
          sink: connectionA.sink,
          source: (async function* () {
            for await (const msg of connectionB.source) {
              if (cutConnection && !u8aEquals(RELAY_PAYLOAD_PREFIX, msg.slice())) {
                console.log(msg)
                throw Error(`Connection must not be used`)
              }

              yield msg
            }
          })()
        },
        self: partyA,
        counterparty: partyB,
        webRTC: {
          channel: PeerA,
          upgradeInbound: () => new Peer({ wrtc, trickle: true })
        },
        onReconnect: async () => {}
      }),
      self: partyA,
      counterparty: partyB,
      channel: PeerA
    })

    const ctxB = new WebRTCConnection({
      conn: new RelayConnection({
        stream: {
          sink: connectionB.sink,
          source: (async function* () {
            for await (const msg of connectionA.source) {
              if (cutConnection && !u8aEquals(RELAY_PAYLOAD_PREFIX, msg.slice())) {
                console.log(msg)
                throw Error(`Connection must not be used`)
              }

              yield msg
            }
          })()
        },
        self: partyB,
        counterparty: partyB,
        webRTC: {
          channel: PeerB,
          upgradeInbound: () => new Peer({ wrtc, trickle: true })
        },
        onReconnect: async () => {}
      }),
      self: partyB,
      counterparty: partyA,
      channel: PeerB
    })

    await new Promise((resolve) => setTimeout(resolve, 1000))

    const TEST_MESSAGES = ['first', 'second', 'third'].map((x) => new TextEncoder().encode(x))

    ctxA.sink(
      (async function* () {
        yield* TEST_MESSAGES
      })()
    )

    cutConnection = true

    for await (const msg of ctxB.source) {
      console.log(`msg`, msg)
    }

    await ctxA.close()
  })

  it('should simulate a reconnect after a WebRTC upgrade', async function () {
    // Sample two parties
    const [partyA, partyB] = await Promise.all(
      Array.from({ length: 2 }).map(() => PeerId.create({ keyType: 'secp256k1' }))
    )

    const connectionA = [Pair(), Pair()]
    const connectionB = [Pair(), Pair()]

    // Define relay - one side for A, one side for B
    const relaySideA = new RelayContext({
      sink: connectionA[0].sink,
      source: connectionA[1].source
    })
    const relaySideB = new RelayContext({
      sink: connectionB[0].sink,
      source: connectionB[1].source
    })

    // Wire both sides of the relay
    relaySideA.sink(relaySideB.source)
    relaySideB.sink(relaySideA.source)

    // Get WebRTC instances
    const PeerA = new Peer({ wrtc, initiator: true, trickle: true })
    const PeerB = new Peer({ wrtc, trickle: true })

    // Store the id of the new sender after a reconnect
    // @ts-ignore
    let newSenderId: number

    // Simulated partyA
    const ctxA = new WebRTCConnection({
      conn: new RelayConnection({
        stream: {
          sink: connectionA[1].sink,
          source: connectionA[0].source
        },
        self: partyA,
        counterparty: partyB,
        webRTC: {
          channel: PeerA,
          upgradeInbound: () => new Peer({ wrtc, trickle: true })
        },
        onReconnect: async () => {}
      }),
      self: partyA,
      counterparty: partyB,
      channel: PeerA
    })

    const ctx = new RelayConnection({
      stream: {
        sink: connectionB[1].sink,
        source: connectionB[0].source
      },
      self: partyB,
      counterparty: partyB,
      webRTC: {
        channel: PeerB,
        upgradeInbound: () => new Peer({ wrtc, trickle: true })
      },
      onReconnect: async (newStream: RelayConnection, counterparty: PeerId) => {
        iteration++

        const demoStream = getStream({ usePrefix: false, designatedReceiverId: newSenderId })

        const newConn = new WebRTCConnection({
          conn: newStream,
          self: partyA,
          counterparty,
          channel: newStream.webRTC!.channel
        })

        newConn.sink(demoStream.source)
        demoStream.sink(newConn.source)

        newStream.sink(demoStream.source)
        demoStream.sink(newStream.source)
      }
    })

    // Simulated partyB
    const ctxB = new WebRTCConnection({
      conn: ctx,
      self: partyB,
      counterparty: partyA,
      channel: PeerB
    })

    // Start duplex streams in both directions
    // A -> B
    // B -> A
    const streamA = getStream({ usePrefix: false })
    iteration++
    const streamB = getStream({ usePrefix: false })

    // Pipe the streams
    ctxA.sink(streamA.source)
    streamA.sink(ctxA.source)

    ctxB.sink(streamB.source)
    streamB.sink(ctxB.source)

    // Initiate a reconnect after a timeout
    // Note that this will interrupt the previous stream
    setTimeout(() => {
      const newConnectionA = [Pair(), Pair()]

      const newPeerA = new Peer({ wrtc, initiator: true, trickle: true })

      iteration++
      const newStreamA = getStream({ usePrefix: false })

      const newConn = new WebRTCConnection({
        conn: new RelayConnection({
          stream: {
            sink: newConnectionA[1].sink,
            source: newConnectionA[0].source
          },
          self: partyA,
          counterparty: partyB,
          webRTC: {
            channel: newPeerA,
            upgradeInbound: () => new Peer({ wrtc, trickle: true })
          },
          onReconnect: async () => {}
        }),
        self: partyA,
        counterparty: partyB,
        channel: newPeerA
      })

      relaySideA.update({
        sink: newConnectionA[0].sink,
        source: newConnectionA[1].source
      })

      newSenderId = iteration

      newConn.sink(newStreamA.source)
      newStreamA.sink(newConn.source)
    }, 200)

    await new Promise<void>((resolve) => setTimeout(resolve, 4000))
  })

  it.skip('should simulate a fallback to a relayed connection', async function () {
    this.timeout(durations.seconds(10))

    // Sample two parties
    const [partyA, partyB] = await Promise.all(
      Array.from({ length: 2 }).map(() => PeerId.create({ keyType: 'secp256k1' }))
    )

    const connectionA = [Pair(), Pair()]
    const connectionB = [Pair(), Pair()]

    // Initiate both sides of the relay - sideA, sideB
    const relaySideA = new RelayContext({
      sink: connectionA[0].sink,
      source: connectionA[1].source
    })

    const relaySideB = new RelayContext({
      sink: connectionB[0].sink,
      source: connectionB[1].source
    })

    // Wire relay internally
    relaySideA.sink(relaySideB.source)
    relaySideB.sink(relaySideA.source)

    // Get fake WebRTC instances to trigger a WebRTC timeout
    const PeerA = new Peer({ wrtc, initiator: true, trickle: true })
    const PeerB = fakeWebRTC()

    // @ts-ignore
    let newSenderId = -1

    const ctxA = new WebRTCConnection({
      conn: new RelayConnection({
        stream: {
          sink: connectionA[1].sink,
          source: connectionA[0].source
        },
        self: partyA,
        counterparty: partyB,
        webRTC: {
          channel: PeerA,
          upgradeInbound: () => new Peer({ wrtc, trickle: true })
        },
        onReconnect: async () => {}
      }),
      self: partyA,
      counterparty: partyB,
      channel: PeerA
    })

    const ctx = new RelayConnection({
      stream: {
        sink: connectionB[1].sink,
        source: connectionB[0].source
      },
      self: partyB,
      counterparty: partyB,
      webRTC: {
        channel: PeerB,
        upgradeInbound: () => fakeWebRTC()
      },
      onReconnect: async (newStream: RelayConnection, counterparty: PeerId) => {
        iteration++
        const demoStream = getStream({ usePrefix: false, designatedReceiverId: newSenderId })

        const newConn = new WebRTCConnection({
          conn: newStream,
          self: partyA,
          counterparty,
          channel: newStream.webRTC!.channel
        })

        newConn.sink(demoStream.source)
        demoStream.sink(newConn.source)

        newStream.sink(demoStream.source)
        demoStream.sink(newStream.source)
      }
    })

    const ctxB = new WebRTCConnection({
      conn: ctx,
      self: partyB,
      counterparty: partyA,
      channel: PeerB
    })

    const streamA = getStream({ usePrefix: false })
    iteration++
    const streamB = getStream({ usePrefix: false })

    ctxA.sink(streamA.source)
    streamA.sink(ctxA.source)

    ctxB.sink(streamB.source)
    streamB.sink(ctxB.source)

    setTimeout(() => {
      const newConnectionA = [Pair(), Pair()]

      const newPeerA = fakeWebRTC()

      iteration++
      const newStreamA = getStream({ usePrefix: false })

      newSenderId = iteration

      const newConn = new WebRTCConnection({
        conn: new RelayConnection({
          stream: {
            sink: newConnectionA[1].sink,
            source: newConnectionA[0].source
          },
          self: partyA,
          counterparty: partyB,
          webRTC: {
            channel: newPeerA,
            upgradeInbound: () => fakeWebRTC()
          },
          onReconnect: async () => {}
        }),
        self: partyA,
        counterparty: partyB,
        channel: newPeerA
      })

      relaySideA.update({
        sink: newConnectionA[0].sink,
        source: newConnectionA[1].source
      })

      newConn.sink(newStreamA.source)
      newStreamA.sink(newConn.source)
    }, WEBRTC_UPGRADE_TIMEOUT + 400)

    await new Promise((resolve) => setTimeout(resolve, 4000))
  })
})
