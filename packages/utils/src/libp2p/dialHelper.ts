/*
 * Add a more usable API on top of LibP2P
 */
import type PeerId from 'peer-id'
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

const DEFAULT_DHT_QUERY_TIMEOUT = 10000

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

const PROTOCOL_SELECT_TIMEOUT = 2e3

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
      deadConnections.map((conn: Connection) => conn.id)
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
export async function establishNewConnection(
  libp2p: Pick<ReducedLibp2p, 'peerStore' | 'dial'>,
  destination: PeerId,
  protocol: string,
  opts: Required<TimeoutOpts>
): Promise<{
  stream: MuxedStream
  conn: Connection
  protocol: string
} | void> {
  const knownAddresses = await libp2p.peerStore.addressBook.get(destination)

  if (knownAddresses.length == 0) {
    return
  }

  const start = Date.now()

  let aborted = false

  const onAbort = () => {
    aborted = true
  }

  opts.signal.addEventListener('abort', onAbort)

  let conn: Connection
  try {
    conn = await libp2p.dial(destination, { signal: opts.signal })
  } catch (err) {
    logError(`Error while dialing ${destination.toB58String()} directly.`)
    if (err?.message) {
      logError(`Dial error:`, err)
    }
  }

  if (!conn) {
    return
  }

  const stream = (await timeout(1000, () => conn.newStream(protocol)))?.stream

  opts.signal.removeEventListener('abort', onAbort)

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

async function establishNewRelayedConnection(
  libp2p: Pick<ReducedLibp2p, 'dial'>,
  addr: Multiaddr,
  protocol: string,
  opts: Required<TimeoutOpts>
): Promise<{
  stream: MuxedStream
  conn: Connection
  protocol: string
}> {
  const start = Date.now()

  let aborted = false

  const onAbort = () => {
    aborted = true
  }

  opts.signal.addEventListener('abort', onAbort)

  let conn: Connection
  try {
    conn = await libp2p.dial(addr, { signal: opts.signal })
  } catch (err) {
    logError(`Error while establising relayed connection using ${addr.toString()}.`)
    if (err?.message) {
      logError(`Dial error:`, err)
    }
  }

  if (!conn) {
    return
  }

  const stream = (await timeout(1000, () => conn.newStream(protocol)))?.stream

  opts.signal.removeEventListener('abort', onAbort)

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
  let struct = await tryExistingConnections(libp2p, destination, protocol)

  if (!struct) {
    struct = await establishNewConnection(libp2p, destination, protocol, opts)
  }

  if (struct) {
    return { status: DialStatus.SUCCESS, resp: struct }
  }

  // Stop if there is no DHT available
  if (!struct && libp2p.contentRouting.routers.length == 0) {
    await printPeerStoreAddresses(
      `Could not establish a connection to ${destination.toB58String()} and libp2p was started without a DHT. Giving up`,
      destination,
      libp2p.peerStore
    )
    return { status: DialStatus.NO_DHT }
  }

  // Try to get some fresh addresses from the DHT
  const dhtResult = await queryDHT(libp2p, destination, opts)

  if (dhtResult.length == 0) {
    await printPeerStoreAddresses(
      `Direct dial attempt to ${destination.toB58String()} failed and DHT query has not brought any new addresses. Giving up`,
      destination,
      libp2p.peerStore
    )
    return { status: DialStatus.DHT_ERROR, query: destination.toB58String() }
  }

  const knownAddresses = (await libp2p.peerStore.addressBook.get(destination))
    .map((address) => address.multiaddr)
    .filter((address) => {
      const tuples = address.tuples()

      return tuples[0][0] == CODE_P2P
    })

  const knownAddressSet = new Set(knownAddresses.map((address) => address.toString()))

  let relayStruct: {
    stream: MuxedStream
    protocol: string
    conn: Connection
  }
  for (const relay of dhtResult) {
    const cirtcuitAddress = createCircuitAddress(relay, destination)

    if (!knownAddressSet.has(cirtcuitAddress.toString())) {
      // Share new knowledge about peer with Libp2p's peerStore
      await libp2p.peerStore.addressBook.add(destination, [cirtcuitAddress])

      // Only establish new connection if not yet successful
      if (!relayStruct) {
        relayStruct = await establishNewRelayedConnection(libp2p, cirtcuitAddress, protocol, opts)
      }
    }
  }

  if (relayStruct) {
    return { status: DialStatus.SUCCESS, resp: relayStruct }
  } else {
    return { status: DialStatus.DIAL_ERROR, dhtContacted: true }
  }
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
