/*
 * Add a more usable API on top of LibP2P
 */
import type { PeerId } from '@libp2p/interface-peer-id'
import type { Connection, ProtocolStream } from '@libp2p/interface-connection'
import type { Components } from '@libp2p/interfaces/components'
import { type Multiaddr, protocols } from '@multiformats/multiaddr'

import { timeout, abortableTimeout, type TimeoutOpts } from '../async/index.js'

import { debug } from '../process/index.js'
import { createRelayerKey } from './relayCode.js'
import { createCircuitAddress } from '../network/index.js'

const DEBUG_PREFIX = `hopr-core:libp2p`

const log = debug(DEBUG_PREFIX)
const error = debug(DEBUG_PREFIX.concat(`:error`))

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

async function printPeerStoreAddresses(msg: string, destination: PeerId, components: Components): Promise<void> {
  error(msg)
  error(`Known addresses:`)

  for (const address of await components.getPeerStore().addressBook.get(destination)) {
    error(address.multiaddr.toString())
  }
}

const PROTOCOL_SELECT_TIMEOUT = 10e3

export async function tryExistingConnections(
  components: Components,
  destination: PeerId,
  protocol: string
): Promise<
  | void
  | (ProtocolStream & {
      conn: Connection
    })
> {
  const existingConnections = components.getConnectionManager().getConnections(destination)

  if (existingConnections == undefined || existingConnections.length == 0) {
    return
  }

  let stream: ProtocolStream['stream'] | undefined
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
    // @fixme does that work?
    try {
      await deadConnection.close()
    } catch (err) {
      error(`Error while closing dead connection`, err)
    }
  }

  if (stream != undefined && conn != undefined) {
    return { conn, stream, protocol }
  }
}

/**
 * Performs a dial attempt and handles possible errors.
 * @param components Libp2p components
 * @param destination which peer to dial
 * @param protocol which protocol to use
 * @param opts timeout options
 */
async function establishNewConnection(
  components: Components,
  destination: PeerId | Multiaddr,
  protocol: string,
  opts: {
    signal: AbortSignal
  }
) {
  const start = Date.now()

  let aborted = false

  const onAbort = () => {
    aborted = true
  }

  opts.signal?.addEventListener('abort', onAbort)

  log(`Trying to establish connection to ${destination.toString()}`)

  let conn: Connection | undefined
  try {
    // @ts-ignore Dialer is not yet part of interface
    conn = await components.getConnectionManager().dialer.dial(destination, opts)
  } catch (err) {
    error(`Error while establishing connection to ${destination.toString()}.`)
    if (err?.message) {
      error(`Dial error:`, err)
    }
  }

  if (!conn) {
    // Do not forget to remove event listener, to prevent leakage
    opts.signal?.removeEventListener('abort', onAbort)
    return
  }

  log(`Connection ${destination.toString()} established !`)

  const stream = (await timeout(10000, () => (conn as Connection).newStream(protocol)))?.stream

  opts.signal?.removeEventListener('abort', onAbort)

  // Libp2p's return types tend to change every now and then
  if (stream != null && aborted) {
    log(`ending obsolete write stream after ${Date.now() - start} ms`)
    try {
      stream.sink((async function* () {})()).catch((err: any) => error(`Error while ending obsolete write stream`, err))
    } catch (err) {
      error(`Error while ending obsolete write stream`, err)
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

/**
 * Performs a DHT query and handles possible errors
 * @param components Libp2p components
 * @param destination which peer to look for
 */
async function queryDHT(components: Components, destination: PeerId): Promise<PeerId[]> {
  const relayers: PeerId[] = []

  const key = createRelayerKey(destination)
  log(`fetching relay keys for node ${destination.toString()} from DHT.`, key)

  const abort = new AbortController()

  let done = false

  setTimeout(() => {
    if (done) {
      return
    }
    done = true

    abort.abort()
  }, DEFAULT_DHT_QUERY_TIMEOUT).unref()
  try {
    // libp2p type clash
    for await (const relayer of components.getContentRouting().findProviders(key as any, { signal: abort.signal })) {
      relayers.push(relayer.id)
    }
    done = true
  } catch (err) {
    done = true
    error(`Error while querying the DHT for ${destination.toString()}.`)
    if (err?.message) {
      error(`DHT error: ${err.message}`)
    }
  }

  if (relayers.length > 0) {
    log(`found ${relayers.map((relayer) => relayer.toString()).join(' ,')} for node ${destination.toString()}.`)
  } else {
    log(`could not find any relayer for ${destination.toString()}`)
  }

  return relayers
}

const CODE_P2P = protocols('p2p').code

/**
 * Runs through the dial strategy and handles possible errors
 *
 * 1. Use already known addresses
 * 2. Check the DHT (if available) for additional addresses
 * 3. Try new addresses
 *
 * @param components components of libp2p instance
 * @param destination which peer to connect to
 * @param protocol which protocol to use
 * @param opts timeout options
 * @returns
 */
async function doDial(
  components: Components,
  destination: PeerId,
  protocol: string,
  opts: Required<TimeoutOpts>
): Promise<DialResponse> {
  // First let's try already existing connections
  let struct = await tryExistingConnections(components, destination, protocol)

  if (struct) {
    log(`Successfully reached ${destination.toString()} via existing connection !`)
    return { status: DialStatus.SUCCESS, resp: struct }
  }

  // Fetch known addresses for the given destination peer
  const knownAddressesForPeer = await components.getPeerStore().addressBook.get(destination)
  if (knownAddressesForPeer.length > 0) {
    // Let's try using the known addresses by connecting directly
    log(`There are ${knownAddressesForPeer.length} already known addresses for ${destination.toString()}:`)
    for (const address of knownAddressesForPeer) {
      log(`- ${address.multiaddr.toString()}`)
    }
    struct = await establishNewConnection(components, destination, protocol, opts)
    if (struct) {
      log(`Successfully reached ${destination.toString()} via already known addresses !`)
      return { status: DialStatus.SUCCESS, resp: struct }
    }
  } else {
    log(`No currently known addresses for peer ${destination.toString()}`)
  }

  let noDht = false
  try {
    components.getDHT()
  } catch {
    // If there's no DHT set, libp2p-components.getDHT() throws an error
    noDht = true
  }

  // Check if DHT is available
  if (noDht || components.getDHT()[Symbol.toStringTag] === /* catchall package of libp2p */ '@libp2p/dummy-dht') {
    // Stop if there is no DHT available
    await printPeerStoreAddresses(
      `Could not establish a connection to ${destination.toString()} and libp2p was started without a DHT. Giving up`,
      destination,
      components
    )
    return { status: DialStatus.NO_DHT }
  }

  // Try to get some fresh addresses from the DHT
  log(`Could not reach ${destination.toString()} using known addresses, querying DHT for more addresses...`)
  const dhtResult = await queryDHT(components, destination)

  if (dhtResult.length == 0) {
    await printPeerStoreAddresses(
      `Direct dial attempt to ${destination.toString()} failed and DHT query has not brought any new addresses. Giving up`,
      destination,
      components
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

  let relayStruct:
    | (ProtocolStream & {
        conn: Connection
      })
    | undefined

  // Filter out the circuit addresses that were tried using the previous attempt
  const circuitsNotTriedYet = dhtResult
    .map((relay) => createCircuitAddress(relay, destination))
    .filter((circuitAddr) => !knownCircuitAddressSet.has(circuitAddr.toString()))

  for (const circuitAddress of circuitsNotTriedYet) {
    // Share new knowledge about peer with Libp2p's peerStore
    await components.getPeerStore().addressBook.add(destination, [circuitAddress as any])

    log(`Trying to reach ${destination.toString()} via circuit ${circuitAddress}...`)

    relayStruct = await establishNewConnection(components, circuitAddress, protocol, {
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
 * @param components components of a libp2p instance
 * @param destination PeerId of the destination
 * @param protocol protocols to use
 * @param opts
 */
export async function dial(
  components: Components,
  destination: PeerId,
  protocol: string,
  opts?: TimeoutOpts
): Promise<DialResponse> {
  return abortableTimeout(
    (timeoutOpts: Required<TimeoutOpts>) => doDial(components, destination, protocol, timeoutOpts),
    { status: DialStatus.ABORTED },
    { status: DialStatus.TIMEOUT },
    {
      timeout: opts?.timeout ?? DEFAULT_DHT_QUERY_TIMEOUT,
      signal: opts?.signal
    }
  )
}

export type { TimeoutOpts as DialOpts }
