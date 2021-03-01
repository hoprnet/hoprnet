// @TODO include libp2p types
// import LibP2P from 'libp2p'

import AbortController from 'abort-controller'
import PeerId from 'peer-id'
import Multiaddr from 'multiaddr'

import Debug from 'debug'
const verbose = Debug('hopr-core:libp2p:verbose')
const error = Debug(`hopr-core:libp2p:error`)

const DEFAULT_DHT_QUERY_TIMEOUT = 10000

/**
 * Combines libp2p methods such as dialProtocol and peerRouting.findPeer
 * to establish a connection.
 * Contains a baseline protection against dailing same addresses twice.
 * @param libp2p a libp2p instance
 * @param destination PeerId of the destination
 * @param protocols protocols to use
 * @param opts
 */
export async function dialHelper(
  libp2p: any,
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
): Promise<any | undefined> {
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
  let struct: any

  let addresses = (libp2p.peerStore.get(destination)?.addresses ?? []).map((addr: any) => addr.multiaddr.toString())

  // Try to use known addresses
  if (addresses.length > 0) {
    try {
      struct = await libp2p.dialProtocol(destination, protocol, { signal })
    } catch (_err) {
      err = _err
    }
  }

  if (struct != null) {
    clearTimeout(timeout as NodeJS.Timeout)
    return struct
  }

  if (signal.aborted || ((err != null || struct == null) && libp2p._dht == undefined)) {
    error(`Could not dial ${destination.toB58String()} directly and libp2p was started without a DHT.`)
    return undefined
  }

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
