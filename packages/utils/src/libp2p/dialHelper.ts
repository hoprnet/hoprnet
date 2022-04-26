/*
 * Add a more usable API on top of LibP2P
 */
import PeerId from 'peer-id'
import type LibP2P from 'libp2p'
import type { Connection } from 'libp2p/src/connection-manager'
import type { MuxedStream } from 'libp2p/src/upgrader'
import { Multiaddr } from 'multiaddr'

import { timeout, abortableTimeout, type TimeoutOpts } from '../async'

import { debug } from '../process'
import { createRelayerKey } from './relayCode'
import { createCircuitAddress } from '../network'

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
      resp: {
        stream: MuxedStream
        conn: Connection
        protocol: string
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
  addressBook: Pick<LibP2P['peerStore']['addressBook'], 'get' | 'add'>
}
type ReducedConnectionManager = Pick<LibP2P['connectionManager'], 'getAll' | 'onDisconnect'>
type ReducedDHT = { contentRouting: Pick<LibP2P['contentRouting'], 'routers' | 'findProviders'> }
type ReducedLibp2p = ReducedDHT & { peerStore: ReducedPeerStore } & {
  connectionManager: ReducedConnectionManager
} & Pick<LibP2P, 'dial'>

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
): Promise<void | {
  conn: Connection
  stream: MuxedStream
  protocol: string
}> {
  const existingConnections = libp2p.connectionManager.getAll(destination)

  if (existingConnections == undefined || existingConnections.length == 0) {
    return
  }

  let stream: MuxedStream | undefined
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
      `dead connection${deadConnections.length == 1 ? '' : 's'} to ${destination.toB58String()}`,
      deadConnections.map((deadConnection: Connection) => deadConnection.id)
    )
  }

  for (const deadConnection of deadConnections) {
    libp2p.connectionManager.onDisconnect(deadConnection)
  }

  if (stream != undefined && conn != undefined) {
    return { conn, stream, protocol }
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

  log(
    `Trying to establish connection to ${
      PeerId.isPeerId(destination) ? destination.toB58String() : destination.toString()
    }`
  )

  let conn: Connection
  try {
    conn = await libp2p.dial(destination, { signal: opts.signal })
  } catch (err) {
    logError(
      `Error while establishing connection to ${
        PeerId.isPeerId(destination) ? destination.toB58String() : destination.toString()
      }.`
    )
    if (err?.message) {
      logError(`Dial error:`, err)
    }
  }

  if (!conn) {
    return
  }

  log(`Connection ${PeerId.isPeerId(destination) ? destination.toB58String() : destination.toString()} established !`)

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
  log(`fetching relay keys for node ${destination.toB58String()} from DHT.`, key)

  try {
    for await (const relayer of libp2p.contentRouting.findProviders(key, {
      timeout: DEFAULT_DHT_QUERY_TIMEOUT
    })) {
      relayers.push(relayer)
    }
  } catch (err) {
    logError(`Error while querying the DHT for ${destination.toB58String()}.`)
    if (err?.message) {
      logError(`DHT error: ${err.message}`)
    }
  }

  if (relayers.length > 0) {
    log(
      `found ${relayers.map((relayer) => relayer.id.toB58String()).join(' ,')} for node ${destination.toB58String()}.`
    )
  } else {
    log(`could not find any relayer for ${destination.toB58String()}`)
  }

  return relayers.map((relayer) => relayer.id)
}

const CODE_P2P = Multiaddr.protocols('p2p').code

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
    log(`Successfully reached ${destination.toB58String()} via existing connection !`)
    return { status: DialStatus.SUCCESS, resp: struct }
  }

  // Fetch known addresses for the given destination peer
  const knownAddressesForPeer = await libp2p.peerStore.addressBook.get(destination)
  if (knownAddressesForPeer.length > 0) {
    // Let's try using the known addresses by connecting directly
    log(`There are ${knownAddressesForPeer.length} already known addresses for ${destination.toB58String()}:`)
    for (const address in knownAddressesForPeer) {
      log(`- ${address}`)
    }
    struct = await establishNewConnection(libp2p, destination, protocol, opts)
    if (struct) {
      log(`Successfully reached ${destination.toB58String()} via already known addresses !`)
      return { status: DialStatus.SUCCESS, resp: struct }
    }
  } else {
    log(`No currently known addresses for peer ${destination.toB58String()}`)
  }

  // Check if DHT is available
  if (libp2p.contentRouting.routers.length == 0) {
    // Stop if there is no DHT available
    await printPeerStoreAddresses(
      `Could not establish a connection to ${destination.toB58String()} and libp2p was started without a DHT. Giving up`,
      destination,
      libp2p.peerStore
    )
    return { status: DialStatus.NO_DHT }
  }

  // Try to get some fresh addresses from the DHT
  log(`Could not reach ${destination.toB58String()} using known addresses, querying DHT for more addresses...`)
  const dhtResult = await queryDHT(libp2p, destination, {
    ...opts,
    signal: undefined
  })

  if (dhtResult.length == 0) {
    await printPeerStoreAddresses(
      `Direct dial attempt to ${destination.toB58String()} failed and DHT query has not brought any new addresses. Giving up`,
      destination,
      libp2p.peerStore
    )
    return { status: DialStatus.DHT_ERROR, query: destination.toB58String() }
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

  let relayStruct: {
    stream: MuxedStream
    protocol: string
    conn: Connection
  }

  // Filter out the circuit addresses that were tried using the previous attempt
  const circuitsNotTriedYet = dhtResult
    .map((relay) => createCircuitAddress(relay, destination))
    .filter((circuitAddr) => !knownCircuitAddressSet.has(circuitAddr.toString()));

  log(`Proceeding to try with ${circuitsNotTriedYet.length} new relays to reach ${destination.toB58String()}...`)

  for (const circuitAddress of circuitsNotTriedYet) {

      // Share new knowledge about peer with Libp2p's peerStore
      await libp2p.peerStore.addressBook.add(destination, [circuitAddress])

      log(`Trying to reach ${destination.toB58String()} via circuit ${circuitAddress}...`)

      relayStruct = await establishNewConnection(libp2p, circuitAddress, protocol, {
        ...opts,
        signal: undefined
      })

      // Return if we were successful
      if (relayStruct) {
        log(`Successfully reached ${destination.toB58String()} via circuit ${circuitAddress} !`)
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
