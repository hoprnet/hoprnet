import myHandshake from './handshake'

import pushable from 'it-pushable'
import pipe from 'it-pipe'

import upgradeToWebRtc from './webrtc'

describe('test webRTC upgrade with custom handshake', function () {
  it('should use the extended stream and use it to feed WebRTC', async function () {
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

    await Promise.all([
      upgradeToWebRtc(webRTCsendAlice, webRTCrecvAlice, { initiator: true }),
      upgradeToWebRtc(webRTCsendBob, webRTCrecvBob),
    ])

    AliceBob.end()
    BobAlice.end()
  })
})
