import { RelayContext } from './relayContext'
import { Stream } from '../../@types/transport'

import Debug from 'debug'
const log = Debug(`hopr-core:transport`)

describe('test overwritable connection', function () {
  let iteration = 0

  function getStream(): Stream {
    return {
      source: (async function* () {
        let _iteration = iteration
        console.log(`source triggered in .spec`)
        let i = 0
        for (; i < 7; i++) {
          yield new TextEncoder().encode(`iteration ${_iteration} - msg no. ${i}`)

          await new Promise((resolve) => setTimeout(resolve, 100))
        }

        return new TextEncoder().encode(`iteration ${_iteration} - msg no. ${i}`)
      })(),
      sink: async (source: Stream['source']) => {
        let _iteration = iteration
        console.log(`sinkTriggered`)
        for await (const msg of source) {
          console.log(`receiver #${_iteration}`, new TextDecoder().decode(msg))
        }
        console.log(`sinkDone`)
      }
    }
  }

  it('should create a connection and overwrite it', async function () {
    const ctx = new RelayContext(getStream())

    let interval = setInterval(
      () =>
        setImmediate(() => {
          ctx.update(getStream())
          iteration++
        }),
      500
    )

    setTimeout(() => {
      log(`timeout done`)
      clearInterval(interval)
    }, 3000)

    let i = 0
    ctx.sink(
      (async function* () {
        await new Promise((resolve) => setTimeout(resolve, 123))
        while (true) {
          yield new TextEncoder().encode(`msg from initial party #${i++}`)
          await new Promise((resolve) => setTimeout(resolve, 100))
        }
      })()
    )

    for await (const msg of ctx.source) {
      console.log(`initial source <${new TextDecoder().decode(msg)}>`)
    }
  })
})
