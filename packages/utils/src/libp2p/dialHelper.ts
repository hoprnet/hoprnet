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
  CONTINUE = 'CONTINUE'
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

// Make sure that Typescript fails to build tests if libp2p API changes
type ReducedPeerStore = {
  peerStore: {
    get: (peer: PeerId) => Pick<NonNullable<ReturnType<LibP2P['peerStore']['get']>>, 'addresses'> | undefined
  }
}
type ReducedDHT = { peerRouting: Pick<LibP2P['peerRouting'], '_routers' | 'findPeer'> }
type ReducedLibp2p = ReducedDHT & ReducedPeerStore & Pick<LibP2P, 'dialProtocol'>

function printPeerStoreAddresses(msg: string, addresses: Address[]): void {
  logError(msg)
  logError(`Known addresses:`)

  for (const address of addresses) {
    logError(address.multiaddr.toString())
  }
}

/**
 * Performs a dial attempt and handles possible errors.
 * @param libp2p Libp2p instance
 * @param destination which peer to dial
 * @param protocol which protocol to use
 * @param opts timeout options
 */
async function attemptDial(
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
    logError(`Error while dialing ${destination.toB58String()} directly.`)
    if (err?.message) {
      logError(`Dial error: ${err.message}`)
    }
  }

  // Libp2p's return types tend to change every now and then
  if (struct != null) {
    return { status: DialStatus.SUCCESS, resp: struct }
  }

  if (opts.signal.aborted) {
    return { status: DialStatus.ABORTED }
  }

  return { status: InternalDialStatus.CONTINUE }
}

type DHTResponse = Awaited<ReturnType<LibP2P['peerRouting']['findPeer']>>

/**
 * Performs a DHT query and handles possible errors
 * @param libp2p Libp2p instance
 * @param destination which peer to look for
 * @param opts timeout options
 */
async function queryDHT(
  libp2p: ReducedDHT,
  destination: PeerId,
  opts: Required<TimeoutOpts>
): Promise<DialResponse | { status: InternalDialStatus.CONTINUE; dhtResponse: DHTResponse }> {
  let dhtResponse: DHTResponse
  try {
    // Let libp2p populate its internal peerStore with fresh addresses
    dhtResponse = await libp2p.peerRouting.findPeer(destination, { timeout: DEFAULT_DHT_QUERY_TIMEOUT })
  } catch (err) {
    logError(`Error while querying the DHT for ${destination.toB58String()}.`)
    if (err?.message) {
      logError(`DHT error: ${err.message}`)
    }
  }

  // Libp2p's return types tend to change every now and then
  if (dhtResponse == null) {
    return { status: DialStatus.DHT_ERROR, query: destination }
  }

  if (opts.signal.aborted) {
    return { status: DialStatus.ABORTED }
  }

  return { status: InternalDialStatus.CONTINUE, dhtResponse }
}

/**
 * Runs through the dial strategy and handles possible errors
 *
 * 1. Use already known addresses
 * 2. Check the DHT (if available) for additional addresses
 * 3. Try new addresses
 *
 * @param libp2p Libp2p instance
 * @param destination which peer to connect to
 * @param protocol which protocol to use
 * @param opts timeout options
 * @returns
 */
async function doDial(
  libp2p: ReducedLibp2p,
  destination: PeerId,
  protocol: string,
  opts: Required<TimeoutOpts>
): Promise<DialResponse> {
  let dialResult: Awaited<ReturnType<typeof attemptDial>>
  let knownAddresses = libp2p.peerStore.get(destination)?.addresses ?? []

  // Try to use known addresses
  if (knownAddresses.length > 0) {
    dialResult = await attemptDial(libp2p, destination, protocol, opts)

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

  dialResult = await attemptDial(libp2p, destination, protocol, opts)

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
 * Performs a dial strategy using libp2p.dialProtocol and libp2p.findPeer
 * to establish a connection.
 * Contains a baseline protection against dialing same addresses twice.
 * @param libp2p a libp2p instance
 * @param destination PeerId of the destination
 * @param protocols protocols to use
 * @param opts
 */
export async function dial(
  libp2p: ReducedLibp2p,
  destination: PeerId,
  protocol: string,
  opts?: DialOpts
): Promise<DialResponse> {
  return abortableTimeout(
    (timeoutOpts: Required<TimeoutOpts>) => doDial(libp2p, destination, protocol, timeoutOpts),
    { status: DialStatus.ABORTED },
    { status: DialStatus.TIMEOUT },
    {
      timeout: opts?.timeout ?? DEFAULT_DHT_QUERY_TIMEOUT,
      signal: opts?.signal
    }
  )
}
