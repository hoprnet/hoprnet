import type { HoprConnectOptions, PeerStoreType, Stream } from '../types'
import type Connection from 'libp2p-interfaces/src/connection/connection'
import type PeerId from 'peer-id'
import type { Multiaddr } from 'multiaddr'
import type HoprConnect from '..'
import type Libp2p from 'libp2p'

import { EventEmitter } from 'events'
import Debug from 'debug'
import { setTimeout as setTimeoutPromise } from 'timers/promises'

import {
  CODE_IP4,
  CODE_IP6,
  CODE_TCP,
  CODE_UDP,
  MAX_RELAYS_PER_NODE,
  CAN_RELAY_PROTCOL,
  OK,
  DEFAULT_DHT_ENTRY_RENEWAL
} from '../constants'

import { createCircuitAddress, nAtATime, oneAtATime, retimer, u8aEquals } from '@hoprnet/hopr-utils'
import { attemptClose, relayFromRelayAddress } from '../utils'
import { compareDirectConnectionInfo } from '../utils/addrs'

const DEBUG_PREFIX = 'hopr-connect:entry'
const log = Debug(DEBUG_PREFIX)
const error = Debug(DEBUG_PREFIX.concat(':error'))
const verbose = Debug(DEBUG_PREFIX.concat(':verbose'))

type EntryNodeData = PeerStoreType & {
  latency: number
}

type ConnectionResult = {
  entry: EntryNodeData
  conn?: Connection
}

type ConnResult = {
  conn: Connection
  stream: Stream
}

function latencyCompare(a: ConnectionResult, b: ConnectionResult) {
  return a.entry.latency - b.entry.latency
}

function isUsableRelay(ma: Multiaddr) {
  const tuples = ma.tuples()

  return (
    tuples[0].length >= 2 && [CODE_IP4, CODE_IP6].includes(tuples[0][0]) && [CODE_UDP, CODE_TCP].includes(tuples[1][0])
  )
}

type UsedRelay = {
  relayDirectAddress: Multiaddr
  ourCircuitAddress: Multiaddr
}

export const RELAY_CHANGED_EVENT = 'relay:changed'

export const ENTRY_NODES_MAX_PARALLEL_DIALS = 14

type ReducedLibp2p = {
  connectionManager: Pick<Libp2p['connectionManager'], '_started' | 'getAll' | 'onDisconnect'>
}

export class EntryNodes extends EventEmitter {
  protected availableEntryNodes: EntryNodeData[]
  protected uncheckedEntryNodes: PeerStoreType[]
  private stopDHTRenewal: (() => void) | undefined

  protected usedRelays: UsedRelay[]

  private _onNewRelay: ((peer: PeerStoreType) => void) | undefined
  private _onRemoveRelay: ((peer: PeerId) => void) | undefined
  private _connectToRelay: EntryNodes['connectToRelay'] | undefined

  constructor(
    private peerId: PeerId,
    private libp2p: ReducedLibp2p,
    private dialDirectly: HoprConnect['dialDirectly'],
    private options: HoprConnectOptions
  ) {
    super()
    this.availableEntryNodes = []
    this.uncheckedEntryNodes = options.initialNodes ?? []

    this.usedRelays = []
  }

  /**
   * Attaches listeners that handle addition and removal of
   * entry nodes
   */
  public start() {
    this._connectToRelay = this.connectToRelay.bind(this)
    if (this.options.publicNodes != undefined) {
      const limiter = oneAtATime()
      this._onNewRelay = (peer: PeerStoreType) => {
        limiter(async () => {
          log(`peer online`, peer.id.toB58String())
          await this.onNewRelay(peer)
        })
      }
      this._onRemoveRelay = (peer: PeerId) => {
        limiter(async () => {
          log(`peer offline`, peer.toB58String())
          await this.onRemoveRelay(peer)
        })
      }

      this.options.publicNodes.on('addPublicNode', this._onNewRelay)
      this.options.publicNodes.on('removePublicNode', this._onRemoveRelay)
    }

    this.startDHTRenewInterval()
  }

  /**
   * Removes event listeners
   */
  public stop() {
    if (this.options.publicNodes != undefined && this._onNewRelay != undefined && this._onRemoveRelay != undefined) {
      this.options.publicNodes.removeListener('addPublicNode', this._onNewRelay)

      this.options.publicNodes.removeListener('removePublicNode', this._onRemoveRelay)
    }

    this.stopDHTRenewal?.()
  }

  private startDHTRenewInterval() {
    const renewDHTEntries = async function (this: EntryNodes) {
      const work: [id: PeerId, mulitaddr: Multiaddr, timeout: number][] = []
      for (const relay of this.getUsedRelayPeerIds()) {
        const relayEntry = this.availableEntryNodes.find((entry: EntryNodeData) => entry.id.equals(relay))

        if (relayEntry == undefined) {
          log(
            `Relay ${relay.toB58String()} has been removed from list of available entry nodes. Not renewing that entry`
          )
          continue
        }

        work.push([relay, relayEntry.multiaddrs[0], 5e3])
      }

      await nAtATime(this._connectToRelay as EntryNodes['connectToRelay'], work, ENTRY_NODES_MAX_PARALLEL_DIALS)
    }.bind(this)

    this.stopDHTRenewal = retimer(renewDHTEntries, () => this.options.dhtRenewalTimeout ?? DEFAULT_DHT_ENTRY_RENEWAL)
  }

  /**
   * @returns a list of entry nodes that are currently used (as relay circuit addresses with us)
   */
  public getUsedRelayAddresses() {
    return this.usedRelays.map((ur) => ur.ourCircuitAddress)
  }

  /**
   * Convenience method to retrieved used relay peer IDs.
   * @returns a list of peer IDs of used relays.
   */
  private getUsedRelayPeerIds() {
    return this.getUsedRelayAddresses().map((ma) => relayFromRelayAddress(ma))
  }

  /**
   * @returns a list of entry nodes that are considered to be online
   */
  public getAvailabeEntryNodes() {
    return this.availableEntryNodes
  }

  /**
   * @returns a list of entry nodes that will be checked once the
   * list of entry nodes is built or rebuilt next time
   */
  public getUncheckedEntryNodes() {
    return this.uncheckedEntryNodes
  }

  /**
   * Called once there is a new relay opportunity known
   * @param ma Multiaddr of node that is added as a relay opportunity
   */
  protected async onNewRelay(peer: PeerStoreType): Promise<void> {
    if (peer.id.equals(this.peerId)) {
      return
    }

    if (peer.multiaddrs == undefined || peer.multiaddrs.length == 0) {
      log(`Received entry node ${peer.id.toB58String()} without any multiaddr`)
      return
    }

    for (const uncheckedNode of this.uncheckedEntryNodes) {
      if (uncheckedNode.id.equals(peer.id)) {
        log(`Received duplicate entry node ${peer.id.toB58String()}`)
        // TODO add difference to previous multiaddrs
        return
      }
    }

    this.uncheckedEntryNodes.push({
      id: peer.id,
      multiaddrs: peer.multiaddrs.filter(isUsableRelay)
    })

    // Stop adding and checking relay nodes if we already have enough.
    // Once a relay goes offline, the node will try to replace the offline relay.
    if (this.usedRelays.length < MAX_RELAYS_PER_NODE) {
      // Rebuild list of relay nodes later
      await this.updatePublicNodes()
    }
  }

  /**
   * Called once a node is considered to be offline
   * @param ma Multiaddr of node that is considered to be offline now
   */
  protected async onRemoveRelay(peer: PeerId) {
    for (const [index, publicNode] of this.availableEntryNodes.entries()) {
      if (publicNode.id.equals(peer)) {
        // Remove node without changing order
        this.availableEntryNodes.splice(index, 1)
      }
    }

    let inUse = false
    for (const relayPeer of this.getUsedRelayPeerIds()) {
      // remove second part of relay address to get relay peerId
      if (relayPeer.equals(peer)) {
        inUse = true
      }
    }

    // Only rebuild list of relay nodes if we were using the
    // offline node
    if (inUse) {
      // Rebuild list of relay nodes later
      await this.updatePublicNodes()
    }
  }

  /**
   * Filters list of unchecked entry nodes before contacting them
   * @returns a filtered list of entry nodes
   */
  private filterUncheckedNodes(): PeerStoreType[] {
    const knownNodes = new Set<string>(this.availableEntryNodes.map((entry: EntryNodeData) => entry.id.toB58String()))
    const nodesToCheck: PeerStoreType[] = []

    for (const uncheckedNode of this.uncheckedEntryNodes) {
      if (uncheckedNode.id.equals(this.peerId)) {
        continue
      }

      const usableAddresses: Multiaddr[] = uncheckedNode.multiaddrs.filter(isUsableRelay)

      if (knownNodes.has(uncheckedNode.id.toB58String())) {
        const index = this.availableEntryNodes.findIndex((entry) => entry.id.equals(uncheckedNode.id))

        if (index < 0) {
          continue
        }

        // Overwrite previous addresses. E.g. a node was restarted
        // and now announces with a different address
        this.availableEntryNodes[index].multiaddrs = usableAddresses

        // Nothing to do. Public nodes are added later
        continue
      }

      // Ignore if entry nodes have more than one address
      const firstUsableAddress = usableAddresses[0]

      // Ignore if we're already connected to the address
      if (
        this.usedRelays.some((usedRelay) =>
          compareDirectConnectionInfo(usedRelay.relayDirectAddress, firstUsableAddress)
        )
      )
        continue

      nodesToCheck.push({
        id: uncheckedNode.id,
        multiaddrs: [firstUsableAddress]
      })
    }

    return nodesToCheck
  }

  /**
   * Updates the list of exposed entry nodes.
   * Called at startup and once an entry node is considered offline.
   */
  async updatePublicNodes(): Promise<void> {
    while (!this.libp2p.connectionManager._started) {
      // Make sure that libp2p is started
      await setTimeoutPromise(250)
    }
    log(`Updating list of used relay nodes ...`)
    const nodesToCheck = this.filterUncheckedNodes()
    const TIMEOUT = 3e3

    const toCheck = nodesToCheck.concat(this.availableEntryNodes)
    const args: Parameters<EntryNodes['connectToRelay']>[] = new Array(toCheck.length)

    for (const [index, nodeToCheck] of toCheck.entries()) {
      args[index] = [nodeToCheck.id, nodeToCheck.multiaddrs[0], TIMEOUT]
    }

    const start = Date.now()
    const results = (
      await nAtATime(this._connectToRelay as EntryNodes['connectToRelay'], args, ENTRY_NODES_MAX_PARALLEL_DIALS)
    )
      .filter(
        // Filter all unsuccessful dials that cause an error
        (value): value is { entry: EntryNodeData; conn: Connection | undefined } => !(value instanceof Error)
      )
      .sort(latencyCompare)

    log(`Checking ${args.length} potential entry nodes done in ${Date.now() - start} ms.`)

    const positiveOnes = results.findIndex((result: ConnectionResult) => result.entry.latency >= 0)

    const previous = new Set<string>(this.getUsedRelayPeerIds().map((p) => p.toB58String()))

    if (positiveOnes >= 0) {
      // Close all unnecessary connections
      await nAtATime(
        attemptClose,
        results
          .slice(positiveOnes + MAX_RELAYS_PER_NODE)
          .map<[Connection, (arg: any) => void]>((result) => [result.conn as Connection, error]),
        ENTRY_NODES_MAX_PARALLEL_DIALS
      )

      // Take all entry nodes that appeared to be online
      this.availableEntryNodes = results.slice(positiveOnes).map((result) => result.entry)

      this.usedRelays = this.availableEntryNodes
        // select only those entry nodes with smallest latencies
        .slice(0, MAX_RELAYS_PER_NODE)
        .map((entry: EntryNodeData) => {
          return {
            relayDirectAddress: entry.multiaddrs[0],
            ourCircuitAddress: createCircuitAddress(entry.id, this.peerId)
          }
        })
    } else {
      log(`Could not connect to any entry node. Other nodes may not or no longer be able to connect to this node.`)
      // Reset to initial state
      this.usedRelays = []
      this.availableEntryNodes = []
    }

    // Reset list of unchecked nodes
    this.uncheckedEntryNodes = []

    let isDifferent = false

    if (this.usedRelays.length != previous.size) {
      isDifferent = true
    } else {
      for (const usedRelayPeerIds of this.getUsedRelayPeerIds()) {
        if (!previous.has(usedRelayPeerIds.toB58String())) {
          isDifferent = true
          break
        }
      }
    }

    if (isDifferent) {
      log(`Current relay addresses:`)
      for (const ma of this.usedRelays) {
        log(` - ${ma.ourCircuitAddress.toString()}`)
      }

      this.emit(RELAY_CHANGED_EVENT)
    }
  }

  private async tryExistingConnections(destination: PeerId): Promise<ConnResult | void> {
    const existingConnections = this.libp2p.connectionManager.getAll(destination)

    if (existingConnections == undefined || existingConnections.length == 0) {
      return
    }

    let stream: Stream | undefined
    let conn: Connection | undefined

    const deadConnections: Connection[] = []

    for (const existingConnection of existingConnections) {
      try {
        stream = (await existingConnection.newStream(CAN_RELAY_PROTCOL(this.options.environment)))?.stream as Stream
        conn = existingConnection
      } catch (err) {
        deadConnections.push(existingConnection)
        continue
      }
      if (stream == undefined) {
        deadConnections.push(existingConnection)
      }
    }

    log(`dead connection`, deadConnections)

    for (const deadConnection of deadConnections) {
      this.libp2p.connectionManager.onDisconnect(deadConnection)
    }

    if (stream != undefined && conn != undefined) {
      return { conn, stream }
    }
  }

  private async establishNewConnection(
    destination: PeerId,
    destinationAddress: Multiaddr,
    timeout: number
  ): Promise<ConnResult | void> {
    const abort = new AbortController()
    let done = false

    setTimeout(() => {
      if (!done) {
        abort.abort()
      }
    }, timeout)

    let conn: Connection | undefined
    try {
      conn = await this.dialDirectly(destinationAddress, { signal: abort.signal })
    } catch (err: any) {
      error(`error while contacting entry node ${destination.toB58String()}.`, err.message)
      await attemptClose(conn, error)
    }

    // Prevent timeout from doing anything
    done = true

    if (conn == undefined) {
      return
    }

    let stream: Stream | undefined
    try {
      stream = (await conn.newStream([CAN_RELAY_PROTCOL(this.options.environment)]))?.stream as any
    } catch (err) {
      error(`Cannot use relay.`, err)
      await attemptClose(conn, error)
    }

    if (conn != undefined && stream != undefined) {
      return {
        conn,
        stream
      }
    }
  }

  /**
   * Attempts to connect to a relay node
   * @param id peerId of the node to dial
   * @param relay multiaddr to perform the dial
   * @param timeout when to timeout on unsuccessful dial attempts
   * @returns a PeerStoreEntry containing the measured latency
   */
  private async connectToRelay(
    id: PeerId,
    relay: Multiaddr,
    timeout: number
  ): Promise<{ entry: EntryNodeData; conn?: Connection }> {
    const start = Date.now()

    let conn = await this.tryExistingConnections(id)

    if (!conn) {
      conn = await this.establishNewConnection(id, relay, timeout)
    }

    if (conn == undefined) {
      return {
        entry: {
          id,
          multiaddrs: [relay],
          latency: -1
        }
      }
    }

    let done = false

    // calls the iterator, thereby starts the stream and
    // consumes the first messages, afterwards closes the stream
    for await (const msg of conn.stream.source) {
      verbose(`can relay received ${new TextDecoder().decode(msg.slice())} from ${id.toB58String()}`)
      if (u8aEquals(msg.slice(), OK)) {
        done = true
      }
      // End receive stream after first message
      break
    }

    try {
      // End the send stream by sending nothing
      conn.stream.sink((async function* () {})()).catch(error)
    } catch (err) {
      error(err)
    }

    if (done) {
      return {
        conn: conn.conn,
        entry: {
          id,
          multiaddrs: [relay],
          latency: Date.now() - start
        }
      }
    } else {
      return {
        entry: {
          id,
          multiaddrs: [relay],
          latency: -1
        }
      }
    }
  }
}
