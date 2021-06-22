/// <reference path="../@types/libp2p.ts" />

import { Stream } from 'libp2p'
import type PeerId from 'peer-id'

import { u8aCompare } from '@hoprnet/hopr-utils'
import { RelayContext } from './context'

import debug from 'debug'

const DEBUG_PREFIX = 'hopr-connect:relay:state'

// const log = debug(DEBUG_PREFIX)
const error = debug(DEBUG_PREFIX.concat(':error'))
// const verbose = debug(DEBUG_PREFIX.concat(':verbose'))

type State = {
  [id: string]: RelayContext
}
class RelayState {
  private relayedConnections: Map<string, State>

  constructor() {
    this.relayedConnections = new Map()
  }

  exists(source: PeerId, destination: PeerId) {
    const id = RelayState.getId(source, destination)

    return this.relayedConnections.has(id)
  }

  async isActive(source: PeerId, destination: PeerId, timeout?: number): Promise<boolean> {
    const id = RelayState.getId(source, destination)
    if (this.relayedConnections.has(id)) {
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

  updateExisting(source: PeerId, destination: PeerId, toSource: Stream): void {
    const id = RelayState.getId(source, destination)

    if (this.relayedConnections.has(id)) {
      throw Error(`Relayed connection does not exit`)
    }

    const context = this.relayedConnections.get(id) as State

    context[source.toB58String()].update(toSource)
  }

  createNew(source: PeerId, destination: PeerId, toSource: Stream, toDestination: Stream) {
    const toSourceContext = new RelayContext(toSource)
    const toDestinationContext = new RelayContext(toDestination)

    toSourceContext.sink(toDestinationContext.source)
    toDestinationContext.sink(toSourceContext.source)

    let relayedConnection: State = {
      [source.toB58String()]: toSourceContext,
      [destination.toB58String()]: toDestinationContext
    }

    this.relayedConnections.set(RelayState.getId(source, destination), relayedConnection)
  }
  static getId(a: PeerId, b: PeerId) {
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
