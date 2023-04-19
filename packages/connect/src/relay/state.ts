import type { HoprConnectOptions, Stream } from '../types.js'
import type { PeerId } from '@libp2p/interface-peer-id'

import { nAtATime } from '@hoprnet/hopr-utils'
import { RelayContext, type RelayContextInterface } from './context.js'

import debug from 'debug'

const DEBUG_PREFIX = 'hopr-connect:relay:state'

const verbose = debug(DEBUG_PREFIX.concat(':verbose'))
const error = debug(DEBUG_PREFIX.concat(':error'))

type RelayConnections = {
  [id: string]: RelayContextInterface
}

/**
 * Encapsulates open relayed connections
 */
class RelayState {
  private relayedConnections: Map<string, RelayConnections>

  constructor(public options: HoprConnectOptions) {
    this.relayedConnections = new Map()
  }

  relayedConnectionCount(): number {
    return this.relayedConnections.size
  }

  /**
   * Checks if there is a relayed connection. Liveness is not checked,
   * so connection might be dead.
   * @param source initiator of the relayed connection
   * @param destination other party of the relayed connection
   * @returns if there is already a relayed connection
   */
  exists(source: PeerId, destination: PeerId): boolean {
    const id = RelayState.getId(source, destination)

    return this.relayedConnections.has(id)
  }

  /**
   * Performs a liveness test on the connection
   * @param source initiator of the relayed connection
   * @param destination other party of the relayed connection
   * @param timeout timeout in miliseconds before connection is considered dead
   * @returns a promise that resolves to true if connection is active
   */
  async isActive(source: PeerId, destination: PeerId, timeout?: number): Promise<boolean> {
    const id = RelayState.getId(source, destination)
    if (!this.relayedConnections.has(id)) {
      verbose(`Connection from ${source.toString()} to ${destination.toString()} does not exist.`)
      return false
    }

    const context = this.relayedConnections.get(id) as RelayConnections

    let latency: number
    try {
      latency = await context[destination.toString()].ping(timeout)
    } catch (err) {
      error(err)
      return false
    }

    if (latency >= 0) {
      verbose(`Connection from ${source.toString()} to ${destination.toString()} is active.`)
      return true
    }

    error(`Connection from ${source.toString()} to ${destination.toString()} is NOT active.`)
    return false
  }

  /**
   * Attaches (and thereby overwrites) an existing stream to source.
   * Initiates a stream handover.
   * @param source initiator of the relayed connection
   * @param destination other party of the relayed connection
   * @param toSource new stream to source
   */
  updateExisting(source: PeerId, destination: PeerId, toSource: Stream): boolean {
    const id = RelayState.getId(source, destination)

    const context = this.relayedConnections.get(id)

    if (context == null) {
      verbose(`Relayed connection between ${source.toString()} and ${destination.toString()} does not exist`)
      return false
    }

    context[source.toString()].update(toSource)

    return true
  }

  /**
   * Returns an iterator over all stored relayed connections
   */
  *[Symbol.iterator]() {
    const it = this.relayedConnections.values()

    let chunk = it.next()

    for (; !chunk.done; chunk = it.next()) {
      yield* Object.entries(chunk.value)
    }
  }

  /**
   * Performs an operation for each relay context in the current set.
   * @param action
   */
  async forEach(action: (dst: string, ctx: RelayContextInterface) => Promise<void>) {
    await nAtATime(
      action,
      this,
      10 // TODO: Make this configurable or use an existing constant
    )
  }

  async printIds() {
    let ret: string[] = []
    for await (let cid of this.relayedConnections.keys()) {
      ret.push(cid)
    }
    return ret.join(',')
  }

  /**
   * Deletes the inactive relay entry given the source and destination
   * @param source
   * @param destination
   */
  delete(source: PeerId, destination: PeerId) {
    this.relayedConnections.delete(RelayState.getId(source, destination))
  }

  /**
   * Creates and stores a new relayed connection
   * This function returns only when the relay connection is terminated.
   * @param source initiator of the relayed connection
   * @param destination other party of the relayed connection
   * @param toSource duplex stream to source
   * @param toDestination duplex stream to destination
   * @param __relayFreeTimeout
   */
  async createNew(
    source: PeerId,
    destination: PeerId,
    toSource: Stream,
    toDestination: Stream,
    __relayFreeTimeout?: number
  ): Promise<void> {
    const toSourceContext = RelayContext(
      toSource,
      { onClose: this.cleanListener(source, destination), onUpgrade: this.cleanListener(source, destination) },
      this.options
    )
    const toDestinationContext = RelayContext(
      toDestination,
      {
        onClose: this.cleanListener(destination, source),
        onUpgrade: this.cleanListener(destination, source)
      },
      this.options
    )

    let sourcePromise = toSourceContext.sink(toDestinationContext.source)
    let destinationPromise = toDestinationContext.sink(toSourceContext.source)

    let relayedConnection: RelayConnections = {
      [source.toString()]: toSourceContext,
      [destination.toString()]: toDestinationContext
    }

    this.relayedConnections.set(RelayState.getId(source, destination), relayedConnection)

    try {
      Promise.all([sourcePromise, destinationPromise]).catch((err) => {
        this.relayedConnections.delete(RelayState.getId(source, destination))
        error(`Could not create new relay connection between ${source.toString()} and ${destination.toString()}`, err)
      })
    } catch (err) {
      this.relayedConnections.delete(RelayState.getId(source, destination))
      error(`Could not create new relay connection between ${source.toString()} and ${destination.toString()}`, err)
    }
  }

  /**
   * Creates a listener that cleans the relayed state once connection is closed
   * @param source initiator of the relayed connection
   * @param destination other party of the relayed connection
   * @returns a listener
   */
  private cleanListener(source: PeerId, destination: PeerId): () => void {
    return function (this: RelayState) {
      const id = RelayState.getId(source, destination)

      this.relayedConnections.delete(id)
    }.bind(this)
  }

  public async prune(timeout?: number) {
    if (this.relayedConnections.size == 0) return 0

    let pruned = 0
    await Promise.all(
      Array.from(this.relayedConnections.entries()).map(async ([id, ctx]) => {
        for (let [_, conn] of Object.entries(ctx)) {
          try {
            if ((await conn.ping(timeout)) < 0) {
              if (this.relayedConnections.delete(id)) {
                ++pruned
              } else {
                error(`could not delete ${id} inactive relayed connection from the relay state`)
              }
              break
            }
          } catch (err) {
            error(err)
          }
        }
      })
    )

    return pruned
  }

  /**
   * Creates an identifier that is used to store the relayed connection
   * instance.
   * @param a peerId of first party
   * @param b peerId of second party
   * @returns the identifier
   */
  static getId(a: PeerId, b: PeerId): string {
    let aStr = a.toString()
    let bStr = b.toString()

    const cmpResult = aStr.localeCompare(bStr)

    // human-readable ID
    switch (cmpResult) {
      case 1:
        return `${aStr}-${bStr}`
      case -1:
        return `${bStr}-${aStr}`
      default:
        throw Error(`Invalid compare result. Loopbacks are not allowed.`)
    }
  }
}

export { RelayState }
