/*
 * Add a more usable API on top of LibP2P
 */
import type PeerId from 'peer-id'
import type LibP2P from 'libp2p'
import type { PromiseValue } from '../typescript'
import type { Address } from 'libp2p/src/peer-store/address-book'
import type { TimeoutOpts } from '../async'

import { abortableTimeout } from '../async'

import { debug } from '../debug'
import { green } from 'chalk'

const verbose = debug('hopr-core:libp2p:verbose')
const logError = debug(`hopr-core:libp2p:error`)

const DEFAULT_DHT_QUERY_TIMEOUT = 10000

export type DialOpts = {
  timeout?: number
  signal?: AbortSignal
}

export enum DialStatus {
  SUCCESS = 'SUCCESS',
  TIMEOUT = 'E_TIMEOUT',
  ABORTED = 'E_ABORTED',
  DIAL_ERROR = 'E_DIAL',
  DHT_ERROR = 'E_DHT_QUERY'
}

export type DialResponse =
  | {
      status: DialStatus.SUCCESS
      resp: PromiseValue<ReturnType<LibP2P['dialProtocol']>>
    }
  | {
      status: DialStatus.TIMEOUT
    }
  | {
      status: DialStatus.ABORTED
    }
  | {
      status: DialStatus.DIAL_ERROR
      error: string
      dhtContacted: boolean
    }
  | {
      status: DialStatus.DHT_ERROR
      error: Error
      query: PeerId
    }

function renderPeerStoreAddresses(addresses: Address[], delimiter: string = '\n  '): string {
  return addresses.map((addr: Address) => addr.multiaddr.toString()).join(delimiter)
}

async function doDial(
  libp2p: LibP2P,
  destination: PeerId,
  protocol: string,
  opts: {
    timeout: number
    signal: AbortSignal
  }
): Promise<DialResponse> {
  let err: any
  let struct: PromiseValue<ReturnType<LibP2P['dialProtocol']>> | null

  let addresses = (libp2p.peerStore.get(destination)?.addresses ?? []).map((addr) => addr.multiaddr.toString())

  // Try to use known addresses
  if (addresses.length > 0) {
    try {
      struct = await libp2p.dialProtocol(destination, protocol, { signal: opts.signal })
    } catch (_err) {
      err = _err
    }
  }

  if (struct != null) {
    return { status: DialStatus.SUCCESS, resp: struct }
  }

  if (opts.signal.aborted) {
    return { status: DialStatus.ABORTED }
  }

  if ((err != null || struct == null) && libp2p.peerRouting._routers.length == 0) {
    logError(`Could not dial ${destination.toB58String()} directly and libp2p was started without a DHT.`)
    return { status: DialStatus.DIAL_ERROR, error: err?.message, dhtContacted: false }
  }

  verbose(`could not dial directly${err ? ` (${err.message})` : ''}, looking in the DHT`)

  // Try to get some fresh addresses from the DHT
  let dhtResponse: PromiseValue<ReturnType<LibP2P.PeerRoutingModule['findPeer']>>
  try {
    // Let libp2p populate its internal peerStore with fresh addresses
    dhtResponse = await libp2p.peerRouting.findPeer(destination, { timeout: DEFAULT_DHT_QUERY_TIMEOUT })
  } catch (err) {
    const knownAddresses = libp2p.peerStore.get(destination)?.addresses ?? []

    logError(
      `Querying the DHT for ${green(destination.toB58String())} failed. Known addresses:\n` +
        `  ${knownAddresses.length > 0 ? renderPeerStoreAddresses(knownAddresses) : 'No addresses known'}.\n`,
      err
    )
  }

  if (opts.signal.aborted) {
    return { status: DialStatus.ABORTED }
  }

  const newAddresses = (dhtResponse?.multiaddrs ?? []).filter((addr) => !addresses.includes(addr.toString()))

  // Only start a dial attempt if we have received new addresses
  if (newAddresses.length == 0) {
    return { status: DialStatus.DIAL_ERROR, error: 'No new addresses after contacting the DHT', dhtContacted: true }
  }

  try {
    struct = await libp2p.dialProtocol(destination, protocol, { signal: opts.signal })
  } catch (_err) {
    err = _err
  }

  if (err != null || struct == null) {
    const knownAddresses = libp2p.peerStore.get(destination)?.addresses ?? []
    logError(
      `Cannot connect to ${green(
        destination.toB58String()
      )}. New addresses after DHT request did not lead to a connection. Used addresses:\n` +
        `  ${knownAddresses.length > 0 ? renderPeerStoreAddresses(knownAddresses) : 'No addresses known'}` +
        `${err ? `\n${err.message}` : ''}`
    )

    return { status: DialStatus.DIAL_ERROR, error: err?.message, dhtContacted: true }
  }

  verbose(`Dial after DHT request successful`, struct)

  if (opts.signal.aborted) {
    return { status: DialStatus.TIMEOUT }
  }

  if (struct != null) {
    return { status: DialStatus.SUCCESS, resp: struct }
  }

  throw new Error('Missing error case in dial')
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
  return abortableTimeout(
    (opts: TimeoutOpts) => doDial(libp2p, destination, protocol, opts as any),
    { status: DialStatus.ABORTED },
    { status: DialStatus.TIMEOUT },
    {
      timeout: opts?.timeout ?? DEFAULT_DHT_QUERY_TIMEOUT,
      signal: opts?.signal
    }
  )
}
