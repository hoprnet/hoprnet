/*
 * Add a more usable API on top of LibP2P
 */
import type PeerId from 'peer-id'
import type LibP2P from 'libp2p'
import type { Address } from 'libp2p/src/peer-store/address-book'
import type { TimeoutOpts } from '../async'

import { abortableTimeout } from '../async'

import { debug } from '../process'
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

enum InternalDialStatus {
  CONTINUE = 'CONTINUE',
  NO_SUCCESS = 'NO_SUCCESS'
}

export type DialResponse =
  | {
      status: DialStatus.SUCCESS
      resp: Awaited<ReturnType<LibP2P['dialProtocol']>>
    }
  | {
      status: DialStatus.TIMEOUT
    }
  | {
      status: DialStatus.ABORTED
    }
  | {
      status: DialStatus.DIAL_ERROR
      dhtContacted: boolean
    }
  | {
      status: DialStatus.DHT_ERROR
      query: PeerId
    }

function printPeerStoreAddresses(msg: string, addresses: Address[], delimiter: string = '\n  '): string {
  return msg.concat(addresses.map((addr: Address) => addr.multiaddr.toString()).join(delimiter))
}

async function directDial(
  libp2p: Pick<LibP2P, 'dialProtocol'>,
  destination: PeerId,
  protocol: string,
  opts: Required<TimeoutOpts>
): Promise<
  | DialResponse
  | {
      status: InternalDialStatus.CONTINUE
    }
> {
  let struct: Awaited<ReturnType<LibP2P['dialProtocol']>> | null

  try {
    struct = await libp2p.dialProtocol(destination, protocol, { signal: opts.signal })
  } catch (err) {
    logError(`Error while dialing ${destination.toB58String()} directly.`, err)
  }

  // Libp2p return types tend to change every now and then
  if (struct != null && struct != undefined) {
    return { status: DialStatus.SUCCESS, resp: struct }
  }

  if (opts.signal.aborted) {
    return { status: DialStatus.ABORTED }
  }

  return {
    status: InternalDialStatus.CONTINUE
  }
}

type DHTResponse = Awaited<ReturnType<LibP2P['peerRouting']['findPeer']>>

async function queryDHT(
  libp2p: Pick<LibP2P, 'peerRouting'>,
  destination: PeerId,
  opts: Required<TimeoutOpts>
): Promise<DialResponse | { status: InternalDialStatus.CONTINUE; dhtResponse: DHTResponse }> {
  let dhtResponse: DHTResponse
  try {
    // Let libp2p populate its internal peerStore with fresh addresses
    dhtResponse = await libp2p.peerRouting.findPeer(destination, { timeout: DEFAULT_DHT_QUERY_TIMEOUT })
  } catch (err) {
    logError(`Error while querying the DHT for ${destination.toB58String()}.`, err)
  }

  // Libp2p return types tend to change every now and then
  if (dhtResponse == null || dhtResponse == undefined) {
    return { status: DialStatus.DHT_ERROR, query: destination }
  }

  if (opts.signal.aborted) {
    return { status: DialStatus.ABORTED }
  }

  return { status: InternalDialStatus.CONTINUE, dhtResponse }
}

async function doDial(
  libp2p: LibP2P,
  destination: PeerId,
  protocol: string,
  opts: Required<TimeoutOpts>
): Promise<DialResponse> {
  let dialResult: Awaited<ReturnType<typeof directDial>>
  let knownAddresses = libp2p.peerStore.get(destination)?.addresses ?? []

  // Try to use known addresses
  if (knownAddresses.length > 0) {
    dialResult = await directDial(libp2p, destination, protocol, opts)

    if (dialResult.status !== InternalDialStatus.CONTINUE) {
      return dialResult
    }
  }

  // Stop if there is no DHT available
  if (libp2p.peerRouting._routers.length == 0) {
    printPeerStoreAddresses(
      `Could not dial ${destination.toB58String()} directly and libp2p was started without a DHT. Giving up`,
      knownAddresses
    )
    return { status: DialStatus.DIAL_ERROR, dhtContacted: false }
  }

  // Try to get some fresh addresses from the DHT
  const dhtResult = await queryDHT(libp2p, destination, opts)

  if (dhtResult.status !== InternalDialStatus.CONTINUE) {
    knownAddresses = libp2p.peerStore.get(destination)?.addresses ?? []

    printPeerStoreAddresses(
      `Could not dial ${destination.toB58String()} directly and libp2p was started without a DHT.`,
      knownAddresses
    )
    return dhtResult
  }

  const knownAddressSet = new Set(knownAddresses.map((address) => address.multiaddr.toString()))

  let newAddresses = 0
  for (const multiaddr of dhtResult.dhtResponse.multiaddrs) {
    if (!knownAddressSet.has(multiaddr.toString())) {
      newAddresses++
    }
  }

  // Only start a dial attempt if we have received new addresses
  if (newAddresses == 0) {
    knownAddresses = libp2p.peerStore.get(destination)?.addresses ?? []

    printPeerStoreAddresses(
      `Querying the DHT for ${green(destination.toB58String())} did not lead to any new addresses. Giving up.`,
      knownAddresses
    )
    return { status: DialStatus.DIAL_ERROR, dhtContacted: true }
  }

  dialResult = await directDial(libp2p, destination, protocol, opts)

  if (dialResult.status !== InternalDialStatus.CONTINUE) {
    knownAddresses = libp2p.peerStore.get(destination)?.addresses ?? []

    printPeerStoreAddresses(
      `New addresses of ${destination.toB58String()} from the DHT did not lead to a connection`,
      knownAddresses
    )
    return dialResult
  }

  return { status: DialStatus.DIAL_ERROR, dhtContacted: true }
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
    (opts: Required<TimeoutOpts>) => doDial(libp2p, destination, protocol, opts),
    { status: DialStatus.ABORTED },
    { status: DialStatus.TIMEOUT },
    {
      timeout: opts?.timeout ?? DEFAULT_DHT_QUERY_TIMEOUT,
      signal: opts?.signal
    }
  )
}
