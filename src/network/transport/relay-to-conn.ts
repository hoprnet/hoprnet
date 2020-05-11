import type { MultiaddrConnection, Stream } from './types'
import type PeerId from 'peer-id'

import Multiaddr from 'multiaddr'

export function relayToConn(options: {
  stream: Stream
  counterparty: PeerId
}): MultiaddrConnection {
  return {
    ...options.stream,
    conn: options.stream,
    remoteAddr: Multiaddr(`/p2p/${options.counterparty.toB58String()}`),
    async close(err?: Error) {},
    timeline: {
      open: Date.now(),
    },
  }
}
