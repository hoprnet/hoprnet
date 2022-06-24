/*
 * Add a more usable API on top of LibP2P
 */
import { PeerId } from '@libp2p/interface-peer-id'
import type { Libp2p } from 'libp2p'
import type { Connection, ProtocolStream } from '@libp2p/interface-connection'
import { Multiaddr, protocols } from '@multiformats/multiaddr'

import { timeout, abortableTimeout, type TimeoutOpts } from '../async/index.js'

import { debug } from '../process/index.js'
import { createRelayerKey } from './relayCode.js'
import { createCircuitAddress } from '../network/index.js'

const DEBUG_PREFIX = `hopr-core:libp2p`

const log = debug(DEBUG_PREFIX)
const logError = debug(DEBUG_PREFIX.concat(`:error`))

const DEFAULT_DHT_QUERY_TIMEOUT = 20000

export enum DialStatus {
  SUCCESS = 'SUCCESS',
  TIMEOUT = 'E_TIMEOUT',
  ABORTED = 'E_ABORTED',
  DIAL_ERROR = 'E_DIAL',
  DHT_ERROR = 'E_DHT_QUERY',
  NO_DHT = 'E_NO_DHT'
}

export type DialResponse =
  | {
      status: DialStatus.SUCCESS
      resp: ProtocolStream & {
        conn: Connection
      }
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
      query: string
    }
  | {
      status: DialStatus.NO_DHT
    }

// Make sure that Typescript fails to build tests if libp2p API changes
type ReducedPeerStore = {
  addressBook: Pick<Libp2p['peerStore']['addressBook'], 'get' | 'add'>
}
type ReducedConnectionManager = Pick<Libp2p['connectionManager'], 'getAll' | 'onDisconnect'>
type ReducedDHT = { contentRouting: Pick<Libp2p['contentRouting'], 'routers' | 'findProviders'> }
type ReducedLibp2p = ReducedDHT & { peerStore: ReducedPeerStore } & {
  connectionManager: ReducedConnectionManager
} & Pick<Libp2p, 'dial'>

async function printPeerStoreAddresses(msg: string, destination: PeerId, peerStore: ReducedPeerStore): Promise<void> {
  logError(msg)
  logError(`Known addresses:`)

  for (const address of await peerStore.addressBook.get(destination)) {
    logError(address.multiaddr.toString())
  }
}

const PROTOCOL_SELECT_TIMEOUT = 10e3

export async function tryExistingConnections(
  libp2p: Pick<ReducedLibp2p, 'connectionManager'>,
  destination: PeerId,
  protocol: string
): Promise<
  | void
  | (ProtocolStream & {
      conn: Connection
    })
> {
  const existingConnections = libp2p.connectionManager.getAll(destination)

  if (existingConnections == undefined || existingConnections.length == 0) {
    return
  }

  let stream: ProtocolStream | undefined
  let conn: Connection | undefined

  const deadConnections: Connection[] = []

  for (const existingConnection of existingConnections) {
    try {
      stream = (await timeout(PROTOCOL_SELECT_TIMEOUT, () => existingConnection.newStream(protocol)))?.stream
    } catch (err) {}

    if (stream == undefined) {
      deadConnections.push(existingConnection)
    } else {
      conn = existingConnection
      break
    }
  }

  if (deadConnections.length > 0) {
    log(
      `dead connection${deadConnections.length == 1 ? '' : 's'} to ${destination.toString()}`,
      deadConnections.map((deadConnection: Connection) => deadConnection.id)
    )
  }

  for (const deadConnection of deadConnections) {
    libp2p.connectionManager.onDisconnect(deadConnection)

    if (stream != undefined && conn != undefined) {
      return { conn, stream, protocol }
    }
  }
}

/**
 * Performs a dial attempt and handles possible errors.
 * @param libp2p Libp2p instance
 * @param destination which peer to dial
 * @param protocol which protocol to use
 * @param opts timeout options
 */
async function establishNewConnection(
  libp2p: Pick<ReducedLibp2p, 'dial'>,
  destination: PeerId | Multiaddr,
  protocol: string,
  opts: TimeoutOpts
) {
  const start = Date.now()

  let aborted = false

  const onAbort = () => {
    aborted = true
  }

  opts.signal?.addEventListener('abort', onAbort)

  log(`Trying to establish connection to ${destination.toString()}`)

  let conn: Connection
  try {
    conn = await libp2p.dial(destination, { signal: opts.signal })
  } catch (err) {
    logError(`Error while establishing connection to ${destination.toString()}.`)
    if (err?.message) {
      logError(`Dial error:`, err)
    }
  }

  if (!conn) {
    // Do not forget to remove event listener, to prevent leakage
    opts.signal?.removeEventListener('abort', onAbort)
    return
  }

  log(`Connection ${destination.toString()} established !`)

  const stream = (await timeout(10000, () => conn.newStream(protocol)))?.stream

  opts.signal?.removeEventListener('abort', onAbort)

  // Libp2p's return types tend to change every now and then
  if (stream != null && aborted) {
    log(`ending obsolete write stream after ${Date.now() - start} ms`)
    try {
      stream
        .sink((async function* () {})())
        .catch((err: any) => logError(`Error while ending obsolete write stream`, err))
    } catch (err) {
      logError(`Error while ending obsolete write stream`, err)
    }
    return
  }

  if (!stream) {
    return
  }

  return {
    conn,
    stream,
    protocol
  }
}

type Relayers = {
  id: PeerId
  multiaddrs: Multiaddr[]
}

/**
 * Performs a DHT query and handles possible errors
 * @param libp2p Libp2p instance
 * @param destination which peer to look for
 * @param _opts timeout options
 */
async function queryDHT(libp2p: ReducedDHT, destination: PeerId, _opts: Required<TimeoutOpts>): Promise<PeerId[]> {
  const relayers: Relayers[] = []

  const key = await createRelayerKey(destination)
  log(`fetching relay keys for node ${destination.toString()} from DHT.`, key)

  try {
    for await (const relayer of libp2p.contentRouting.findProviders(key, {
      timeout: DEFAULT_DHT_QUERY_TIMEOUT
    })) {
      relayers.push(relayer)
    }
  } catch (err) {
    logError(`Error while querying the DHT for ${destination.toString()}.`)
    if (err?.message) {
      logError(`DHT error: ${err.message}`)
    }
  }

  if (relayers.length > 0) {
    log(`found ${relayers.map((relayer) => relayer.id.toString()).join(' ,')} for node ${destination.toString()}.`)
  } else {
    log(`could not find any relayer for ${destination.toString()}`)
  }

  return relayers.map((relayer) => relayer.id)
}

const CODE_P2P = protocols('p2p').code

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
  // First let's try already existing connections
  let struct = await tryExistingConnections(libp2p, destination, protocol)
  if (struct) {
    log(`Successfully reached ${destination.toString()} via existing connection !`)
    return { status: DialStatus.SUCCESS, resp: struct }
  }

  // Fetch known addresses for the given destination peer
  const knownAddressesForPeer = await libp2p.peerStore.addressBook.get(destination)
  if (knownAddressesForPeer.length > 0) {
    // Let's try using the known addresses by connecting directly
    log(`There are ${knownAddressesForPeer.length} already known addresses for ${destination.toString()}:`)
    for (const address of knownAddressesForPeer) {
      log(`- ${address.multiaddr.toString()}`)
    }
    struct = await establishNewConnection(libp2p, destination, protocol, opts)
    if (struct) {
      log(`Successfully reached ${destination.toString()} via already known addresses !`)
      return { status: DialStatus.SUCCESS, resp: struct }
    }
  } else {
    log(`No currently known addresses for peer ${destination.toString()}`)
  }

  // Check if DHT is available
  if (libp2p.contentRouting.routers.length == 0) {
    // Stop if there is no DHT available
    await printPeerStoreAddresses(
      `Could not establish a connection to ${destination.toString()} and libp2p was started without a DHT. Giving up`,
      destination,
      libp2p.peerStore
    )
    return { status: DialStatus.NO_DHT }
  }

  // Try to get some fresh addresses from the DHT
  log(`Could not reach ${destination.toString()} using known addresses, querying DHT for more addresses...`)
  const dhtResult = await queryDHT(libp2p, destination, {
    ...opts,
    signal: undefined
  })

  if (dhtResult.length == 0) {
    await printPeerStoreAddresses(
      `Direct dial attempt to ${destination.toString()} failed and DHT query has not brought any new addresses. Giving up`,
      destination,
      libp2p.peerStore
    )
    return { status: DialStatus.DHT_ERROR, query: destination.toString() }
  }

  // Take all the known circuit addresses from the existing set of known addresses
  const knownCircuitAddressSet = new Set(
    knownAddressesForPeer
      .map((address) => address.multiaddr)
      .filter((address) => {
        const tuples = address.tuples()
        return tuples[0][0] == CODE_P2P
      })
      .map((address) => address.toString())
  )

  let relayStruct: ProtocolStream & {
    conn: Connection
  }

  // Filter out the circuit addresses that were tried using the previous attempt
  const circuitsNotTriedYet = dhtResult
    .map((relay) => createCircuitAddress(relay, destination))
    .filter((circuitAddr) => !knownCircuitAddressSet.has(circuitAddr.toString()))

  for (const circuitAddress of circuitsNotTriedYet) {
    // Share new knowledge about peer with Libp2p's peerStore
    await libp2p.peerStore.addressBook.add(destination, [circuitAddress])

    log(`Trying to reach ${destination.toString()} via circuit ${circuitAddress}...`)

    relayStruct = await establishNewConnection(libp2p, circuitAddress, protocol, {
      ...opts,
      signal: undefined
    })

    // Return if we were successful
    if (relayStruct) {
      log(`Successfully reached ${destination.toString()} via circuit ${circuitAddress} !`)
      return { status: DialStatus.SUCCESS, resp: relayStruct }
    }
  }

  return { status: DialStatus.DIAL_ERROR, dhtContacted: true }
}

/**
 * Performs a dial strategy using libp2p.dialProtocol and libp2p.findPeer
 * to establish a connection.
 * Contains a baseline protection against dialing same addresses twice.
 * @param libp2p a libp2p instance
 * @param destination PeerId of the destination
 * @param protocol protocols to use
 * @param opts
 */
export async function dial(
  libp2p: ReducedLibp2p,
  destination: PeerId,
  protocol: string,
  opts?: TimeoutOpts
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

export type { TimeoutOpts as DialOpts }
