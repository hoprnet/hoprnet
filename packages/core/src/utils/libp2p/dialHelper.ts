import LibP2P from 'libp2p'
import AbortController from 'abort-controller'
import type { Handler } from 'libp2p'
import PeerId from 'peer-id'
import Multiaddr from 'multiaddr'

import Debug from 'debug'
const verbose = Debug('hopr-core:libp2p:verbose')
const error = Debug(`hopr-core:libp2p:error`)

/**
 * Combines libp2p methods such as dialProtocol and peerRouting.findPeer
 * to establish a connection.
 * Contains a baseline protection against dailing same addresses twice.
 * @param libp2p a libp2p instance
 * @param counterparty PeerId of the destination
 * @param protocols protocols to use
 * @param ms timeout in ms
 */
export async function dialHelper(
  libp2p: LibP2P,
  counterparty: PeerId,
  protocols: string[],
  ms: number
): Promise<Handler | void> {
  // Prevent us from dialing ourself
  if (counterparty.equals(libp2p.peerId)) {
    console.trace(`Preventing self dial.`)
    return
  }

  const abort = new AbortController()

  const timeout = setTimeout(() => {
    abort.abort()
    verbose(`heartbeat timeout while querying ${counterparty.toB58String()}`)
  }, ms)

  let struct: Handler

  let addresses = (libp2p.peerStore.get(counterparty)?.addresses ?? []).map((addr) => addr.toString())

  // Try to use known addresses
  if (addresses.length > 0) {
    try {
      struct = await libp2p.dialProtocol(counterparty, protocols[0], { signal: abort.signal })
    } catch (err) {
      if (err.type === 'aborted') {
        return
      }
      error(`Error while trying to connect with known addresses. ${err}`)
    }
  }

  if (struct != null) {
    clearTimeout(timeout)
    return struct
  }

  if (abort.signal.aborted) {
    return
  }

  // Only use relayed connection / WebRTC upgrade if we haven't tried this before
  if (!addresses.includes(`/p2p/${counterparty.toB58String()}`)) {
    // Try to bypass any existing NATs
    try {
      struct = await libp2p.dialProtocol(Multiaddr(`/p2p/${counterparty.toB58String()}`), protocols[0], {
        signal: abort.signal
      })
    } catch (err) {
      if (err.type === 'aborted') {
        return
      }
    }
  }

  if (struct != null) {
    clearTimeout(timeout)
    return struct
  }

  if (abort.signal.aborted) {
    return
  }

  // Try to get some fresh addresses from the DHT
  let dhtAddresses: Multiaddr[]
  try {
    // Let libp2p populate its internal peerStore with fresh addresses
    dhtAddresses = (await libp2p.peerRouting?.findPeer(counterparty))?.multiaddrs ?? []
  } catch (err) {
    error(`Querying the DHT as peer ${libp2p.peerId.toB58String()} for ${counterparty.toB58String()} failed. ${err}`)
    return
  }

  if (abort.signal.aborted) {
    return
  }

  const newAddresses: Multiaddr[] = dhtAddresses.filter((addr) => addresses.includes(addr.toString()))

  // Only start a dial attempt if we have received new addresses
  if (newAddresses.length > 0) {
    try {
      struct = await libp2p.dialProtocol(counterparty, protocols[0], { signal: abort.signal })
    } catch (err) {
      error(`Using new addresses after querying the DHT did not lead to a connection. Cannot connect. ${err}`)
      return
    }
  }

  if (struct != null) {
    clearTimeout(timeout)
    return struct
  }

  return
}
