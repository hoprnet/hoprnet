import type { HoprConnectOptions, Stream } from '../types.js'
import type { PeerId } from '@libp2p/interface-peer-id'
import { unmarshalPublicKey } from '@libp2p/crypto/keys'

import { nAtATime, u8aCompare } from '@hoprnet/hopr-utils'
import { IStream, Server, connect_relay_set_panic_hook } from '../../lib/connect_relay.js'
connect_relay_set_panic_hook()

import debug from 'debug'

const DEBUG_PREFIX = 'hopr-connect:relay:state'

const verbose = debug(DEBUG_PREFIX.concat(':verbose'))
const error = debug(DEBUG_PREFIX.concat(':error'))

type RelayConnections = {
  [id: string]: Server
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

    context[source.toString()].update(toSource as IStream)

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
  async forEach(action: (dst: string, ctx: Server) => Promise<void>) {
    await nAtATime(
      action,
      this,
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
    const toSourceContext = new Server(
      toSource as IStream,
      { onClose: this.cleanListener(source, destination), onUpgrade: this.cleanListener(source, destination) },
      this.options
    )
    const toDestinationContext = new Server(
      toDestination as IStream,
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

    // human-readable ID
    switch (cmpResult) {
      case 1:
        return `${a.toString()}-${b.toString()}`
      case -1:
        return `${b.toString()}-${a.toString()}`
      default:
        throw Error(`Invalid compare result. Loopbacks are not allowed.`)
    }
  }
}

export { RelayState }
