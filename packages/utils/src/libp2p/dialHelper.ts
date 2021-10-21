/*
 * Add a more usable API on top of LibP2P
 */
import type PeerId from 'peer-id'
import { green } from 'chalk'
import type LibP2P from 'libp2p'

import AbortController from 'abort-controller'
import { debug } from '../debug'
import type { PromiseValue } from '../typescript'
import type { Address } from 'libp2p/src/peer-store/address-book'

const verbose = debug('hopr-core:libp2p:verbose')
const logError = debug(`hopr-core:libp2p:error`)

const DEFAULT_DHT_QUERY_TIMEOUT = 10000

export type DialOpts = {
  timeout: number
}

export type DialResponse =
  | {
      status: 'SUCCESS'
      resp: PromiseValue<ReturnType<LibP2P['dialProtocol']>>
    }
  | {
      status: 'E_TIMEOUT'
    }
  | {
      status: 'E_DIAL'
      error: string
      dhtContacted: boolean
    }
  | {
      status: 'E_DHT_QUERY'
      error: Error
      query: PeerId
    }

function renderPeerStoreAddresses(addresses: Address[], delimiter: string = '\n  '): string {
  return addresses.map((addr: Address) => addr.multiaddr.toString()).join(delimiter)
}
/**
 * Combines libp2p methods such as dialProtocol and peerRouting.findPeer
 * to establish a connection.
 * Contains a baseline protection against dialing same addresses twice.
 * @param libp2p a libp2p instance
 * @param destination PeerId of the destination
 * @param protocols protocols to use
 * @param opts
 */
export async function dial(
  libp2p: LibP2P,
  destination: PeerId,
  protocol: string,
  opts?: DialOpts
): Promise<DialResponse> {
  let timeout: NodeJS.Timeout
  const abort = new AbortController()
  let timeoutPromise = new Promise<DialResponse>((resolve) => {
    timeout = setTimeout(() => {
      abort.abort()
      verbose(`timeout while trying to dial ${destination.toB58String()}`)
      resolve({ status: 'E_TIMEOUT' })
    }, opts.timeout || DEFAULT_DHT_QUERY_TIMEOUT)
  })

  async function doDial(): Promise<DialResponse> {
    let err: any
    let struct: PromiseValue<ReturnType<LibP2P['dialProtocol']>> | null

    let addresses = (libp2p.peerStore.get(destination)?.addresses ?? []).map((addr) => addr.multiaddr.toString())

    // Try to use known addresses
    if (addresses.length > 0) {
      try {
        struct = await libp2p.dialProtocol(destination, protocol, { signal: abort.signal })
      } catch (_err) {
        err = _err
      }
    }

    if (struct != null) {
      clearTimeout(timeout)
      return { status: 'SUCCESS', resp: struct }
    }

    if (abort.signal.aborted) {
      return { status: 'E_TIMEOUT' }
    }

    if ((err != null || struct == null) && libp2p.peerRouting._routers.length > 0) {
      logError(`Could not dial ${destination.toB58String()} directly and libp2p was started without a DHT.`)
      clearTimeout(timeout)
      return { status: 'E_DIAL', error: err.message, dhtContacted: false }
    }

    verbose(`could not dial directly (${err.message}), looking in the DHT`)

    // Try to get some fresh addresses from the DHT
    let dhtResponse: PromiseValue<ReturnType<LibP2P.PeerRoutingModule['findPeer']>>
    try {
      // Let libp2p populate its internal peerStore with fresh addresses
      dhtResponse = await libp2p.peerRouting.findPeer(destination, { timeout: DEFAULT_DHT_QUERY_TIMEOUT })
    } catch (err) {
      logError(
        `Querying the DHT for ${green(destination.toB58String())} failed. Known addresses:\n` +
          `  ${renderPeerStoreAddresses(libp2p.peerStore.get(destination)?.addresses ?? [])}.\n`
      )
    }

    const newAddresses = (dhtResponse?.multiaddrs ?? []).filter((addr) => addresses.includes(addr.toString()))

    if (abort.signal.aborted) {
      return { status: 'E_TIMEOUT' }
    }

    // Only start a dial attempt if we have received new addresses
    if (newAddresses.length == 0) {
      clearTimeout(timeout)
      return { status: 'E_DIAL', error: 'No new addresses after contacting the DHT', dhtContacted: true }
    }

    try {
      struct = await libp2p.dialProtocol(destination, protocol, { signal: abort.signal })
      verbose(`Dial after DHT request successful`, struct)
    } catch (err) {
      logError(
        `Cannot connect to ${green(
          destination.toB58String()
        )}. New addresses after DHT request did not lead to a connection. Used addresses:\n` +
          `  ${renderPeerStoreAddresses(libp2p.peerStore.get(destination)?.addresses ?? [])}\n` +
          `${err.message}`
      )

      clearTimeout(timeout)
      return { status: 'E_DIAL', error: err.message, dhtContacted: true }
    }

    if (abort.signal.aborted) {
      return { status: 'E_TIMEOUT' }
    }

    if (struct != null) {
      clearTimeout(timeout)
      return { status: 'SUCCESS', resp: struct }
    }

    throw new Error('Missing error case in dial')
  }

  // You may be wondering why we race the timeout promise here rather than just
  // relying on the Abort signal.
  // As of #2611, we noticed that the E_TIMEOUT was not being returned until
  // after the request came back, thus the timeout signal was not functioning
  // correctly in this version of libp2p. This is a compromise that means we
  // regain control flow after the timeout, but at the expense of a timed out
  // dial potentially succeeding and being discarded.
  return Promise.race([timeoutPromise, doDial()])
}
