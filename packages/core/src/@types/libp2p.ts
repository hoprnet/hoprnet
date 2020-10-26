
declare module 'libp2p' {
  type PeerId = import('peer-id')
  type Multiaddr = import('multiaddr')
  type EventEmitter = import('events').EventEmitter

  export type Stream = {
    sink: (source: AsyncIterable<Uint8Array>) => Promise<void>
    source: AsyncIterable<Uint8Array>
  }

  export interface Connection {
    localAddr: Multiaddr
    remoteAddr: Multiaddr
    localPeer: PeerId
    remotePeer: PeerId
    newStream(
      protocols?: string[]
    ): Promise<{
      protocol: string
      stream: Stream
    }>
    close(): Promise<void>
    getStreams(): any[]
    stat: {
      direction: 'outbound' | 'inbound'
      timeline: {
        open: number
        upgraded: number
      }
      multiplexer?: any
      encryption?: any
    }
  }

  export type PeerRoute = { 
    id: PeerId;
    multiaddrs: Multiaddr[];
  }

  export type PeerStore = {
    //https://github.com/libp2p/js-libp2p/blob/master/doc/API.md#peerstoreget 
    get(peerId: PeerId):  { id: PeerId, addresses: Array<Multiaddr>, metadata: Map<string, Uint8Array>, protocols: Array<string> } | undefined
    peers: Map<string, { id: PeerId, addresses: Array<Multiaddr>, metadata: Map<string, Uint8Array>, protocols: Array<string> }>
    delete(peer: PeerId): void

    // https://github.com/libp2p/js-libp2p/blob/master/doc/API.md#peerstoreaddressbookadd
    addressBook: {
      add(id: PeerId, addr: Array<Multiaddr>)
      delete(id: PeerId)
      get(id: PeerId): Array<{multiaddr: Multiaddr}>
      getMultiaddrsForPeer(id: PeerId): Array<string>
      set(peerId: PeerId, multiaddrs: Array<Multiaddr>)
    }

    protoBook: {} // TODO
  }
  export interface DialOptions {
    signal?: AbortSignal
    relay?: PeerId 
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
    findPeer(peerId: PeerId): Promise<{ id: PeerId, multiaddrs: Multiaddr[] }>
  }

  export interface Upgrader {
    upgradeOutbound(multiaddrConnection: MultiaddrConnection): Promise<Connection>
    upgradeInbound(multiaddrConnection: MultiaddrConnection): Promise<Connection>
  }

  export interface Registrar {
    getConnection(peer: PeerId): Connection | undefined
    handle(protocol: string, handler: Handler): void
  }

  export interface Dialer {
    connectToPeer(peer: PeerId | Multiaddr | string, options?: any): Promise<Connection>
  }

  export type ConnHandler = (conn: Connection) => void


  export interface Listener extends EventEmitter {
    close(): void
    listen(ma: Multiaddr): Promise<void>
    getAddrs(): Multiaddr[]
  }

  export default class LibP2P {
    constructor(options: any) //: LibP2P
    static create(options: any): any
    // @TODO add libp2p types
    emit: (event: string, ...args: any[]) => void
    dial: (addr: Multiaddr | PeerId, options?: { signal: AbortSignal }) => Promise<Handler>
    dialer: any // TODO
    dialProtocol: (
      addr: Multiaddr | PeerId,
      protocol: string,
      options?: { signal: AbortSignal }
    ) => Promise<Handler>
    hangUp: (addr: PeerId | Multiaddr | string) => Promise<void>
    peerStore: PeerStore
    peerRouting: {
      findPeer: (addr: PeerId) => Promise<PeerRoute>
    }
    handle: (protocol: string | string[], handler: (struct: { connection: any; stream: any }) => void) => void
    start(): Promise<any>
    stop(): Promise<void>

    multiaddrs: Multiaddr[]
    connectionManager: EventEmitter

    peerId: PeerId // ATTN: Not documented API
  }
}
