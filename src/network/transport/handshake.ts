import type { Stream } from './types'
import { WEBRTC_TRAFFIC_PREFIX, REMAINING_TRAFFIC_PREFIX, WEBRTC_TIMEOUT } from './constants'
import defer from 'p-defer'
import type { Pushable } from 'it-pushable'
import pushable from 'it-pushable'
import { u8aConcat } from '@hoprnet/hopr-utils'

import debug from 'debug'

const log = debug('hopr-core:transport')
const error = debug('hopr-core:transport:error')

export default function myHandshake(
  webRTCsendBuffer: Pushable<Uint8Array> | undefined,
  webRTCrecvBuffer: Pushable<Uint8Array> | undefined,
  options?: { signal?: AbortSignal }
): {
  relayStream: Stream
  webRtcStream: Stream
} {
  const sourcePromise = defer<AsyncIterable<Uint8Array>>()

  const connector = pushable<Uint8Array>()

  let webRTCused = false

  const webRtcStream = {
    sink: (source: AsyncIterable<Uint8Array>) => {
      webRTCused = webRTCused || true

      return (async function* () {
        if (webRTCsendBuffer != null) {
          const timeout = setTimeout(() => {
            webRTCsendBuffer.end()
          }, WEBRTC_TIMEOUT)

          for await (const msg of webRTCsendBuffer) {
            if (msg == null) {
              continue
            }

            yield u8aConcat(new Uint8Array([WEBRTC_TRAFFIC_PREFIX]), msg.slice())
          }

          clearTimeout(timeout)
        }

        const source = await sourcePromise.promise
        yield* source
      })()
    },
    source: async (source: AsyncIterable<Uint8Array>) => {
      webRTCused = webRTCused || true

      let doneWithWebRTC = false

      for await (const msg of source) {
        if (msg == null) {
          continue
        }

        if (!doneWithWebRTC && msg.slice(0, 1)[0] == WEBRTC_TRAFFIC_PREFIX) {
          webRTCrecvBuffer?.push(msg.slice(1))
        } else if (msg.slice(0, 1)[0] == REMAINING_TRAFFIC_PREFIX) {
          if (!doneWithWebRTC) {
            doneWithWebRTC = true
            webRTCrecvBuffer?.end()
          }

          connector.push(msg.slice(1))
        }
      }

      connector.end()
    },
  }
  const relayStream = {
    sink: async (source: AsyncIterable<Uint8Array>) => {
      webRTCsendBuffer?.end()

      sourcePromise.resolve(
        (async function* () {
          for await (const msg of source) {
            if (msg == null) {
              continue
            }

            yield u8aConcat(new Uint8Array([REMAINING_TRAFFIC_PREFIX]), msg.slice())
          }
        })()
      )
    },
    source: connector,
  }

  return {
    // @ts-ignore
    relayStream,
    // @ts-ignore
    webRtcStream,
  }
}
