/// <reference path="./@types/libp2p.ts" />
/// <reference path="./@types/bl.ts" />

import type { Stream, StreamType } from 'libp2p'
import type Multiaddr from 'multiaddr'
import PeerId from 'peer-id'

import { CODE_P2P } from './constants'

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

export function extractPeerIdFromMultiaddr(ma: Multiaddr) {
  const tuples = ma.stringTuples()

  let destPeerId: string
  if (tuples[0][0] == CODE_P2P) {
    destPeerId = tuples[0][1] as string
  } else if (tuples.length >= 3 && tuples[2][0] == CODE_P2P) {
    destPeerId = tuples[2][1] as string
  } else {
    throw Error(`Invalid Multiaddr. Got ${ma.toString()}`)
  }

  return PeerId.createFromB58String(destPeerId)
}
