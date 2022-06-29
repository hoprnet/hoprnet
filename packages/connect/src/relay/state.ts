import type { Stream } from '../types.js'
import type { PeerId } from '@libp2p/interface-peer-id'
import { unmarshalPublicKey } from '@libp2p/crypto/keys'

import { nAtATime, u8aCompare } from '@hoprnet/hopr-utils'
import { RelayContext } from './context.js'

import debug from 'debug'

const DEBUG_PREFIX = 'hopr-connect:relay:state'

const verbose = debug(DEBUG_PREFIX.concat(':verbose'))
const error = debug(DEBUG_PREFIX.concat(':error'))

type RelayConnections = {
  [id: string]: RelayContext
}

/**
 * Encapsulates open relayed connections
 */
class RelayState {
  private relayedConnections: Map<string, RelayConnections>

  constructor() {
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
  updateExisting(source: PeerId, destination: PeerId, toSource: Stream): void {
    const id = RelayState.getId(source, destination)

    if (!this.relayedConnections.has(id)) {
      throw Error(`Relayed connection does not exist`)
    }

    const context = this.relayedConnections.get(id) as RelayConnections

    context[source.toString()].update(toSource)
  }

  /**
   * Performs an operation for each relay context in the current set.
   * @param action
   */
  async forEach(action: (dst: string, ctx: RelayContext) => Promise<void>) {
    await nAtATime(
      (objEntries) => action(objEntries[0], objEntries[1]),
      Array.from(this.relayedConnections.values()).map((s) => Object.entries(s)),
      10 // TODO: Make this configurable or use an existing constant
    )
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
    const toSourceContext = new RelayContext(toSource, __relayFreeTimeout)
    const toDestinationContext = new RelayContext(toDestination, __relayFreeTimeout)

    let sourcePromise = toSourceContext.sink(toDestinationContext.source)
    let destinationPromise = toDestinationContext.sink(toSourceContext.source)

    let relayedConnection: RelayConnections = {
      [source.toString()]: toSourceContext,
      [destination.toString()]: toDestinationContext
    }

    toSourceContext.once('close', this.cleanListener(source, destination))
    toSourceContext.once('close', this.cleanListener(destination, source))

    toSourceContext.once('upgrade', this.cleanListener(source, destination))
    toSourceContext.once('upgrade', this.cleanListener(destination, source))

    this.relayedConnections.set(RelayState.getId(source, destination), relayedConnection)

    try {
      await Promise.all([sourcePromise, destinationPromise])
    } catch (err) {
      this.relayedConnections.delete(RelayState.getId(source, destination))
      throw err
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

      let found = this.relayedConnections.get(id)

      if (found) {
        delete found[source.toString()]

        if (!found.hasOwnProperty(destination.toString())) {
          this.relayedConnections.delete(id)
        }
      }
    }.bind(this)
  }

  /**
   * Creates an identifier that is used to store the relayed connection
   * instance.
   * @param a peerId of first party
   * @param b peerId of second party
   * @returns the identifier
   */
  static getId(a: PeerId, b: PeerId): string {
    const cmpResult = u8aCompare(
      unmarshalPublicKey(a.publicKey as Uint8Array).marshal(),
      unmarshalPublicKey(b.publicKey as Uint8Array).marshal()
    )

    switch (cmpResult) {
      case 1:
        return `${a.toString()}${b.toString()}`
      case -1:
        return `${b.toString()}${a.toString()}`
      default:
        throw Error(`Invalid compare result. Loopbacks are not allowed.`)
    }
  }
}

export { RelayState }
