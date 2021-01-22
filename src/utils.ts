/// <reference path="./@types/libp2p.ts" />

import type { Stream } from 'libp2p'

type MyStream = AsyncGenerator<Uint8Array | Buffer | string, void>

export function toU8aStream(source: MyStream): Stream['source'] {
  return (async function* () {
    for await (const msg of source) {
      if (typeof msg === 'string') {
        yield new TextEncoder().encode(msg)
      } else if (Buffer.isBuffer(msg)) {
        yield msg
      } else {
        yield msg.slice()
      }
    }
  })()
}
