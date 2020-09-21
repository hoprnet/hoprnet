import pipe from 'it-pipe'
import pushable from 'it-pushable'

import myHandshake from './handshake'

import assert from 'assert'
import { u8aEquals } from '@hoprnet/hopr-utils'
import { randomBytes } from 'crypto'

import Pair = require('it-pair')

describe('test handshake stream implementation', function () {
  it('should create a stream and upgrade it', async function () {
    const AliceBob = Pair()
    const BobAlice = Pair()

    const webRTCsendAlice = pushable<Uint8Array>()
    const webRTCrecvAlice = pushable<Uint8Array>()

    const webRTCsendBob = pushable<Uint8Array>()
    const webRTCrecvBob = pushable<Uint8Array>()

    const streamAlice = myHandshake(webRTCsendAlice, webRTCrecvAlice)
    const streamBob = myHandshake(webRTCsendBob, webRTCrecvBob)

    pipe(
      // prettier-ignore
      BobAlice.source,
      streamAlice.webRtcStream.source
    )

    pipe(
      // prettier-ignore
      streamAlice.webRtcStream.sink,
      AliceBob.sink
    )

    pipe(
      // prettier-ignore
      AliceBob.source,
      streamBob.webRtcStream.source
    )

    pipe(
      // prettier-ignore
      streamBob.webRtcStream.sink,
      BobAlice.sink
    )

    await new Promise((resolve) => setTimeout(resolve, 100))

    setTimeout(() => {
      webRTCrecvBob.end()
      webRTCrecvAlice.end()
    }, 100)

    let webRTCmessageBobreceived = false
    const webRTCmessageForBob = randomBytes(23)

    let webRTCmessageAlicereceived = false
    const webRTCmessageForAlice = randomBytes(41)

    setImmediate(() => webRTCsendAlice.push(webRTCmessageForBob))
    setImmediate(() => webRTCsendBob.push(webRTCmessageForAlice))

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

    await Promise.all([pipePromiseBobAlice, pipePromiseAliceBob])

    assert(webRTCmessageBobreceived && webRTCmessageAlicereceived, `both parties should receive a fake WebRTC message`)

    let relayMessageBobreceived = false
    const relayMessageForBob = randomBytes(23)

    let relayMessageAlicereceived = false
    const relayMessageForAlice = randomBytes(41)

    pipe(
      // prettier-ignore
      [relayMessageForBob],
      streamAlice.relayStream.sink
    )

    const relayPipeAliceBob = pipe(
      // prettier-ignore
      streamAlice.relayStream.source,
      async (source: AsyncIterable<Uint8Array>) => {
        for await (const msg of source) {
          if (u8aEquals(msg, relayMessageForAlice)) {
            relayMessageAlicereceived = true
          }
        }
      }
    )

    pipe(
      // prettier-ignore
      [relayMessageForAlice],
      streamBob.relayStream.sink
    )

    const relayPipeBobAlice = pipe(
      // prettier-ignore
      streamBob.relayStream.source,
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
    const AliceBob = Pair()
    const BobAlice = Pair()

    const streamAlice = myHandshake(undefined, undefined)
    const streamBob = myHandshake(undefined, undefined)

    let relayMessageBobreceived = false
    const relayMessageForBob = randomBytes(23)

    let relayMessageAlicereceived = false
    const relayMessageForAlice = randomBytes(41)

    pipe(
      // prettier-ignore
      BobAlice.source,
      streamAlice.webRtcStream.source
    )

    pipe(
      // prettier-ignore
      streamAlice.webRtcStream.sink,
      AliceBob.sink
    )

    pipe(
      // prettier-ignore
      AliceBob.source,
      streamBob.webRtcStream.source
    )

    pipe(
      // prettier-ignore
      streamBob.webRtcStream.sink,
      BobAlice.sink
    )

    pipe(
      // prettier-ignore
      [relayMessageForBob],
      streamAlice.relayStream.sink
    )

    const relayPipeAliceBob = pipe(
      // prettier-ignore
      streamAlice.relayStream.source,
      async (source: AsyncIterable<Uint8Array>) => {
        for await (const msg of source) {
          if (u8aEquals(msg, relayMessageForAlice)) {
            relayMessageAlicereceived = true
          }
        }
      }
    )

    pipe(
      // prettier-ignore
      [relayMessageForAlice],
      streamBob.relayStream.sink
    )

    const relayPipeBobAlice = pipe(
      // prettier-ignore
      streamBob.relayStream.source,
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
    const AliceBob = Pair()
    const BobAlice = Pair()

    const webRTCsendAlice = pushable<Uint8Array>()
    const webRTCrecvAlice = pushable<Uint8Array>()

    const streamAlice = myHandshake(webRTCsendAlice, webRTCrecvAlice)
    const streamBob = myHandshake(undefined, undefined)

    pipe(
      // prettier-ignore
      BobAlice.source,
      streamAlice.webRtcStream.source
    )

    pipe(
      // prettier-ignore
      streamAlice.webRtcStream.sink,
      AliceBob.sink
    )

    pipe(
      // prettier-ignore
      AliceBob.source,
      streamBob.webRtcStream.source
    )

    pipe(
      // prettier-ignore
      streamBob.webRtcStream.sink,
      BobAlice.sink
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
