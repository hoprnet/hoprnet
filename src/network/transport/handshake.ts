import type { Stream } from './types'
import { WEBRTC_TRAFFIC_PREFIX, REMAINING_TRAFFIC_PREFIX } from './constants'
import defer from 'p-defer'
import type { Pushable } from 'it-pushable'
import pushable from 'it-pushable'
import { u8aConcat } from '@hoprnet/hopr-utils'

import debug from 'debug'

const log = debug('hopr-core:transport')
const error = debug('hopr-core:transport:error')

export default function myHandshake(
  stream: Stream,
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

  let sinkPromise
  const webRtcStream = {
    sink(source: AsyncIterable<Uint8Array>) {
      webRTCused = webRTCused || true

      try {
        sinkPromise = stream.sink(
          // @ts-ignore
          (async function* () {
            if (webRTCsendBuffer != null) {
              for await (const msg of webRTCsendBuffer) {
                if (msg == null) {
                  continue
                }
                yield u8aConcat(new Uint8Array([WEBRTC_TRAFFIC_PREFIX]), msg.slice())
              }
            }

            const source = await sourcePromise.promise
            yield* source
          })()
        )
      } catch (err) {
        if (err.type !== 'aborted') {
          error(err)
        }
      }
    },
    source: (async function* () {
      webRTCused = webRTCused || true

      // let source = options != null && options.signal ? abortable(stream.source, options.signal) : stream.source

      let doneWithWebRTC = false
      for await (const msg of stream.source) {
        if (msg == null) {
          continue
        }

        if (!doneWithWebRTC && msg.slice(0, 1)[0] == WEBRTC_TRAFFIC_PREFIX) {
          webRTCrecvBuffer.push(msg.slice(1))
        } else if (msg.slice(0, 1)[0] == REMAINING_TRAFFIC_PREFIX) {
          if (!doneWithWebRTC) {
            doneWithWebRTC = true
            webRTCrecvBuffer.end()
          }

          connector.push(msg.slice(1))
        }
      }

      connector.end()
    })(),
  }
  const relayStream = {
    async sink(source: AsyncIterable<Uint8Array>) {
      let sink = (async function* () {
        for await (const msg of source) {
          if (msg == null) {
            continue
          }

          yield u8aConcat(new Uint8Array([REMAINING_TRAFFIC_PREFIX]), msg.slice())
        }
      })()

      if (webRTCused) {
        webRTCsendBuffer.end()

        sourcePromise.resolve(sink)
        return sinkPromise
      } else {
        return stream.sink(sink)
      }
    },
    source: (async function* () {
      if (webRTCused) {
        yield* connector
      } else {
        for await (const msg of stream.source) {
          if (msg == null) {
            continue
          }

          if (msg.slice(0, 1)[0] == REMAINING_TRAFFIC_PREFIX) {
            yield msg.slice(1)
          }
        }
      }
    })(),
  }

  return {
    // @ts-ignore
    relayStream,
    // @ts-ignore
    webRtcStream,
  }
}
