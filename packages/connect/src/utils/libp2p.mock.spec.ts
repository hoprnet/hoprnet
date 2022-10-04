import type { Connection } from '@libp2p/interface-connection'
import type { Dialer, ConnectionManagerEvents } from '@libp2p/interface-connection-manager'
import type { Components } from '@libp2p/interfaces/components'
import type { AbortOptions } from '@libp2p/interfaces'
import { type PeerId, isPeerId } from '@libp2p/interface-peer-id'

import { CODE_P2P } from '../constants.js'

import { EventEmitter } from 'events'

import { Multiaddr } from '@multiformats/multiaddr'

import { peerIdFromString } from '@libp2p/peer-id'
import type { PeerStore } from '@libp2p/interfaces/peer-store'
import { CustomEvent, EventEmitter as TypedEventEmitter } from '@libp2p/interfaces/events'
import type { Stream } from '../types.js'

class MyConnectionManager extends TypedEventEmitter<ConnectionManagerEvents> {
  dialer: Dialer

  connections: Map<string, Connection[]>

  components: Components | undefined

  constructor(
    peerStore: PeerStore,
    connManagerOpts: {
      outerDial?: (ma: Multiaddr, throwError?: boolean) => Connection
      network?: EventEmitter
      getStream?: (protocol?: string | string[]) => Stream
    } = {}
  ) {
    super()

    this.connections = new Map<string, Connection[]>()

    this.dialer = {
      dial: async (peer: PeerId | Multiaddr, _opts: Parameters<Dialer['dial']>[1]) => {
        if (connManagerOpts.outerDial == undefined) {
          throw Error(`Network not connected`)
        }

        let conn: Connection | undefined
        let fullAddr: Multiaddr | undefined
        let peerId: PeerId

        if (isPeerId(peer)) {
          // Connect using PeerId and known addresses
          peerId = peer
          const addrs = await peerStore.addressBook.get(peer)

          if (addrs == null || addrs.length == 0) {
            throw Error(`No addresses known`)
          }

          for (const addr of addrs) {
            fullAddr = addr.multiaddr.decapsulateCode(CODE_P2P).encapsulate(`/p2p/${peer.toString()}`)
            try {
              conn = connManagerOpts.outerDial(fullAddr) as any
            } catch (err) {
              // try next address
              continue
            }

            if (conn != undefined) {
              break
            }
          }

          if (conn == undefined) {
            throw Error(`Dial error: no valid addresses known`)
          }
        } else {
          // Connect using given Multiaddr that contains a PeerId
          peerId = peerIdFromString(peer.getPeerId() as string)
          fullAddr = peer.decapsulateCode(CODE_P2P).encapsulate(`/p2p/${peer.toString()}`)

          conn = connManagerOpts.outerDial(fullAddr)
        }

        if (conn != undefined) {
          this.connections.set(peer.toString(), (this.connections.get(peer.toString()) ?? []).concat([conn]))

          connManagerOpts.network?.once(disconnectEvent(fullAddr as Multiaddr), () => this.onClose(peerId))

          return conn as any
        }

        throw Error(`not implemented within unit test`)
      }
    } as Dialer
  }

  public init(components: Components) {
    this.components = components
  }

  public getConnections(_peer: PeerId | undefined, _options?: AbortOptions): Connection[] {
    return []
  }

  private onClose(peer: PeerId) {
    const existingConnections = this.connections.get(peer.toString())
    if (existingConnections != undefined && existingConnections.length > 0) {
      const toClose = existingConnections.shift()

      this.connections.set(peer.toString(), existingConnections)

      if (existingConnections.length == 0) {
        this.dispatchEvent(
          new CustomEvent<Connection>('peer:disconnect', {
            detail: toClose
          })
        )
      }
    }
  }
}

function fakePeerStore(): PeerStore {
  const addrs = new Map<string, Multiaddr[]>()

  return {
    addressBook: {
      async get(id: PeerId) {
        return (addrs.get(id.toString()) ?? []).map((ma) => ({ multiaddr: ma, isCertified: true }))
      },
      async add(id: PeerId, multiaddrs: Multiaddr[]) {
        addrs.set(
          id.toString(),
          [
            ...new Set(
              (addrs.get(id.toString()) ?? []).concat(multiaddrs).map((ma) => ma.decapsulateCode(CODE_P2P).toString())
            )
          ].map((str) => new Multiaddr(str))
        )
      }
    }
  } as PeerStore
}

export function connectEvent(addr: Multiaddr): string {
  return `connect:${addr.decapsulateCode(CODE_P2P).toString()}`
}

export function disconnectEvent(addr: Multiaddr) {
  return `disconnect:${addr.decapsulateCode(CODE_P2P).toString()}`
}

function createConnection(
  remotePeer: PeerId,
  spokenProtocols: Map<string, () => Stream>,
  throwError: boolean = false
): Connection {
  const conn = {
    remotePeer,
    _closed: false,
    close: async () => {
      // @ts-ignore
      conn._closed = true
    },
    stat: {
      timeline: {
        open: Date.now()
      }
    },
    newStream: async (protocols: string[]) => {
      if (throwError) {
        throw Error(`boom - protocol error`)
      }

      for (const protocol of protocols) {
        const found = spokenProtocols.get(protocol)
        if (found != undefined) {
          return {
            protocol,
            stream: found()
          }
        }
      }

      throw Error(`None of the given protocols '${protocols.join(', ')}' are spoken`)
    }
  } as unknown as Connection

  return conn as Connection
}

export function createFakeNetwork() {
  const network = new EventEmitter()

  const protolHandlers = new Map<string, Map<string, () => Stream>>()

  const listen = (addr: Multiaddr, protocols: Iterable<[string | string[], () => Stream]>) => {
    const emitter = new EventEmitter()
    network.on(connectEvent(addr), () => emitter.emit('connected'))

    const peerId = addr.getPeerId() as string

    const protocolMap = new Map<string, () => Stream>()
    for (const [protocol, handler] of protocols) {
      for (const individualProtocol of Array.isArray(protocol) ? protocol : [protocol]) {
        protocolMap.set(individualProtocol, handler)
      }
    }

    protolHandlers.set(peerId, protocolMap)

    return emitter
  }

  const connect = (ma: Multiaddr, throwError: boolean = false) => {
    let remotePeer: PeerId

    if (isPeerId(ma)) {
      remotePeer = ma
    } else {
      remotePeer = peerIdFromString((ma as Multiaddr).getPeerId() as string)
    }

    network.emit(connectEvent(ma))
    const spokenProtocols = protolHandlers.get(remotePeer.toString())

    if (spokenProtocols != undefined) {
      return createConnection(remotePeer, spokenProtocols, throwError)
    }

    throw Error(`Cannot connect. Maybe not listening?`)
  }

  const close = (ma: Multiaddr) => {
    const peerId = ma.getPeerId() as string

    protolHandlers.delete(peerId)
    network.emit(disconnectEvent(ma), ma)
  }

  return {
    events: network,
    listen,
    connect,
    close,
    stop: network.removeAllListeners.bind(network)
  }
}

export function createFakeComponents(
  peerId: PeerId,
  opts: {
    outerDial?: (ma: Multiaddr, throwError?: boolean) => Connection
    network?: EventEmitter
    defaultStream?: Stream
  } = {}
) {
  const peerStore = fakePeerStore()

  const connectionManager = new MyConnectionManager(peerStore, opts) as NonNullable<Components['connectionManager']>

  const getUpgrader = () =>
    ({
      upgradeInbound: (x: any) => x,
      upgradeOutbound: (x: any) => x
    } as Components['upgrader'])

  return {
    getPeerId: () => peerId,
    getConnectionManager: () => connectionManager,
    getUpgrader,
    getPeerStore: () => peerStore
  } as Components
}
