/// <reference path="./@types/libp2p.ts" />
/// <reference path="./@types/bl.ts" />

import type { Stream, StreamType } from 'libp2p'

type MyStream = AsyncGenerator<StreamType | Buffer | string, void>

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
