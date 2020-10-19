import { u8aConcat } from '@hoprnet/hopr-utils'
import { RELAY_PAYLOAD_PREFIX } from './constants'
import { RelayContext } from './relayContext'
import { Stream } from '../../@types/transport'

const Pair: () => Stream = require('it-pair')

import Debug from 'debug'
const log = Debug(`hopr-core:transport`)

describe('test overwritable connection', function () {
  let iteration = 0

  function getStream({ usePrefix }: { usePrefix: boolean }): Stream {
    return {
      source: (async function* () {
        let _iteration = iteration
        console.log(`source triggered in .spec`)
        let i = 0
        let msg: Uint8Array
        for (; i < 7; i++) {
          msg = new TextEncoder().encode(`iteration ${_iteration} - msg no. ${i}`)
          if (usePrefix) {
            yield u8aConcat(RELAY_PAYLOAD_PREFIX, msg)
          } else {
            yield msg
          }

          await new Promise((resolve) => setTimeout(resolve, 100))
        }

        return new TextEncoder().encode(`iteration ${_iteration} - msg no. ${i}`)
      })(),
      sink: async (source: Stream['source']) => {
        let _iteration = iteration
        console.log(`sinkTriggered`)
        let msg: Uint8Array
        for await (const _msg of source) {
          if (usePrefix) {
            msg = _msg.slice(1)
          } else {
            msg = _msg.slice()
          }

          console.log(`receiver #${_iteration}`, new TextDecoder().decode(msg))
        }
        console.log(`sinkDone`)
      }
    }
  }

  it('should create a connection and overwrite it', async function () {
    const ctx = new RelayContext(getStream({ usePrefix: false }), {
      useRelaySubprotocol: false,
      sendRestartMessage: false
    })

    let interval = setInterval(
      () =>
        setImmediate(() => {
          ctx.update(getStream({ usePrefix: false }))
          iteration++
        }),
      500
    )

    let done = false

    setTimeout(() => {
      done = true
      log(`timeout done`)
      clearInterval(interval)
    }, 3000)

    let i = 0
    ctx.sink(
      (async function* () {
        await new Promise((resolve) => setTimeout(resolve, 123))
        while (i < 28) {
          yield new TextEncoder().encode(`msg from initial party #${i++}`)
          await new Promise((resolve) => setTimeout(resolve, 100))
        }
      })()
    )

    // @TODO count messages

    for await (const msg of ctx.source) {
      if (done) {
        break
      }
      console.log(`initial source <${new TextDecoder().decode(msg.slice())}>`)
    }
  })

  it('should simulate relay usage', async function () {
    const streamA = getStream({ usePrefix: true })
    const streamB = getStream({ usePrefix: true })

    const AtoB = Pair()
    const BtoA = Pair()

    const ctxA = new RelayContext(
      {
        sink: AtoB.sink,
        source: BtoA.source
      },
      {
        useRelaySubprotocol: true,
        sendRestartMessage: true
      }
    )

    const ctxB = new RelayContext(
      {
        sink: BtoA.sink,
        source: AtoB.source
      },
      {
        useRelaySubprotocol: true,
        sendRestartMessage: true
      }
    )

    streamA.sink(ctxA.source)
    ctxA.sink(streamA.source)

    streamB.sink(ctxB.source)
    ctxB.sink(streamB.source)

    console.log(`ping`, await ctxA.ping())

    await new Promise((resolve) => setTimeout(resolve, 2000))
  })
})
