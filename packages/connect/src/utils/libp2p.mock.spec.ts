import type { Connection } from '@libp2p/interface-connection'
import type { Dialer, ConnectionManagerEvents } from '@libp2p/interface-connection-manager'
import type { Components, Initializable } from '@libp2p/interfaces/components'
import type { AbortOptions } from '@libp2p/interfaces'
import { type PeerId, isPeerId } from '@libp2p/interface-peer-id'
import { CustomEvent, EventEmitter as TypedEventEmitter } from '@libp2p/interfaces/events'
import { peerIdFromString } from '@libp2p/peer-id'
import type { PeerStore } from '@libp2p/interfaces/peer-store'
import { duplexPair } from 'it-pair/duplex'

import { EventEmitter } from 'events'
import { Multiaddr } from '@multiformats/multiaddr'

import type { Stream } from '../types.js'
import { CODE_P2P } from '../constants.js'
import { StreamHandler } from '@libp2p/interfaces/registrar'
import { MultiaddrConnection } from '@libp2p/interfaces/transport'

/**
 * Minimal TransportManager, used for unit testing
 */
class MyTransportManager implements Initializable {
  private components: Components | undefined

  private addrs: Set<string>

  constructor(private network: ReturnType<typeof createFakeNetwork>) {
    this.addrs = new Set<string>()
  }

  public init(components: Components) {
    this.components = components
  }

  public getComponents() {
    if (this.components == undefined) {
      throw Error(`Components not set`)
    }
    return this.components
  }

  public async listen(addrs: Multiaddr[]) {
    for (const addr of addrs) {
      addr.decapsulateCode(CODE_P2P)
      if (!this.addrs.has(addr.toString())) {
        this.addrs.add(addr.toString())
      }
      this.network.listen(addr, this.getComponents())
    }
  }

  public getAddrs(): Multiaddr[] {
    return [...this.addrs].map((str) => new Multiaddr(str))
  }
}

/**
 * Minimal Registrar, used for unit testing
 */
class MyRegistrar {
  handlers: Map<string, StreamHandler>

  constructor() {
    this.handlers = new Map<string, StreamHandler>()
  }

  public async handle(protocols: string | string[], handler: StreamHandler): Promise<void> {
    for (const protocol of Array.isArray(protocols) ? protocols : [protocols]) {
      this.handlers.set(protocol, handler)
    }
  }

  public getHandler(protocol: string): StreamHandler {
    return this.handlers.get(protocol) as StreamHandler
  }
}

/**
 * Minimal ConnectionManager used for unit testing
 */
class MyConnectionManager extends TypedEventEmitter<ConnectionManagerEvents> implements Initializable {
  dialer: Dialer

  connections: Map<string, Connection[]>

  components: Components | undefined

  constructor(
    peerStore: PeerStore,
    network: ReturnType<typeof createFakeNetwork>,
    connManagerOpts: {
      outerDial?: ReturnType<typeof createFakeNetwork>['connect']
      getStream?: (protocol?: string | string[]) => Stream
    } = {}
  ) {
    super()

    this.connections = new Map<string, Connection[]>()

    this.dialer = {
      dial: async (peer: PeerId | Multiaddr, _opts: Parameters<Dialer['dial']>[1]) => {
        const dialMethod = connManagerOpts.outerDial ?? network.connect

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
              conn = dialMethod(this.getComponents().getPeerId(), fullAddr) as any
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

          conn = dialMethod(this.getComponents().getPeerId(), fullAddr)
        }

        if (conn != undefined) {
          this.connections.set(peer.toString(), (this.connections.get(peer.toString()) ?? []).concat([conn]))

          network.events.once(disconnectEvent(fullAddr as Multiaddr), (emitEvent: boolean = true) =>
            this.onClose(peerId, emitEvent)
          )

          return conn as any
        }

        throw Error(`not implemented within unit test`)
      }
    } as Dialer
  }

  public init(components: Components) {
    this.components = components
  }

  public getConnections(peer: PeerId | undefined, _options?: AbortOptions): Connection[] {
    if (peer == undefined) {
      return [...this.connections.values()].flat(1)
    } else {
      return this.connections.get(peer.toString()) ?? []
    }
  }

  public getComponents() {
    if (this.components == undefined) {
      throw Error(`Components not set`)
    }
    return this.components
  }

  private onClose(peer: PeerId, emitEvent: boolean = true) {
    const existingConnections = this.connections.get(peer.toString())
    if (existingConnections != undefined && existingConnections.length > 0) {
      const toClose = existingConnections.shift()

      this.connections.set(peer.toString(), existingConnections)

      if (emitEvent && existingConnections.length == 0) {
        this.dispatchEvent(
          new CustomEvent<Connection>('peer:disconnect', {
            detail: toClose
          })
        )
      }
    }
  }
}

/**
 * Implements a minimal, non-persistent PeerStore
 * @returns mocked PeerStore
 */
function createFakePeerStore(): PeerStore {
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

/**
 * Implements a minimal Upgrader, bypassing any protocol uprades
 * @param onInboundStream reply to inbound stream
 * @returns
 */
function createFakeUpgrader(onInboundStream?: (stream: Stream) => Promise<void>): NonNullable<Components['upgrader']> {
  return {
    async upgradeInbound(maConn: MultiaddrConnection) {
      maConn.timeline.upgraded = Date.now()

      // @TODO enhance this
      onInboundStream?.(maConn)

      return maConn
    },
    async upgradeOutbound(maConn: MultiaddrConnection) {
      maConn.timeline.upgraded = Date.now()

      return maConn
    }
  }
}

export function connectEvent(addr: Multiaddr): string {
  return `connect:${addr.decapsulateCode(CODE_P2P).toString()}`
}

export function disconnectEvent(addr: Multiaddr) {
  return `disconnect:${addr.decapsulateCode(CODE_P2P).toString()}`
}

/**
 * Creates a connection that mimics libp2p's protocol selection
 * @param self initiator of the connection
 * @param remoteComponents libp2p instance of remote peer
 * @param throwError if true, throw an error instead of returning a stream
 * @returns
 */
function createConnection(self: PeerId, remoteComponents: Components, throwError: boolean = false): Connection {
  const conn = {
    remotePeer: remoteComponents.getPeerId(),
    remoteAddr: new Multiaddr('/ip4/1.2.3.4/tcp/567'),
    _closed: false,
    close: async () => {
      // @ts-ignore
      conn._closed = true
    },
    stat: {
      direction: 'outbound',
      timeline: {
        open: Date.now()
      }
    },
    newStream: async (protocols: string[]) => {
      if (throwError) {
        throw Error(`boom - protocol error`)
      }

      for (const protocol of protocols) {
        const streamHandler = remoteComponents.getRegistrar().getHandler(protocol)
        if (streamHandler != undefined) {
          const duplex = duplexPair<Uint8Array>()

          streamHandler({
            stream: {
              sink: duplex[1].sink,
              source: duplex[1].source
            } as any,
            protocol,
            connection: {
              remotePeer: self,
              stat: {
                direction: 'inbound',
                timeline: {
                  open: Date.now()
                }
              }
            } as any
          })
          return {
            protocol,
            stream: {
              sink: duplex[0].sink,
              source: duplex[0].source
            }
          }
        }
      }

      throw Error(`None of the given protocols '${protocols.join(', ')}' are spoken`)
    }
  } as unknown as Connection

  return conn as Connection
}

/**
 * Creates a network that behaves similarly to a socket-based network
 * @returns Event-based network implementation
 */
export function createFakeNetwork() {
  const network = new EventEmitter()

  // Multiaddr (including PeerId) -> Components
  let components: Map<string, Components> = new Map<string, Components>()

  const listen = (addr: Multiaddr, nodeComponents: Components) => {
    components.set(
      addr.decapsulateCode(CODE_P2P).encapsulate(`/p2p/${nodeComponents.getPeerId().toString()}`).toString(),
      nodeComponents
    )
  }

  const connect = (self: PeerId, ma: Multiaddr, throwError: boolean = false) => {
    const remoteComponents = components.get(ma.toString())

    if (remoteComponents != undefined) {
      return createConnection(self, remoteComponents, throwError)
    }

    throw Error(`Cannot connect. Maybe not listening?`)
  }

  const close = (ma: Multiaddr, emitEvent: boolean = true) => {
    components.delete(ma.toString())
    network.emit(disconnectEvent(ma), emitEvent)
  }

  return {
    events: network,
    listen,
    connect,
    close,
    stop: network.removeAllListeners.bind(network)
  }
}

/**
 * Returns a minimal implementation of libp2p components
 * @param peerId the components identity
 * @param opts customizable dial behavior
 * @returns
 */
export async function createFakeComponents(
  peerId: PeerId,
  network: ReturnType<typeof createFakeNetwork>,
  opts: {
    outerDial?: ReturnType<typeof createFakeNetwork>['connect']
    protocols?: Iterable<[string | string[], StreamHandler]>
    listeningAddrs?: Multiaddr[]
    onIncomingStream?: (stream: Stream) => Promise<void>
  } = {}
) {
  const peerStore = createFakePeerStore()

  const registrar = new MyRegistrar() as NonNullable<Components['registrar']>

  const transportManager = new MyTransportManager(network) as NonNullable<Components['transportManager']>

  const connectionManager = new MyConnectionManager(peerStore, network, opts) as NonNullable<
    Components['connectionManager']
  >

  const upgrader = createFakeUpgrader(opts.onIncomingStream)

  const components = {
    getConnectionManager: () => connectionManager,
    getPeerId: () => peerId,
    getPeerStore: () => peerStore,
    getRegistrar: () => registrar,
    getTransportManager: () => transportManager,
    getUpgrader: () => upgrader
  } as Components

  connectionManager.init(components)
  transportManager.init(components)

  for (const protcolHandler of opts.protocols ?? []) {
    await components.getRegistrar().handle(...protcolHandler)
  }

  if (opts.listeningAddrs) {
    await components.getTransportManager().listen(opts.listeningAddrs)
  }

  return components
}
