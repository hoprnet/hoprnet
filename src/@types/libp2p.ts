/// <reference path="./bl.ts" />

declare module 'libp2p' {
  type PeerId = import('peer-id')
  type Multiaddr = import('multiaddr')
  type EventEmitter = import('events').EventEmitter
  type AbortSignal = import('abort-controller').AbortSignal
  type Connection = import('libp2p-interfaces').Connection
  type BL = import('bl').BLInterface

  export type PeerRoute = {
    id: PeerId
    multiaddrs: Multiaddr[]
  }

  // https://github.com/libp2p/js-libp2p/blob/master/doc/API.md#peerstoreaddressbookadd
  type AddressBook = {
    add(id: PeerId, addr: Array<Multiaddr>): AddressBook
    delete(id: PeerId): boolean
    get(id: PeerId): Array<{ multiaddr: Multiaddr }>
    getMultiaddrsForPeer(id: PeerId): Array<string>
    set(peerId: PeerId, multiaddrs: Array<Multiaddr>): AddressBook
  }

  export type PeerStore = {
    //https://github.com/libp2p/js-libp2p/blob/master/doc/API.md#peerstoreget
    get(
      peerId: PeerId
    ):
      | { id: PeerId; addresses: Array<Multiaddr>; metadata: Map<string, Uint8Array>; protocols: Array<string> }
      | undefined
    peers: Map<
      string,
      { id: PeerId; addresses: Array<Multiaddr>; metadata: Map<string, Uint8Array>; protocols: Array<string> }
    >
    delete(peer: PeerId): void

    addressBook: AddressBook

    protoBook: {} // TODO
  }
  export interface DialOptions {
    signal?: AbortSignal
  }

  export type StreamType = BL | Uint8Array

  export type StreamResult = IteratorResult<StreamType, void>

  export type Stream = {
    sink: (source: Stream['source']) => Promise<void>
    source: AsyncGenerator<StreamType, void>
  }

  export type Handler = {
    stream: Stream
    connection?: Connection
    protocol?: string
  }

  export interface MultiaddrConnection extends Stream {
    close(err?: Error): Promise<void>
    conn: any
    remoteAddr: Multiaddr
    localAddr?: Multiaddr
    timeline: {
      open: number
      close?: number
    }
  }

  //https://github.com/libp2p/js-libp2p-interfaces/tree/master/src/peer-routing
  export interface PeerRouting {
    findPeer(peerId: PeerId): Promise<{ id: PeerId; multiaddrs: Multiaddr[] }>
  }

  export interface Upgrader {
    upgradeOutbound(multiaddrConnection: MultiaddrConnection): Promise<Connection>
    upgradeInbound(multiaddrConnection: MultiaddrConnection): Promise<Connection>
    protocols: Map<string, Handler>
  }

  export interface Registrar {
    getConnection(peer: PeerId): Connection | undefined
    handle(protocol: string, handler: Handler): void
  }

  interface TimeoutController {
    clear(): void
  }

  interface DialRequest {
    addrs: Multiaddr[]
    dialer: Dialer
    // @TODO
    dialAction: any
  }
  export interface Dialer {
    connectToPeer(peer: PeerId | Multiaddr | string, options?: any): Promise<Connection>
    _pendingDials: {
      [index: string]: {
        dialRequest: DialRequest
        controller: TimeoutController
        promise: Promise<Connection>
        destroy(): void
      }
    }
  }

  export interface ConnectionManager extends EventEmitter {
    connections: Map<string, [Connection]>
  }

  export interface Registrar {
    getConnection(peerId: PeerId): Connection | null
  }

  export type ConnHandler = (conn: Connection) => void

  export interface Listener extends EventEmitter {
    close(): void
    listen(ma: Multiaddr): Promise<void>
    getAddrs(): Multiaddr[]
  }

  export default class LibP2P {
    constructor(options: any) //: LibP2P
    static create(options: any): Promise<LibP2P>
    // @TODO add libp2p types
    addressManager: {
      getListenAddrs(): Multiaddr[]
    }
    transportManager: {
      getAddrs(): Multiaddr[]
    }
    isStarted(): boolean
    emit: (event: string, ...args: any[]) => void
    dial: (addr: Multiaddr | PeerId, options?: { signal: AbortSignal }) => Promise<Handler>
    dialer: Dialer
    dialProtocol: (addr: Multiaddr | PeerId, protocol: string, options?: { signal: AbortSignal }) => Promise<Handler>
    hangUp: (addr: PeerId | Multiaddr | string) => Promise<void>
    peerStore: PeerStore
    peerRouting: PeerRouting
    handle: (protocol: string | string[], handler: (struct: Handler) => void) => void
    start(): Promise<void>
    stop(): Promise<void>

    multiaddrs: Multiaddr[]
    connectionManager: ConnectionManager // Undoumented
    registrar: Registrar // Undocumented
    _dht: { peerRouting: PeerRouting } | undefined // Undocumented

    peerId: PeerId // ATTN: Not documented API
  }
}
