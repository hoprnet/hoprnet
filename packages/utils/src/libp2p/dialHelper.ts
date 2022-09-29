/*
 * Add a more usable API on top of LibP2P
 */
import type { PeerId } from '@libp2p/interface-peer-id'
import type { Connection, ProtocolStream } from '@libp2p/interface-connection'
import type { ConnectionManager, Dialer } from '@libp2p/interface-connection-manager'
import type { Components } from '@libp2p/interfaces/components'
import { type Multiaddr, protocols as maProtocols } from '@multiformats/multiaddr'

import { timeout, type TimeoutOpts } from '../async/index.js'

import { debug } from '../process/index.js'
import { createRelayerKey } from './relayCode.js'
import { createCircuitAddress } from '../network/index.js'

const DEBUG_PREFIX = `hopr-core:libp2p`

const CODE_P2P = maProtocols('p2p').code

const log = debug(DEBUG_PREFIX)
const error = debug(DEBUG_PREFIX.concat(`:error`))

const DEFAULT_DHT_QUERY_TIMEOUT = 20000

type MyConnectionManager = ConnectionManager & { dialer: Dialer }

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

async function printPeerStoreAddresses(prefix: string, destination: PeerId, components: Components): Promise<string> {
  const SUFFIX = 'Known addresses:\n'

  let out = `${prefix}\n${SUFFIX}`

  for (const address of await components.getPeerStore().addressBook.get(destination)) {
    if (out.length > prefix.length + 1 + SUFFIX.length) {
      out += '  \n'
    }
    out += `  ${address.multiaddr.toString()}`
  }

  if (out.length == prefix.length + 1 + SUFFIX.length) {
    out += `  No addresses known for peer ${destination.toString()}`
  }

  return out
}

// Timeout protocol selection to prevent from irresponsive nodes
const PROTOCOL_SELECTION_TIMEOUT = 10e3

/**
 * Tries to use existing connection to connect to the given peer.
 * Closes all connection that could not be used to speak the desired
 * protocols.
 * @dev if used with unsupported protocol, this function might close
 * connections unintendedly
 *
 * @param components libp2p components
 * @param destination peer to connect to
 * @param protocol desired protocol
 * @returns
 */
export async function tryExistingConnections(
  components: Components,
  destination: PeerId,
  protocols: string | string[]
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

  let stream: ProtocolStream | undefined
  let conn: Connection | undefined

  const deadConnections: Connection[] = []

  for (const existingConnection of existingConnections) {
    try {
      stream = await timeout(PROTOCOL_SELECTION_TIMEOUT, () => existingConnection.newStream(protocols))
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

  // Close dead connections later
  ;(async function () {
    for (const deadConnection of deadConnections) {
      // @fixme does that work?
      try {
        await deadConnection.close()
      } catch (err) {
        error(`Error while closing dead connection`, err)
      }
    }
  })()

  if (stream != undefined && conn != undefined) {
    return { conn, ...stream }
  }
}

/**
 * Performs a dial attempt and handles possible errors.
 * Uses global connection timeout as defined in libp2p constructor call
 * (see ConnectionManager config)
 *
 * @param components Libp2p components
 * @param destination which peer to dial
 * @param protocols which protocol to use
 */
async function establishNewConnection(
  components: Components,
  destination: PeerId | Multiaddr,
  protocols: string | string[],
  keepAlive: boolean = false
): Promise<
  | void
  | (ProtocolStream & {
      conn: Connection
    })
> {
  log(`Trying to establish connection to ${destination.toString()}`)

  let conn: Connection | undefined
  try {
    conn = (await (components.getConnectionManager() as unknown as MyConnectionManager).dialer.dial(destination, {
      // @ts-ignore - hack
      keepAlive
    })) as any as Connection
  } catch (err: any) {
    error(`Error while establishing connection to ${destination.toString()}.`)
    if (err?.message) {
      error(`Dial error:`, err)
    }
  }

  if (!conn) {
    return
  }

  log(`Connection ${destination.toString()} established !`)

  let stream: ProtocolStream | undefined
  let errThrown = false
  try {
    // Timeout protocol selection to prevent from irresponsive nodes
    stream = await timeout(PROTOCOL_SELECTION_TIMEOUT, () => (conn as Connection).newStream(protocols))
  } catch (err) {
    error(`error while trying to establish protocol ${protocols} with ${destination.toString()}`, err)
    errThrown = true
  }

  if (stream == undefined || errThrown) {
    try {
      await conn.close()
    } catch (err) {
      error(`Error while ending obsolete write stream`, err)
    }
    return
  }

  return {
    conn,
    ...stream
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
  } catch (err: any) {
    done = true
    error(`Error while querying the DHT for ${destination.toString()}.`)
    if (err?.message) {
      error(`DHT error: ${err.message}`)
    }
  }

  if (relayers.length > 0) {
    log(`found ${relayers.map((relayer) => relayer.toString()).join(', ')} for node ${destination.toString()}.`)
  } else {
    log(`could not find any relayer for ${destination.toString()}`)
  }

  return relayers
}

async function doDirectDial(
  components: Components,
  destination: PeerId,
  protocols: string | string[],
  keepAlive: boolean = false
): Promise<DialResponse> {
  // First let's try already existing connections
  let struct = await tryExistingConnections(components, destination, protocols)

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
    struct = await establishNewConnection(components, destination, protocols, keepAlive)
    if (struct) {
      log(`Successfully reached ${destination.toString()} via already known addresses !`)
      return { status: DialStatus.SUCCESS, resp: struct }
    }
  } else {
    log(`No currently known addresses for peer ${destination.toString()}`)
  }

  return { status: DialStatus.DIAL_ERROR, dhtContacted: false }
}

async function fetchCircuitAddressesAndDial(
  components: Components,
  destination: PeerId,
  protocols: string | string[]
): Promise<DialResponse> {
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
    error(
      await printPeerStoreAddresses(
        `Could not establish a connection to ${destination.toString()} and libp2p was started without a DHT. Giving up`,
        destination,
        components
      )
    )
    return { status: DialStatus.NO_DHT }
  }

  const knownAddressesForPeer = await components.getPeerStore().addressBook.get(destination)

  // Try to get some fresh addresses from the DHT
  log(`Could not reach ${destination.toString()} using known addresses, querying DHT for more addresses...`)
  const dhtResult = await queryDHT(components, destination)

  if (dhtResult.length == 0) {
    error(
      await printPeerStoreAddresses(
        `Direct dial attempt to ${destination.toString()} failed and DHT query has not brought any new addresses. Giving up`,
        destination,
        components
      )
    )
    return { status: DialStatus.DHT_ERROR, query: destination.toString() }
  }

  // Take all the known circuit addresses from the existing set of known addresses
  const knownCircuitAddressSet = new Set<string>()

  for (const knownAddressForPeer of knownAddressesForPeer) {
    const tuples = knownAddressForPeer.multiaddr.tuples()

    if (tuples.length > 0 && tuples[0].length > 0 && tuples[0][0] == CODE_P2P) {
      knownCircuitAddressSet.add(knownAddressForPeer.multiaddr.toString())
    }
  }

  let relayStruct:
    | void
    | (ProtocolStream & {
        conn: Connection
      })

  for (const relay of dhtResult) {
    // Make sure we don't use self as relay
    if (relay.equals(components.getPeerId())) {
      continue
    }

    const circuitAddress = createCircuitAddress(relay).encapsulate(`/p2p/${destination.toString()}`)

    // Filter out the circuit addresses that were tried using the previous attempt
    if (knownCircuitAddressSet.has(circuitAddress.toString())) {
      continue
    }

    // Share new knowledge about peer with Libp2p's peerStore, dropping `/p2p/<DESTINATION>`
    await components.getPeerStore().addressBook.add(destination, [createCircuitAddress(relay) as any])

    log(`Trying to reach ${destination.toString()} via circuit ${circuitAddress}...`)

    relayStruct = await establishNewConnection(components, circuitAddress, protocols)

    // Return if we were successful
    if (relayStruct) {
      log(`Successfully reached ${destination.toString()} via circuit ${circuitAddress} !`)
      return { status: DialStatus.SUCCESS, resp: relayStruct }
    }
  }

  return { status: DialStatus.DIAL_ERROR, dhtContacted: true }
}

/**
 * Runs through the dial strategy and handles possible errors
 *
 * 1. Use already known addresses
 * 2. Check the DHT (if available) for additional addresses
 * 3. Try new addresses
 *
 * @param components components of libp2p instance
 * @param destination which peer to connect to
 * @param protocols which protocol to use
 * @returns
 */
export async function dial(
  components: Components,
  destination: PeerId,
  protocols: string | string[],
  withDHT: boolean = true,
  keepAlive: boolean = false
): Promise<DialResponse> {
  const res = await doDirectDial(components, destination, protocols, keepAlive)

  if (withDHT == false || (withDHT == true && res.status == DialStatus.SUCCESS)) {
    // Take first result and don't do any further steps
    return res
  }

  return await fetchCircuitAddressesAndDial(components, destination, protocols)
}

export type { TimeoutOpts as DialOpts }
