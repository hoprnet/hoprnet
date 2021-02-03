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
 * @param counterparty PeerId of the destination
 * @param protocols protocols to use
 * @param options
 */
export async function dialHelper(
  libp2p: any,
  counterparty: PeerId,
  protocols: string[],
  options:
    | {
        timeout?: number
        signal: AbortSignal
      }
    | {
        timeout: number
        signal?: AbortSignal
      }
): Promise<any | undefined> {
  // Prevent us from dialing ourself
  if (counterparty.equals(libp2p.peerId)) {
    console.trace(`Preventing self dial.`)
    return
  }

  let signal: AbortSignal

  let timeout: NodeJS.Timeout | undefined

  if (options.signal == undefined) {
    const abort = new AbortController()
    signal = abort.signal

    timeout = setTimeout(() => {
      abort.abort()
      verbose(`heartbeat timeout while querying ${counterparty.toB58String()}`)
    }, options.timeout)
  } else {
    signal = options.signal
  }

  let struct: any

  let addresses = (libp2p.peerStore.get(counterparty)?.addresses ?? []).map((addr: Multiaddr) => addr.toString())

  // Try to use known addresses
  if (addresses.length > 0) {
    try {
      struct = await libp2p.dialProtocol(counterparty, protocols[0], { signal })
    } catch (err) {
      if (err.type === 'aborted') {
        return
      }
      error(`Error while trying to connect with known addresses. ${err.message}`)
    }
  }

  if (struct != null) {
    clearTimeout(timeout)
    return struct
  }

  if (signal.aborted) {
    return
  }

  // Only use relayed connection / WebRTC upgrade if we haven't tried this before
  if (!addresses.includes(`/p2p/${counterparty.toB58String()}`)) {
    // Try to bypass any existing NATs
    try {
      struct = await libp2p.dialProtocol(Multiaddr(`/p2p/${counterparty.toB58String()}`), protocols[0], { signal })
    } catch (err) {
      if (err.type === 'aborted') {
        return
      }
      error(`Error while trying to bypass NATs. ${err.message}`)
    }
  }

  if (struct != null) {
    clearTimeout(timeout)
    return struct
  }

  // Only try a DHT query if our libp2p instance comes with a DHT
  if (libp2p._dht != undefined) {
    if (signal.aborted) {
      return
    }

    // Try to get some fresh addresses from the DHT
    let dhtAddresses: Multiaddr[]
    try {
      // Let libp2p populate its internal peerStore with fresh addresses
      dhtAddresses =
        (await libp2p._dht.findPeer(counterparty, { timeout: DEFAULT_DHT_QUERY_TIMEOUT }))?.multiaddrs ?? []
    } catch (err) {
      error(
        `Querying the DHT as peer ${libp2p.peerId.toB58String()} for ${counterparty.toB58String()} failed. ${
          err.message
        }`
      )
      return
    }

    if (signal.aborted) {
      return
    }

    const newAddresses: Multiaddr[] = dhtAddresses.filter((addr) => addresses.includes(addr.toString()))

    // Only start a dial attempt if we have received new addresses
    if (newAddresses.length > 0) {
      try {
        struct = await libp2p.dialProtocol(counterparty, protocols[0], { signal })
      } catch (err) {
        error(`Using new addresses after querying the DHT did not lead to a connection. Cannot connect. ${err.message}`)
        return
      }
    }

    if (struct != null) {
      clearTimeout(timeout)
      return struct
    }
  }

  return
}
