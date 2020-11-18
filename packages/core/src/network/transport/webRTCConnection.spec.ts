import { WebRTCConnection } from './webRTCConnection'
import Peer from 'simple-peer'
import { yellow } from 'chalk'

// @ts-ignore
import wrtc = require('wrtc')

import { u8aConcat } from '@hoprnet/hopr-utils'
import { RELAY_PAYLOAD_PREFIX } from './constants'
import { RelayContext } from './relayContext'
import { RelayConnection } from './relayConnection'
import type { MultiaddrConnection, Stream } from 'libp2p'
//import assert from 'assert'

import Pair from 'it-pair'

import PeerId from 'peer-id'

describe('test overwritable connection', function () {
  let iteration = 0

  function getStream({ usePrefix }: { usePrefix: boolean }): Stream {
    let _iteration = iteration

    return {
      source: (async function* () {
        let i = 0
        let msg: Uint8Array
        for (; i < 7; i++) {
          msg = new TextEncoder().encode(`iteration ${_iteration} - msg no. ${i}`)
          if (usePrefix) {
            yield u8aConcat(RELAY_PAYLOAD_PREFIX, msg)
          } else {
            yield msg
          }

          await new Promise((resolve) => setTimeout(resolve, 40))
        }
      })(),
      sink: async (source: Stream['source']) => {
        let msg: Uint8Array
        for await (const _msg of source) {
          if (_msg != null) {
            if (usePrefix) {
              msg = _msg.slice(1)
            } else {
              msg = _msg.slice()
            }

            console.log(yellow(`receiver #${_iteration}`, new TextDecoder().decode(msg)))
          } else {
            console.log(`received empty message`, _msg)
          }
        }
        console.log(`sinkDone`)
      }
    }
  }

  it('should simulate a reconnect', async function () {
    const [partyA, partyB] = await Promise.all(
      Array.from({ length: 2 }).map(() => PeerId.create({ keyType: 'secp256k1' }))
    )

    const connectionA = [Pair(), Pair()]
    const connectionB = [Pair(), Pair()]

    // Define relays
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

    // Get WebRTC instances
    const PeerA = new Peer({ wrtc, initiator: true, trickle: true })
    const PeerB = new Peer({ wrtc, trickle: true })

    const ctxA = new WebRTCConnection({
      conn: new RelayConnection({
        stream: {
          sink: connectionA[1].sink,
          source: connectionA[0].source
        },
        self: partyA,
        counterparty: partyB,
        webRTC: PeerA,
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
      webRTC: PeerB,
      onReconnect: async (newStream: MultiaddrConnection, counterparty: PeerId) => {
        console.log(`reconnected`)

        iteration++
        console.log(`in reconnect: iteration ${iteration}`)
        const demoStream = getStream({ usePrefix: false })

        const newConn = new WebRTCConnection({
          conn: newStream,
          self: partyA,
          counterparty,
          channel: (newStream as RelayConnection).webRTC
        })

        newConn.sink(demoStream.source)
        demoStream.sink(newConn.source)

        newStream.sink(demoStream.source)
        demoStream.sink(newStream.source)
      },
      webRTCUpgradeInbound: () => new Peer({ wrtc, trickle: true })
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
          webRTC: newPeerA,
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
    }, 200)

    await new Promise((resolve) => setTimeout(resolve, 4000))
  })
})
