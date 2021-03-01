/// <reference path="../@types/libp2p.ts" />
/// <reference path="../@types/bl.ts" />

import type { Stream, StreamType, Handler } from 'libp2p'
import type LibP2P from 'libp2p'
import Debug from 'debug'
import AbortController, { AbortSignal } from 'abort-controller'
import PeerId from 'peer-id'
import { Multiaddr } from 'libp2p/src/peer-store/address-book'

const verbose = Debug('hopr-connect:dialer:verbose')
const error = Debug('hopr-connect:dialer:error')

export * from './network'

type MyStream = AsyncGenerator<StreamType | Buffer | string, void>

const DEFAULT_DHT_QUERY_TIMEOUT = 2000 // ms

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

export async function dialHelper(
  libp2p: LibP2P,
  destination: PeerId,
  protocol: string,
  opts:
    | {
        timeout?: number
        signal: AbortSignal
      }
    | {
        timeout: number
        signal?: AbortSignal
      }
): Promise<Handler | undefined> {
  let signal: AbortSignal
  let timeout: NodeJS.Timeout | undefined
  if (opts.signal == undefined) {
    const abort = new AbortController()
    signal = abort.signal
    timeout = setTimeout(() => {
      abort.abort()
      verbose(`timeout while querying ${destination.toB58String()}`)
    }, opts.timeout)
  } else {
    signal = opts.signal
  }

  let err: any
  let struct: Handler | undefined
  try {
    struct = await libp2p.dialProtocol(destination, protocol, { signal })
  } catch (_err) {
    err = _err
  }

  if (struct != null) {
    clearTimeout(timeout as NodeJS.Timeout)
    return struct
  }

  if (signal.aborted || ((err != null || struct == null) && libp2p._dht == undefined)) {
    error(`Could not dial ${destination.toB58String()} directly and libp2p was started without a DHT.`)
    return undefined
  }

  let addresses = (libp2p.peerStore.get(destination)?.addresses ?? []).map((addr: any) => addr.multiaddr.toString())

  // Try to get some fresh addresses from the DHT
  let dhtAddresses: Multiaddr[]

  try {
    // Let libp2p populate its internal peerStore with fresh addresses
    dhtAddresses =
      (await (libp2p._dht as any)?.findPeer(destination, { timeout: DEFAULT_DHT_QUERY_TIMEOUT })?.multiaddrs) ?? []
  } catch (err) {
    error(
      `Querying the DHT as peer ${libp2p.peerId.toB58String()} for ${destination.toB58String()} failed. ${err.message}`
    )
    return undefined
  }

  const newAddresses = dhtAddresses.filter((addr) => addresses.includes(addr.toString()))

  // Only start a dial attempt if we have received new addresses
  if (signal.aborted || newAddresses.length > 0) {
    return undefined
  }

  try {
    struct = await libp2p.dialProtocol(destination, protocol, { signal })
    verbose(`Dial after DHT request successful`, struct)
  } catch (err) {
    error(`Using new addresses after querying the DHT did not lead to a connection. Cannot connect. ${err.message}`)
    return undefined
  }

  if (struct != null) {
    clearTimeout(timeout as NodeJS.Timeout)
    return struct
  }

  return undefined
}
