import pipe from 'it-pipe'
import pushable from 'it-pushable'

import myHandshake from './handshake'

describe('test handshake stream implementation', function () {
  it('should create a stream and upgrade it', async function () {
    const AliceBob = pushable<Uint8Array>()
    const BobAlice = pushable<Uint8Array>()

    const Alice = {
      sink: async (source: AsyncIterable<Uint8Array>) => {
        for await (const msg of source) {
          AliceBob.push(msg)
        }
      },
      source: (async function* () {
        for await (const msg of BobAlice) {
          console.log(`Alice received:`, msg)
          yield msg
        }
      })(),
    }

    const Bob = {
      sink: async (source: AsyncIterable<Uint8Array>) => {
        for await (const msg of source) {
          BobAlice.push(msg)
        }
      },
      source: (async function* () {
        for await (const msg of AliceBob) {
          console.log(`Bob received:`, msg)
          yield msg
        }
      })(),
    }

    const webRTCsendAlice = pushable<Uint8Array>()
    const webRTCrecvAlice = pushable<Uint8Array>()

    const webRTCsendBob = pushable<Uint8Array>()
    const webRTCrecvBob = pushable<Uint8Array>()

    const streamAlice = myHandshake(Alice, webRTCsendAlice, webRTCrecvAlice)
    const streamBob = myHandshake(Bob, webRTCsendBob, webRTCrecvBob)

    pipe(
      // prettier-ignore
      Alice,
      streamAlice.webRtcStream
    )

    pipe(
      // prettier-ignore
      streamBob.webRtcStream,
      Bob
    )

    pipe(
      // prettier-ignore
      Bob,
      streamBob.webRtcStream
    )

    pipe(
      // prettier-ignore
      streamAlice.webRtcStream,
      Alice
    )

    setTimeout(() => {
      webRTCrecvBob.end()
      webRTCrecvAlice.end()
    }, 100)

    const pipePromiseBobAlice = pipe(
      // prettier-ignore
      webRTCrecvAlice,
      async (source: AsyncIterable<Uint8Array>) => {
        for await (const msg of source) {
          console.log('from webRTC recv buffer', msg)
        }
      }
    )

    const pipePromiseAliceBob = pipe(
      // prettier-ignore
      webRTCrecvBob,
      async (source: AsyncIterable<Uint8Array>) => {
        for await (const msg of source) {
          console.log('from webRTC recv buffer', msg)
        }
      }
    )

    webRTCsendAlice.push(new Uint8Array([23, 27]))
    webRTCsendAlice.push(new Uint8Array([24, 28]))

    webRTCsendBob.push(new Uint8Array([33, 37]))
    webRTCsendBob.push(new Uint8Array([34, 38]))

    await Promise.all([pipePromiseBobAlice, pipePromiseAliceBob])

    pipe(
      // prettier-ignore
      [new Uint8Array([0, 4, 5, 6])],
      streamAlice.relayStream
    )

    pipe(
      // prettier-ignore
      streamBob.relayStream,
      async (source: Uint8Array) => {
        for await (const msg of source) {
          console.log(`Bob received:`, msg)
        }
      }
    )

    pipe(
      // prettier-ignore
      [new Uint8Array([0, 1, 2, 3])],
      streamBob.relayStream
    )

    pipe(
      // prettier-ignore
      streamAlice.relayStream,
      async (source: Uint8Array) => {
        for await (const msg of source) {
          console.log(`Alice received:`, msg)
        }
      }
    )
  })

  it('should create a downgraded stream without any further WebRTC interaction', async function () {
    const AliceBob = pushable<Uint8Array>()
    const BobAlice = pushable<Uint8Array>()

    const Alice = {
      sink: async (source: AsyncIterable<Uint8Array>) => {
        for await (const msg of source) {
          AliceBob.push(msg)
        }
      },
      source: (async function* () {
        for await (const msg of BobAlice) {
          console.log(`Alice received:`, msg)
          yield msg
        }
      })(),
    }

    const Bob = {
      sink: async (source: AsyncIterable<Uint8Array>) => {
        for await (const msg of source) {
          BobAlice.push(msg)
        }
      },
      source: (async function* () {
        for await (const msg of AliceBob) {
          console.log(`Bob received:`, msg)
          yield msg
        }
      })(),
    }

    const streamAlice = myHandshake(Alice, undefined, undefined)
    const streamBob = myHandshake(Bob, undefined, undefined)

    pipe(
      // prettier-ignore
      [new Uint8Array([0, 4, 5, 6])],
      streamAlice.relayStream
    )

    pipe(
      // prettier-ignore
      streamBob.relayStream,
      async (source: Uint8Array) => {
        for await (const msg of source) {
          console.log(`Bob received:`, msg)
        }
      }
    )

    pipe(
      // prettier-ignore
      [new Uint8Array([0, 1, 2, 3])],
      streamBob.relayStream
    )

    pipe(
      // prettier-ignore
      streamAlice.relayStream,
      async (source: Uint8Array) => {
        for await (const msg of source) {
          console.log(`Alice received:`, msg)
        }
      }
    )
  })

  it('should create a stream that uses on one side WebRTC buffer and downgrade that stream to the base stream', async function () {
    const AliceBob = pushable<Uint8Array>()
    const BobAlice = pushable<Uint8Array>()

    const Alice = {
      sink: async (source: AsyncIterable<Uint8Array>) => {
        for await (const msg of source) {
          AliceBob.push(msg)
        }
      },
      source: (async function* () {
        for await (const msg of BobAlice) {
          console.log(`Alice received:`, msg)
          yield msg
        }
      })(),
    }

    const Bob = {
      sink: async (source: AsyncIterable<Uint8Array>) => {
        for await (const msg of source) {
          BobAlice.push(msg)
        }
      },
      source: (async function* () {
        for await (const msg of AliceBob) {
          console.log(`Bob received:`, msg)
          yield msg
        }
      })(),
    }

    const webRTCsendAlice = pushable<Uint8Array>()
    const webRTCrecvAlice = pushable<Uint8Array>()

    const streamAlice = myHandshake(Alice, webRTCsendAlice, webRTCrecvAlice)
    const streamBob = myHandshake(Bob, undefined, undefined)

    pipe(
      // prettier-ignore
      Alice,
      streamAlice.webRtcStream
    )

    pipe(
      // prettier-ignore
      streamAlice.webRtcStream,
      Alice
    )

    setTimeout(() => {
      webRTCrecvAlice.end()
    }, 100)

    pipe(
      // prettier-ignore
      [new Uint8Array([0, 4, 5, 6])],
      streamAlice.relayStream
    )

    pipe(
      // prettier-ignore
      streamBob.relayStream,
      async (source: Uint8Array) => {
        for await (const msg of source) {
          console.log(`Bob received:`, msg)
        }
      }
    )

    pipe(
      // prettier-ignore
      [new Uint8Array([0, 1, 2, 3])],
      streamBob.relayStream
    )

    pipe(
      // prettier-ignore
      streamAlice.relayStream,
      async (source: Uint8Array) => {
        for await (const msg of source) {
          console.log(`Alice received:`, msg)
        }
      }
    )
  })
})
