import type { HoprConnectOptions, PeerStoreType, StreamType } from '../types.js'
import type { Connection } from '@libp2p/interface-connection'
import type { PeerId } from '@libp2p/interface-peer-id'
import type { Initializable, Components } from '@libp2p/interfaces/components'
import type { Startable } from '@libp2p/interfaces/startable'
import { Multiaddr } from '@multiformats/multiaddr'
import { peerIdFromString } from '@libp2p/peer-id'
import { handshake } from 'it-handshake'

import errCode from 'err-code'
import { EventEmitter } from 'events'
import Debug from 'debug'

import {
  CODE_IP4,
  CODE_IP6,
  CODE_TCP,
  CODE_UDP,
  MAX_RELAYS_PER_NODE,
  MIN_RELAYS_PER_NODE,
  CAN_RELAY_PROTOCOLS,
  OK,
  DEFAULT_DHT_ENTRY_RENEWAL,
  CODE_P2P
} from '../constants.js'

import {
  dial,
  createCircuitAddress,
  nAtATime,
  oneAtATime,
  retimer,
  u8aEquals,
  retryWithBackoffThenThrow,
  durations,
  defer,
  DialStatus
} from '@hoprnet/hopr-utils'
import { attemptClose, relayFromRelayAddress } from '../utils/index.js'

const DEBUG_PREFIX = 'hopr-connect:entry'
const log = Debug(DEBUG_PREFIX)
const error = Debug(DEBUG_PREFIX.concat(':error'))

const ENTRY_NODE_CONTACT_TIMEOUT = 5e3
const ENTRY_NODES_MAX_PARALLEL_DIALS = 14

const DEFAULT_ENTRY_NODE_RECONNECT_BASE_TIMEOUT = 10e3
const DEFAULT_ENTRY_NODE_RECONNECT_BACKOFF = 2

const KNOWN_DISCONNECT_ERROR = `Not successful`

type EntryNodeData = PeerStoreType & {
  latency: number
}

type ConnectionResult = {
  entry: EntryNodeData
  conn?: Connection
}

type Grouped = [id: string, results: (ConnectionResult | Error | undefined)[]]

/**
 * Compare function to sort EntryNodeData ascending in latency
 */
function latencyCompare(a: EntryNodeData, b: EntryNodeData) {
  return a.latency - b.latency
}

enum ResultClass {
  AVAILABLE = 0,
  TIMEOUT = 1,
  ERROR = 2,
  UNCHECKED = 3
}

function connectionResultToNumber(res: ConnectionResult | undefined | Error): ResultClass {
  if (res == undefined) {
    return ResultClass.UNCHECKED
  }

  if (res instanceof Error) {
    return ResultClass.ERROR
  }

  if (res.entry.latency < 0) {
    return ResultClass.TIMEOUT
  }

  return ResultClass.AVAILABLE
}

/**
 * Compare function to sort results, such that:
 *
 * positive latencies, sorted ascending in latency
 * negative latencies (timeout)
 * Error (something went wrong)
 * undefined (break condition reached, no result)
 */
function compareConnectionResults(a: ConnectionResult | undefined | Error, b: ConnectionResult | undefined | Error) {
  const first = connectionResultToNumber(a)
  const second = connectionResultToNumber(b)

  switch (first) {
    case ResultClass.UNCHECKED:
    case ResultClass.ERROR:
    case ResultClass.TIMEOUT:
      return first - second
    case ResultClass.AVAILABLE:
      switch (second) {
        case ResultClass.UNCHECKED:
        case ResultClass.ERROR:
        case ResultClass.TIMEOUT:
          return first - second
        case ResultClass.AVAILABLE:
          return (a as ConnectionResult).entry.latency - (b as ConnectionResult).entry.latency
        default:
          throw Error(`Invalid result class. Got ${second}`)
      }
    default:
      throw Error(`Invalid result class. Got ${first}`)
  }
}

/**
 * Check if given Multiaddr can be used as an entry node
 * @param ma addr to check
 * @returns true if given Multiaddr can be used as entry node
 */
function isUsableRelay(ma: Multiaddr): boolean {
  const tuples = ma.tuples() as [code: number, addr: Uint8Array][]

  return (
    tuples[0].length >= 2 && [CODE_IP4, CODE_IP6].includes(tuples[0][0]) && [CODE_UDP, CODE_TCP].includes(tuples[1][0])
  )
}

/**
 * Transform dialed addrs + corresponding connection results to more
 * usable data structure.
 * @param args addrs used to connect
 * @param results connection result for each address
 * @returns
 */
function groupConnectionResults(
  args: Iterable<Parameters<EntryNodes['connectToRelay']>>,
  results: Iterable<ConnectionResult | Error | undefined>
): Grouped[] {
  const grouped: { [index: string]: (ConnectionResult | Error | undefined)[] } = {}

  const argIterator = args[Symbol.iterator]()
  const resultsIterator = results[Symbol.iterator]()

  let arg = argIterator.next()
  let result = resultsIterator.next()

  while (!arg.done && !result.done) {
    const address = arg.value[0].toString()

    if (grouped[address] == undefined) {
      grouped[address] = [result.value]
    } else {
      grouped[address].push(result.value)
    }

    arg = argIterator.next()
    result = resultsIterator.next()
  }

  // Sort each list of results and sort the entire list of lists after lowest latency of the first result
  const sorted: Grouped[] = Object.entries(grouped)
    .map<Grouped>((value) => [value[0], value[1].sort(compareConnectionResults)])
    .sort((a: Grouped, b: Grouped) => compareConnectionResults(a[1][0], b[1][0]))

  return sorted
}

/**
 * Renders connection results to a string
 *
 * @param grouped grouped and sorted connection results
 * @param prefix string to print before listing the results
 * @returns string to log
 */
function printGroupedConnectionResults(grouped: Iterable<Grouped>, prefix: string = ''): string {
  let out = `${prefix}\n`

  for (const [id, results] of grouped) {
    // First entry is always the best result, so considering only first one
    if (results[0] == undefined) {
      continue
    }

    out += `  - ${id}: ${
      results[0] instanceof Error
        ? 'Error'
        : results[0].entry.latency < 0
        ? 'Timeout'
        : `${results[0].entry.latency} ms`
    }\n`
  }

  // Remove last occurence of `/n`
  return out.substring(0, out.length - 1)
}

/**
 * Renders currently used relays to a string
 *
 * @param relays used relays
 * @param prefix string to print before listing the results
 * @returns
 */
function printListOfUsedRelays(relays: Iterable<UsedRelay>, prefix: string = ''): string {
  let out = `${prefix}\n`

  for (const relay of relays) {
    out += `  - ${relayFromRelayAddress(relay.ourCircuitAddress).toString()}\n`
  }

  // Remove last occurence of `/n`
  return out.substring(0, out.length - 1)
}

type UsedRelay = {
  relayDirectAddress: Multiaddr
  ourCircuitAddress: Multiaddr
}

// Emitted whenever list of relays changed
export const RELAY_CHANGED_EVENT = 'relay:changed'

/**\
 * Manages known entry nodes and provides a list of those
 * that are currently used as relays.
 *
 * ┌───────────────────────┐     ┌─ ┌─────────┐
 * │ usedRelays            ├─────┤  │unchecked│
 * │                       │     │  └─────────┘
 * │<= MAX_RELAYS_PER_NODE │     │
 * │                       │     │  ┌─────────┐
 * │>= MIN_RELAYS_PER_NODE │     │  │available│
 * │                       │     │  └─────────┘
 * └───────────────────────┘     │
 *                               │  ┌─────────┐
 *                               │  │offline  │
 *                               └─ └─────────┘
 */
export class EntryNodes extends EventEmitter implements Initializable, Startable {
  // Nodes with good availability
  protected availableEntryNodes: EntryNodeData[]
  // New nodes with unclear availability
  protected uncheckedEntryNodes: PeerStoreType[]
  // Nodes that have been offline in the past
  protected offlineEntryNodes: PeerStoreType[]

  // Threshold of entry nodes to connect to
  private maxRelaysPerNode: number
  // Lower bound once node will try to rebuild list
  private minRelaysPerNode: number
  // How many calls to do in parallel
  private maxParallelDials: number
  // Timeout once node is considered offline
  private contactTimeout: number
  // The DHT comes with an auto-clean mechanism that evicts old entries.
  // To keep the entry, the node needs to renew it
  private dhtRenewalInterval: number

  private stopDHTRenewal: (() => void) | undefined
  private stopReconnectAttempts: (() => void) | undefined

  protected usedRelays: UsedRelay[]

  private _isStarted: boolean
  private enabled: boolean

  private components: Components | undefined

  private _onNewRelay: ((peer: PeerStoreType) => void) | undefined
  private _onRemoveRelay: ((peer: PeerId) => void) | undefined

  private disconnectListener: ((conn: CustomEvent<Connection>) => void) | undefined

  private addToUpdateQueue: ReturnType<typeof oneAtATime>

  constructor(
    private options: HoprConnectOptions,
    overwrites?: {
      maxRelaysPerNode?: number
      minRelaysPerNode?: number
      maxParallelDials?: number
      contactTimeout?: number
    }
  ) {
    super()

    this.enabled = false

    this.maxRelaysPerNode = overwrites?.maxRelaysPerNode ?? MAX_RELAYS_PER_NODE
    this.minRelaysPerNode = overwrites?.minRelaysPerNode ?? MIN_RELAYS_PER_NODE
    this.contactTimeout = overwrites?.contactTimeout ?? ENTRY_NODE_CONTACT_TIMEOUT
    this.maxParallelDials = overwrites?.maxParallelDials ?? ENTRY_NODES_MAX_PARALLEL_DIALS
    this.dhtRenewalInterval = options.dhtRenewalTimeout ?? DEFAULT_DHT_ENTRY_RENEWAL

    if (this.minRelaysPerNode > this.maxRelaysPerNode) {
      throw Error(`Invalid configuration. minRelaysPerNode must be smaller or equal to maxRelaysPerNode`)
    }

    if (this.contactTimeout >= this.dhtRenewalInterval) {
      throw Error(`Invalid configuration. contactTimeout must be strictly smaller than dhtRenewalTimeout`)
    }

    // FIFO-Queue that manages all connect and reconnect attempts
    this.addToUpdateQueue = oneAtATime()

    this._isStarted = false
    this.availableEntryNodes = []
    this.uncheckedEntryNodes = options.initialNodes ?? []
    this.offlineEntryNodes = []

    this.usedRelays = []

    this.connectToRelay = this.connectToRelay.bind(this)
  }

  /**
   * Enables monitoring of entry nodes. Called by Listener
   * once it is clear that nodes requires relayed connections.
   *
   * If not called, node won't contact any entry nodes or
   * monitor their availability.
   */
  public enable() {
    this.enabled = true
  }

  public isStarted() {
    return this._isStarted
  }

  public start() {
    this._isStarted = true
  }

  /**
   * Attaches listeners that handle addition and removal of
   * entry nodes
   */
  public async afterStart() {
    if (this.options.publicNodes != undefined) {
      this._onNewRelay = function (this: EntryNodes, peer: PeerStoreType) {
        this.addToUpdateQueue(async () => {
          log(`peer online`, peer.id.toString())
          await this.onNewRelay(peer)
        })
      }.bind(this)
      this._onRemoveRelay = function (this: EntryNodes, peer: PeerId) {
        this.addToUpdateQueue(async () => {
          log(`peer offline`, peer.toString())
          await this.onRemoveRelay(peer)
        })
      }.bind(this)

      this.options.publicNodes.on('addPublicNode', this._onNewRelay)
      this.options.publicNodes.on('removePublicNode', this._onRemoveRelay)
    }

    if (this.enabled) {
      this.startDHTRenewInterval()

      await new Promise((resolve, reject) => {
        this.addToUpdateQueue(() => this.updatePublicNodes().then(resolve, reject))
      })
    }
  }

  /**
   * Removes event listeners
   */
  public stop() {
    if (!this._isStarted) {
      return
    }

    if (this.options.publicNodes != undefined && this._onNewRelay != undefined && this._onRemoveRelay != undefined) {
      this.options.publicNodes.removeListener('addPublicNode', this._onNewRelay)

      this.options.publicNodes.removeListener('removePublicNode', this._onRemoveRelay)
    }

    if (this.enabled) {
      this.stopReconnectAttempts?.()
      this.stopDHTRenewal?.()
    }

    this._isStarted = false
  }

  public init(components: Components) {
    this.components = components
  }

  public getComponents(): Components {
    if (this.components == null) {
      throw errCode(new Error('components not set'), 'ERR_SERVICE_MISSING')
    }

    return this.components
  }

  /**
   * Iterates through all sets of known entry nodes to find
   * given peer. Once found, apply given function `fn` to entry and
   * return the result of the invocation of `fn`.
   * @param id node to find
   * @param fn function to apply on entry
   * @param throwIfNotFound [default=true] whether to throw if not found
   * @returns
   */
  private someNode<Return>(
    id: PeerId,
    fn: (entry: EntryNodeData | PeerStoreType, index: number, addrsList: (EntryNodeData | PeerStoreType)[]) => Return,
    throwIfNotFound: boolean = true
  ): Return | undefined {
    for (const nodeList of [this.uncheckedEntryNodes, this.availableEntryNodes, this.offlineEntryNodes]) {
      for (const [index, node] of nodeList.entries()) {
        if (node.id.equals(id)) {
          return fn(node, index, nodeList)
        }
      }
    }

    if (throwIfNotFound) {
      throw Error(`No entry found for ${id.toString()}`)
    }
  }

  /**
   * Used to track once a connection to a selected entry node got closed.
   */
  private trackEntryNodeConnections(): void {
    const usedRelays = new Set(this.getUsedRelayPeerIds().map((p: PeerId) => p.toString()))

    if (this.disconnectListener != undefined) {
      this.getComponents().getConnectionManager().removeEventListener('peer:disconnect', this.disconnectListener)
    }
    this.disconnectListener = (event: CustomEvent<Connection>) => {
      const remotePeer = event.detail.remotePeer

      if (usedRelays.has(remotePeer.toString())) {
        log(`Disconnected from entry node ${remotePeer.toString()}, trying to reconnect ...`)
        this.addToUpdateQueue(() => this.reconnectToEntryNode(remotePeer))
      }
    }

    this.getComponents().getConnectionManager().addEventListener('peer:disconnect', this.disconnectListener)
  }

  private async reconnectToEntryNode(peer: PeerId) {
    const addrToContact = (
      this.someNode(peer, (addrs: EntryNodeData | PeerStoreType) => addrs.multiaddrs) as Multiaddr[]
    ).map<[id: PeerId, ma: Multiaddr]>((entry) => [peer, entry])

    let attempt = 0

    try {
      await retryWithBackoffThenThrow(
        async () => {
          attempt++
          const results = await nAtATime(this.connectToRelay, addrToContact, this.maxParallelDials)

          log(
            printGroupedConnectionResults(
              groupConnectionResults(addrToContact, results),
              `Connection result to entry nodes at entry node disconnect`
            )
          )

          if (
            results.every(
              (result: ConnectionResult | Error | undefined) =>
                result == undefined || result instanceof Error || result.entry.latency < 0
            )
          ) {
            // Throw error to signal `retryWithBackoff` that dial attempt
            // was not successful
            log(`Reconnect attempt ${attempt} to entry node ${peer.toString()} was not successful`)
            throw Error(KNOWN_DISCONNECT_ERROR)
          }

          log(`Reconnect attempt ${attempt} to entry node ${peer.toString()} was successful`)
        },
        {
          minDelay: this.options.entryNodeReconnectBaseTimeout ?? DEFAULT_ENTRY_NODE_RECONNECT_BASE_TIMEOUT,
          maxDelay: 10 * (this.options.entryNodeReconnectBaseTimeout ?? DEFAULT_ENTRY_NODE_RECONNECT_BASE_TIMEOUT),
          delayMultiple: this.options.entryNodeReconnectBackoff ?? DEFAULT_ENTRY_NODE_RECONNECT_BACKOFF
        }
      )
    } catch (err: any) {
      log(`Gave up reconnecting to ${peer.toString()}`)

      if (err.message !== KNOWN_DISCONNECT_ERROR) {
        // Forward unexpected errors
        throw err
      } else {
        // Peer is offline, rebuild list if below threshold
        if (this.usedRelays.length - 1 < this.minRelaysPerNode) {
          log(
            `Number of connected entry nodes (${this.usedRelays.length}) has fallen below threshold of ${this.minRelaysPerNode}, rebuilding list of entry nodes`
          )
          await this.updatePublicNodes()
        } else {
          this.updateUsedRelays([[peer.toString(), [undefined]]])
          this.trackEntryNodeConnections()
          // Publish a new DHT entry
          this.emit(RELAY_CHANGED_EVENT)
        }
      }
    }
  }

  private async renewDHTEntries() {
    const work: Parameters<EntryNodes['connectToRelay']>[] = []
    log(`Renewing DHT entry for selected entry nodes`)

    for (const relay of this.getUsedRelayPeerIds()) {
      const relayEntry = this.someNode(relay, (addrs: PeerStoreType) => addrs.multiaddrs)

      if (relayEntry == undefined) {
        log(`Relay ${relay.toString()} has been removed from list of available entry nodes. Not renewing this entry`)
        // this.updateUsedRelays([[relay.toString(), [undefined]]])
        continue
      }

      const addrsToContact = this.someNode(relay, (addrs) => addrs.multiaddrs, true) as Multiaddr[]

      for (const ma of addrsToContact) {
        work.push([relay, ma])
      }
    }

    const results = groupConnectionResults(work, await nAtATime(this.connectToRelay, work, this.maxParallelDials))

    log(printGroupedConnectionResults(results, `Connection results to entry nodes at DHT renewal:`))

    this.updateUsedRelays(results)

    if (this.usedRelays.length < this.minRelaysPerNode) {
      log(
        `Renewing DHT entry has shown that number of connected entry nodes has fallen below threshold of ${this.minRelaysPerNode} nodes, reconnecting to entry nodes`
      )
      // full rebuild
      await this.updatePublicNodes()
    }
  }

  /**
   * Start the interval in which the entry in the DHT gets renewed.
   */
  private startDHTRenewInterval() {
    this.stopDHTRenewal = retimer(
      () => {
        const entriesRenewed = defer<void>()
        this.addToUpdateQueue(async () => {
          await this.renewDHTEntries()
          entriesRenewed.resolve()
        })
        // Don't do a renew before the previous attempt
        // has finished
        return entriesRenewed.promise
      },
      () => this.dhtRenewalInterval
    )
  }

  private startReconnectAttemptInterval() {
    let allowed = false
    let initialDelay = durations.minutes(1)
    this.stopReconnectAttempts = retimer(
      async () => {
        if (
          !allowed &&
          this.options.isAllowedToAccessNetwork != undefined &&
          !(await this.options.isAllowedToAccessNetwork(this.getComponents().getPeerId()))
        ) {
          log(
            `Node has not been registered and thus not allowed to access network. Skip trying to connect to entry nodes`
          )
          return
        } else {
          allowed = true
        }

        const reconnectAttemptFinished = defer<void>()
        this.addToUpdateQueue(async () => {
          await this.updatePublicNodes()
          reconnectAttemptFinished.resolve()
        })
        // Don't issue another attempt before previous
        // one has finished
        await reconnectAttemptFinished.promise
      },
      () => {
        if (!allowed) {
          return durations.seconds(30)
        }

        return (initialDelay *= 1.5)
      }
    )
  }
  /**
   * @returns a list of entry nodes that are currently used (as relay circuit addresses with us)
   */
  public getUsedRelayAddresses() {
    return this.usedRelays.map((ur: UsedRelay) => ur.ourCircuitAddress)
  }

  /**
   * Convenience method to retrieved used relay peer IDs.
   * @returns a list of peer IDs of used relays.
   */
  public getUsedRelayPeerIds() {
    return this.getUsedRelayAddresses().map(relayFromRelayAddress)
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
   * @param peer PeerInfo of node that is added as a relay opportunity
   */
  protected async onNewRelay(peer: PeerStoreType): Promise<void> {
    if (peer.id.equals(this.getComponents().getPeerId())) {
      // Cannot use self as entry node
      return
    }

    if (peer.multiaddrs == undefined || peer.multiaddrs.length == 0) {
      log(`Received entry node ${peer.id.toString()} without any multiaddr, ignoring`)
      // Nothing to do
      return
    }

    let receivedNewAddrs = false

    let knownPeer = false

    this.someNode(
      peer.id,
      (entry: PeerStoreType | EntryNodeData) => {
        knownPeer = true

        log(
          `Received new address${peer.multiaddrs.length == 1 ? '' : 'es'} ${peer.multiaddrs
            .map((ma) => ma.decapsulateCode(CODE_P2P).toString())
            .join(', ')} for ${peer.id.toString()}`
        )

        // Check if we received any *new* address(es)
        const alreadyKnownAddrs = new Set(
          entry.multiaddrs.map((ma: Multiaddr) => ma.decapsulateCode(CODE_P2P).toString())
        )
        for (const ma of peer.multiaddrs) {
          if (!isUsableRelay(ma)) {
            continue
          }
          if (!alreadyKnownAddrs.has(ma.decapsulateCode(CODE_P2P).toString())) {
            entry.multiaddrs.push(ma.decapsulateCode(CODE_P2P).encapsulate(`/p2p/${peer.id.toString()}`))
            receivedNewAddrs = true
            break
          }
        }
      },
      false
    )

    if (!knownPeer) {
      this.uncheckedEntryNodes.push({
        id: peer.id,
        multiaddrs: peer.multiaddrs.filter(isUsableRelay)
      })
      receivedNewAddrs = true
    }

    // Stop adding and checking relay nodes if we already have enough.
    // Once a relay goes offline, the node will try to replace the offline relay.
    if (this.enabled && receivedNewAddrs && this.usedRelays.length < this.maxRelaysPerNode) {
      log(
        `Number of connected relay nodes (${this.usedRelays.length} nodes) below threshold of ${this.minRelaysPerNode}, using new addresses to rebuild list`
      )
      // Rebuild list of relay nodes later
      await this.updatePublicNodes()
    }
  }

  /**
   * Called once a node is considered to be offline
   * @param peer PeerId of node that is considered to be offline now
   */
  protected async onRemoveRelay(peer: PeerId) {
    for (const [index, publicNode] of this.availableEntryNodes.entries()) {
      if (publicNode.id.equals(peer)) {
        // Remove node without changing order
        this.availableEntryNodes.splice(index, 1)
        this.offlineEntryNodes.push(publicNode)

        // Assuming that node does not appear more than once
        break
      }
    }

    if (this.enabled) {
      let inUse = false
      for (const relayPeer of this.getUsedRelayPeerIds()) {
        // remove second part of relay address to get relay peerId
        if (relayPeer.equals(peer)) {
          inUse = true
          break
        }
      }

      // Only rebuild list of relay nodes if node is *in use*
      if (inUse) {
        if (this.usedRelays.length - 1 < this.minRelaysPerNode) {
          log(
            `Peer ${peer.toString()} appeared to be offline. Number of connected relay nodes (${
              this.usedRelays.length
            }) below threshold of ${this.minRelaysPerNode}, rebuilding list`
          )
          // full rebuild because below threshold
          await this.updatePublicNodes()
        } else {
          this.updateUsedRelays([[peer.toString(), [undefined]]])
          this.trackEntryNodeConnections()
          this.emit(RELAY_CHANGED_EVENT)
        }
      }
    }
  }

  /**
   * Returns arguments to call `connectToRelay`
   *
   * 1. unchecked nodes <- recently learned
   * 2. available nodes <- have been online once
   * 3. offline nodes <- have been marked offline but could be back
   *
   * @returns array of arguments to probe nodes
   */
  private async getAddrsToContact(): Promise<Parameters<EntryNodes['connectToRelay']>[]> {
    const args: Parameters<EntryNodes['connectToRelay']>[] = []

    for (const nodeList of [this.uncheckedEntryNodes, this.availableEntryNodes, this.offlineEntryNodes]) {
      for (const node of nodeList) {
        // In case the addrs are not known to libp2p
        await this.getComponents()
          .getPeerStore()
          .addressBook.add(
            node.id,
            node.multiaddrs.map((ma) => ma.decapsulateCode(CODE_P2P))
          )
        for (const ma of node.multiaddrs) {
          args.push([node.id, ma])
        }
      }
    }

    return args
  }

  /**
   * Rebuild the *entire* list of used entry nodes, assuming
   * that given connect results refer to current availability
   * of entry nodes.
   * @param groupedResults results from concurrent connect attempt
   */
  private rebuildUsedRelays(groupedResults: Iterable<Grouped>) {
    this.usedRelays = []

    // Assuming groupedResults is sorted
    for (const [_id, results] of groupedResults) {
      if (results[0] == undefined || results[0] instanceof Error) {
        break
      }

      if (results[0].entry.latency < 0) {
        for (const result of results) {
          if (result == undefined || result instanceof Error) {
            break
          }
          // Includes a catch-all block and swallows, but
          // logs all exceptions
          attemptClose((result as ConnectionResult).conn, error)
        }

        // Don't add entry node
        break
      }

      this.usedRelays.push({
        // @TODO take the address that really got used
        relayDirectAddress: (results[0] as ConnectionResult).entry.multiaddrs[0],
        ourCircuitAddress: createCircuitAddress((results[0] as ConnectionResult).entry.id)
      })
    }
  }

  /**
   * Updates specific entries of the list of used entry nodes
   * @param groupedResults results from concurrent connect attempt
   */
  private updateUsedRelays(groupedResults: Iterable<Grouped>) {
    for (const [id, results] of groupedResults) {
      if (results[0] == undefined || results[0] instanceof Error || results[0].entry.latency < 0) {
        // Mark offline
        this.someNode(
          peerIdFromString(id),
          (_addrs: EntryNodeData | PeerStoreType, index: number, nodeList: (EntryNodeData | PeerStoreType)[]) => {
            this.offlineEntryNodes.push(nodeList.splice(index, 1)[0])
          }
        )

        for (const [index, relay] of this.usedRelays.entries()) {
          if (relayFromRelayAddress(relay.ourCircuitAddress)) {
            this.usedRelays.splice(index, 1)
            break
          }
        }

        // results are sorted, so there is no better result
        break
      }
    }
  }

  /**
   * Updates knowledge about offline, available and unchecked entry nodes
   * @param groupedResults results from concurrent connect attempt
   */
  protected updateRecords(groupedResults: Iterable<Grouped>) {
    // @TODO replace this by a more efficient data structure
    const availableOnes = new Set<string>(
      function* (this: EntryNodes) {
        for (const nodeData of this.availableEntryNodes) {
          yield nodeData.id.toString()
        }
      }.call(this)
    )
    const uncheckedOnes = new Set<string>(
      function* (this: EntryNodes) {
        for (const nodeData of this.uncheckedEntryNodes) {
          yield nodeData.id.toString()
        }
      }.call(this)
    )
    const offlineOnes = new Set<string>(
      function* (this: EntryNodes) {
        for (const nodeData of this.offlineEntryNodes) {
          yield nodeData.id.toString()
        }
      }.call(this)
    )

    for (const [id, results] of groupedResults) {
      if (results.every((result) => result == undefined)) {
        // Nothing to do because no additional good or bad knowledge
        continue
      }

      if (results.every((result) => result instanceof Error || (result != undefined && result.entry.latency < 0))) {
        if (availableOnes.has(id)) {
          for (const [i, available] of this.availableEntryNodes.entries()) {
            if (available.id.toString() === id) {
              // move from available to offline
              this.offlineEntryNodes.push(this.availableEntryNodes.splice(i, 1)[0])
              break
            }
          }
        } else if (uncheckedOnes.has(id)) {
          for (const [i, unchecked] of this.uncheckedEntryNodes.entries()) {
            if (unchecked.id.toString() === id) {
              // move from unchecked to offline
              this.offlineEntryNodes.push(this.uncheckedEntryNodes.splice(i, 1)[0])
              break
            }
          }
        }

        // none of the results if usable, so move forward
        continue
      }

      // At least one of the results has been non-negative, now get the lowest one
      let lowestPositiveLatency = (results[0] as ConnectionResult).entry.latency

      if (offlineOnes.has(id)) {
        for (const [i, offline] of this.offlineEntryNodes.entries()) {
          if (offline.id.toString() === id) {
            // move from offlne to available
            this.availableEntryNodes.push({
              ...this.offlineEntryNodes.splice(i, 1)[0],
              latency: lowestPositiveLatency
            })
            break
          }
        }
      } else if (uncheckedOnes.has(id)) {
        for (const [i, unchecked] of this.uncheckedEntryNodes.entries()) {
          if (unchecked.id.toString() === id) {
            // move from unchecked to available
            this.availableEntryNodes.push({
              ...this.uncheckedEntryNodes.splice(i, 1)[0],
              latency: lowestPositiveLatency
            })
            break
          }
        }
      }
    }

    // sorts in-place
    this.availableEntryNodes.sort(latencyCompare)
  }

  /**
   * Updates the list of exposed entry nodes.
   * Called at startup and once an entry node is considered offline.
   */
  async updatePublicNodes(): Promise<void> {
    log(`Updating list of used relay nodes ...`)

    const addrsToContact = await this.getAddrsToContact()

    const start = Date.now()
    const results = groupConnectionResults(
      addrsToContact,
      await nAtATime(
        this.connectToRelay,
        addrsToContact,
        this.maxParallelDials,
        (results: (Awaited<ReturnType<EntryNodes['connectToRelay']>> | Error | undefined)[]) => {
          let availableNodes = 0

          // @TODO this assumes that there is only address per entry node that leads
          //       to a usable connection. In some networks, this might not be true.
          for (const result of results) {
            if (result != undefined && !(result instanceof Error) && result.entry.latency >= 0) {
              availableNodes++
              if (availableNodes >= this.maxRelaysPerNode) {
                return true
              }
            }
          }

          return false
        }
      )
    )

    log(printGroupedConnectionResults(results, `Connection results after contacting entry nodes`))

    this.updateRecords(results)

    log(
      `Probing ${results.filter((result) => result != undefined).length} potential entry node addresses took ${
        Date.now() - start
      } ms.`
    )

    const previouslyUsedRelays = new Set<string>(this.getUsedRelayPeerIds().map((p: PeerId) => p.toString()))

    this.rebuildUsedRelays(results)

    let isDifferent = false

    if (this.usedRelays.length != previouslyUsedRelays.size) {
      isDifferent = true
    } else {
      for (const previouslyUsedRelay of this.getUsedRelayPeerIds()) {
        if (!previouslyUsedRelays.has(previouslyUsedRelay.toString())) {
          isDifferent = true
          break
        }
      }
    }

    // If we haven't been able to find an entry node to which we can connect,
    // start an interval to try again until we find some entry nodes to
    // which we can successfully connect to.
    if (this.stopReconnectAttempts == undefined && (this.usedRelays == undefined || this.usedRelays.length == 0)) {
      this.startReconnectAttemptInterval()
    }

    // Once we got at least one entry node, stop the interval
    if (this.usedRelays != undefined && this.usedRelays.length > 0) {
      this.stopReconnectAttempts?.()
    }

    // Only emit events and debug log if something has changed.
    // This reduces debug log noise.
    if (isDifferent) {
      log(printListOfUsedRelays(this.usedRelays, `Updated list of entry nodes:`))

      this.trackEntryNodeConnections()
      this.emit(RELAY_CHANGED_EVENT)
    }
  }

  /**
   * Attempts to connect to a relay node
   * @param id peerId of the node to dial
   * @param relay multiaddr to perform the dial
   * @returns a PeerStoreEntry containing the measured latency
   */
  private async connectToRelay(id: PeerId, relay: Multiaddr): Promise<{ entry: EntryNodeData; conn?: Connection }> {
    const result = await dial(
      this.getComponents(),
      id,
      CAN_RELAY_PROTOCOLS(this.options.network, this.options.supportedNetworks),
      false,
      true
    )

    if (result.status != DialStatus.SUCCESS) {
      // Dial error
      return {
        entry: {
          id,
          multiaddrs: [relay],
          latency: -1
        }
      }
    }

    const start = Date.now()

    // Only reads from socket once but keeps socket open
    // to eventually use it to exchange signalling information
    const shaker = handshake(result.resp.stream)

    let chunk: StreamType | undefined
    try {
      chunk = await shaker.read()
    } catch (err) {
      error(err)
      return {
        entry: {
          id,
          multiaddrs: [relay],
          latency: -1
        }
      }
    }

    if (chunk == undefined || !u8aEquals(chunk.subarray(), OK)) {
      return {
        entry: {
          id,
          multiaddrs: [relay],
          latency: -1
        }
      }
    }

    return {
      entry: {
        id,
        multiaddrs: [relay],
        latency: Date.now() - start
      }
    }
  }
}
