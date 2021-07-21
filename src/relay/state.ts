/// <reference path="../@types/libp2p.ts" />

import { Stream } from 'libp2p'
import type PeerId from 'peer-id'

import { u8aCompare } from '@hoprnet/hopr-utils'
import { RelayContext } from './context'

import debug from 'debug'

const DEBUG_PREFIX = 'hopr-connect:relay:state'

const error = debug(DEBUG_PREFIX.concat(':error'))

type State = {
  [id: string]: RelayContext
}

/**
 * Encapsulates open relayed connections
 */
class RelayState {
  private relayedConnections: Map<string, State>

  constructor() {
    this.relayedConnections = new Map()
  }

  relayedConnectionCount() {
    return this.relayedConnections.size
  }

  /**
   * Checks if there is a relayed connection. Liveness is not checked,
   * so connection might be dead.
   * @param source initiator of the relayed connection
   * @param destination other party of the relayed connection
   * @returns if there is already a relayed connection
   */
  exists(source: PeerId, destination: PeerId) {
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
      return false
    }

    const context = this.relayedConnections.get(id) as State

    let latency: number
    try {
      latency = await context[destination.toB58String()].ping(timeout)
    } catch (err) {
      error(err)
      return false
    }

    if (latency >= 0) {
      return true
    }

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

    const context = this.relayedConnections.get(id) as State

    context[source.toB58String()].update(toSource)
  }

  /**
   * Creates and stores a new relayed connection
   * @param source initiator of the relayed connection
   * @param destination other party of the relayed connection
   * @param toSource duplex stream to source
   * @param toDestination duplex stream to destination
   */
  createNew(source: PeerId, destination: PeerId, toSource: Stream, toDestination: Stream) {
    const toSourceContext = new RelayContext(toSource)
    const toDestinationContext = new RelayContext(toDestination)

    toSourceContext.sink(toDestinationContext.source)
    toDestinationContext.sink(toSourceContext.source)

    let relayedConnection: State = {
      [source.toB58String()]: toSourceContext,
      [destination.toB58String()]: toDestinationContext
    }

    toSourceContext.once('close', this.cleanListener(source, destination))
    toSourceContext.once('close', this.cleanListener(destination, source))

    this.relayedConnections.set(RelayState.getId(source, destination), relayedConnection)
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
        delete found[source.toB58String()]

        if (!found.hasOwnProperty(destination.toB58String())) {
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
    const cmpResult = u8aCompare(a.pubKey.marshal(), b.pubKey.marshal())

    switch (cmpResult) {
      case 1:
        return `${a.toB58String()}${b.toB58String()}`
      case -1:
        return `${b.toB58String()}${a.toB58String()}`
      default:
        throw Error(`Invalid compare result. Loopbacks are not allowed.`)
    }
  }
}

export { RelayState }
