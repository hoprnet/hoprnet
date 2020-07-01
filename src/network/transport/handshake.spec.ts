import pipe from 'it-pipe'
import pushable from 'it-pushable'

import myHandshake from './handshake'

import assert from 'assert'
import { u8aEquals } from '@hoprnet/hopr-utils'
import { randomBytes } from 'crypto'

describe('test handshake stream implementation', function () {
  it('should create a stream and upgrade it', async function () {
    const AliceBob = pushable<Uint8Array>()
    const BobAlice = pushable<Uint8Array>()

    const Alice = {
      sink: async (source: AsyncIterable<Uint8Array>) => {
        for await (const msg of source) {
          AliceBob.push(msg)
        }
        AliceBob.end()
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
        BobAlice.end()
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

    let webRTCmessageBobreceived = false
    const webRTCmessageForBob = randomBytes(23)

    let webRTCmessageAlicereceived = false
    const webRTCmessageForAlice = randomBytes(41)

    const pipePromiseBobAlice = pipe(
      // prettier-ignore
      webRTCrecvAlice,
      async (source: AsyncIterable<Uint8Array>) => {
        for await (const msg of source) {
          if (u8aEquals(msg, webRTCmessageForAlice)) {
            webRTCmessageAlicereceived = true
          }
        }
      }
    )

    const pipePromiseAliceBob = pipe(
      // prettier-ignore
      webRTCrecvBob,
      async (source: AsyncIterable<Uint8Array>) => {
        for await (const msg of source) {
          if (u8aEquals(msg, webRTCmessageForBob)) {
            webRTCmessageBobreceived = true
          }
        }
      }
    )

    webRTCsendAlice.push(webRTCmessageForBob)
    webRTCsendBob.push(webRTCmessageForAlice)

    await Promise.all([pipePromiseBobAlice, pipePromiseAliceBob])

    assert(webRTCmessageBobreceived && webRTCmessageAlicereceived, `both parties should receive a fake WebRTC message`)

    let relayMessageBobreceived = false
    const relayMessageForBob = randomBytes(23)

    let relayMessageAlicereceived = false
    const relayMessageForAlice = randomBytes(41)

    const relayPipeAliceBob = pipe(
      // prettier-ignore
      [relayMessageForBob],
      streamAlice.relayStream,
      async (source: AsyncIterable<Uint8Array>) => {
        for await (const msg of source) {
          if (u8aEquals(msg, relayMessageForAlice)) {
            relayMessageAlicereceived = true
          }
        }
      }
    )

    const relayPipeBobAlice = pipe(
      // prettier-ignore
      [relayMessageForAlice],
      streamBob.relayStream,
      async (source: AsyncIterable<Uint8Array>) => {
        for await (const msg of source) {
          if (u8aEquals(msg, relayMessageForBob)) {
            relayMessageBobreceived = true
          }
        }
      }
    )

    await Promise.all([relayPipeAliceBob, relayPipeBobAlice])

    assert(relayMessageBobreceived && relayMessageAlicereceived, `both parties must receive a fake relayed message`)
  })

  it('should create a downgraded stream without any further WebRTC interactions', async function () {
    const AliceBob = pushable<Uint8Array>()
    const BobAlice = pushable<Uint8Array>()

    const Alice = {
      sink: async (source: AsyncIterable<Uint8Array>) => {
        for await (const msg of source) {
          AliceBob.push(msg)
        }
        AliceBob.end()
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
        BobAlice.end()
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

    let relayMessageBobreceived = false
    const relayMessageForBob = randomBytes(23)

    let relayMessageAlicereceived = false
    const relayMessageForAlice = randomBytes(41)

    const relayPipeAliceBob = pipe(
      // prettier-ignore
      [relayMessageForBob],
      streamAlice.relayStream,
      async (source: AsyncIterable<Uint8Array>) => {
        for await (const msg of source) {
          if (u8aEquals(msg, relayMessageForAlice)) {
            relayMessageAlicereceived = true
          }
        }
      }
    )

    const relayPipeBobAlice = pipe(
      // prettier-ignore
      [relayMessageForAlice],
      streamBob.relayStream,
      async (source: AsyncIterable<Uint8Array>) => {
        for await (const msg of source) {
          if (u8aEquals(msg, relayMessageForBob)) {
            relayMessageBobreceived = true
          }
        }
      }
    )

    await Promise.all([relayPipeAliceBob, relayPipeBobAlice])

    assert(relayMessageBobreceived && relayMessageAlicereceived, `both parties must receive a fake relayed message`)
  })

  it('should create a stream that uses on one side WebRTC buffer and downgrade that stream to the base stream', async function () {
    const AliceBob = pushable<Uint8Array>()
    const BobAlice = pushable<Uint8Array>()

    const Alice = {
      sink: async (source: AsyncIterable<Uint8Array>) => {
        for await (const msg of source) {
          AliceBob.push(msg)
        }
        AliceBob.end()
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
        BobAlice.end()
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

    let relayMessageBobreceived = false
    const relayMessageForBob = randomBytes(23)

    let relayMessageAlicereceived = false
    const relayMessageForAlice = randomBytes(41)

    const relayPipeAliceBob = pipe(
      // prettier-ignore
      [relayMessageForBob],
      streamAlice.relayStream,
      async (source: AsyncIterable<Uint8Array>) => {
        for await (const msg of source) {
          if (u8aEquals(msg, relayMessageForAlice)) {
            relayMessageAlicereceived = true
          }
        }
      }
    )

    const relayPipeBobAlice = pipe(
      // prettier-ignore
      [relayMessageForAlice],
      streamBob.relayStream,
      async (source: AsyncIterable<Uint8Array>) => {
        for await (const msg of source) {
          if (u8aEquals(msg, relayMessageForBob)) {
            relayMessageBobreceived = true
          }
        }
      }
    )

    await Promise.all([relayPipeAliceBob, relayPipeBobAlice])

    assert(relayMessageBobreceived && relayMessageAlicereceived, `both parties must receive a fake relayed message`)
  })
})
