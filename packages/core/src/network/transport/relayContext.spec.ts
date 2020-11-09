import { u8aConcat } from '@hoprnet/hopr-utils'
import { RELAY_PAYLOAD_PREFIX } from './constants'
import { RelayContext } from './relayContext'
import { RelayConnection } from './relayConnection'
import type { Stream } from 'libp2p'
import assert from 'assert'

import Pair from 'it-pair'

import Debug from 'debug'
import PeerId from 'peer-id'

const log = Debug(`hopr-core:transport`)

describe('test overwritable connection', function () {
  let iteration = 0

  function getStream({ usePrefix }: { usePrefix: boolean }): Stream {
    let _iteration = iteration

    return {
      source: (async function* () {
        log(`source triggered in .spec`)
        let i = 0
        let msg: Uint8Array
        for (; i < 7; i++) {
          msg = new TextEncoder().encode(`iteration ${_iteration} - msg no. ${i}`)
          if (usePrefix) {
            yield u8aConcat(RELAY_PAYLOAD_PREFIX, msg)
          } else {
            yield msg
          }

          //await new Promise((resolve) => setTimeout(resolve, 40))
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

            console.log(`receiver #${_iteration}`, new TextDecoder().decode(msg))
          } else {
            console.log(`received empty message`, _msg)
          }
        }
        console.log(`sinkDone`)
      }
    }
  }

  // it('should create a connection and overwrite it', async function () {
  //   const ctx = new RelayContext(getStream({ usePrefix: true }))

  //   let interval = setInterval(
  //     () =>
  //       setImmediate(() => {
  //         ctx.update(getStream({ usePrefix: true }))
  //         iteration++
  //       }),
  //     500
  //   )

  //   let done = false

  //   setTimeout(() => {
  //     done = true
  //     log(`timeout done`)
  //     clearInterval(interval)
  //   }, 3000)

  //   let i = 0
  //   ctx.sink(
  //     (async function* () {
  //       await new Promise((resolve) => setTimeout(resolve, 123))
  //       while (i < 28) {
  //         yield u8aConcat(RELAY_PAYLOAD_PREFIX, new TextEncoder().encode(`msg from initial party #${i++}`))
  //         await new Promise((resolve) => setTimeout(resolve, 100))
  //       }
  //     })()
  //   )

  //   // @TODO count messages

  //   for await (const msg of ctx.source) {
  //     if (done) {
  //       break
  //     }
  //     console.log(`initial source <${new TextDecoder().decode(msg.slice())}>`)
  //   }
  // })

  // it('should simulate relay usage', async function () {
  //   const streamA = getStream({ usePrefix: true })
  //   const streamB = getStream({ usePrefix: true })

  //   const [self, counterparty] = await Promise.all(
  //     Array.from({ length: 2 }).map(() => PeerId.create({ keyType: 'secp256k1' }))
  //   )

  //   const AtoB = Pair()
  //   const BtoA = Pair()

  //   const ctxA = new RelayConnection({
  //     stream: {
  //       sink: AtoB.sink,
  //       source: BtoA.source
  //     },
  //     onReconnect: async () => {},
  //     self,
  //     counterparty
  //   })

  //   const ctxB = new RelayContext({
  //     sink: BtoA.sink,
  //     source: AtoB.source
  //   })

  //   streamA.sink(ctxA.source)
  //   ctxA.sink(streamA.source)

  //   streamB.sink(ctxB.source)
  //   ctxB.sink(streamB.source)

  //   console.log(`ping`, await ctxB.ping())

  //   await new Promise((resolve) => setTimeout(resolve, 2000))
  // })

  it('should simulate a reconnect', async function () {
    const connectionA = Pair()
    const connectionB = Pair()

    const ctxSender = new RelayContext({
      sink: connectionA.sink,
      source: connectionB.source
    })

    const ctxCounterparty = new RelayContext(getStream({ usePrefix: true }))

    ctxSender.sink(ctxCounterparty.source)
    ctxCounterparty.sink(ctxSender.source)

    const [self, counterparty] = await Promise.all(
      Array.from({ length: 2 }).map(() => PeerId.create({ keyType: 'secp256k1' }))
    )

    iteration++
    const receiverStream = getStream({ usePrefix: false })

    const ctx = new RelayConnection({
      stream: {
        sink: connectionB.sink,
        source: connectionA.source
      },
      self,
      counterparty,
      onReconnect: async () => {
        console.log(`reconnected`)
        const newStream = ctx.switch()

        iteration++
        console.log(`in reconnect: iteration ${iteration}`)
        const demoStream = getStream({ usePrefix: false })

        newStream.sink(demoStream.source)
        demoStream.sink(newStream.source)
      }
    })

    ctx.sink(receiverStream.source)
    receiverStream.sink(ctx.source)

    let pingPromise: Promise<number>
    setTimeout(() => {
      iteration++
      pingPromise = ctxSender.ping()
      ctxCounterparty.update(getStream({ usePrefix: true }))
    }, 200)

    await new Promise((resolve) => setTimeout(resolve, 1000))

    assert((await pingPromise) > 0)

    await ctx.close()
  })
})
