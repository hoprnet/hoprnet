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
  sinkBuffer: Pushable<Uint8Array> | undefined,
  srcBuffer: Pushable<Uint8Array> | undefined,
  options?: { signal: AbortSignal }
): {
  relayStream: Stream
  webRtcStream: Stream
} {
  const sourcePromise = defer<AsyncIterable<Uint8Array>>()

  const connector = pushable<Uint8Array>()

  let webRTCused = false

  const webRtcStream = {
    async sink(source: AsyncIterable<Uint8Array>) {
      webRTCused = webRTCused || true

      try {
        await stream.sink(
          // @ts-ignore
          (async function* () {
            if (sinkBuffer != null) {
              for await (const msg of sinkBuffer) {
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

      for await (const msg of stream.source) {
        if (msg == null) {
          continue
        }

        switch (msg.slice(0, 1)[0]) {
          case WEBRTC_TRAFFIC_PREFIX:
            srcBuffer.push(msg.slice(1))
            break
          case REMAINING_TRAFFIC_PREFIX:
            connector.push(msg.slice(1))
            break
        }
      }

      connector.end()
      srcBuffer.end()
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
        sinkBuffer.end()

        sourcePromise.resolve(sink)
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
    webRtcStream,
  }
}
