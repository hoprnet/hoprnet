declare module 'libp2p-interfaces' {
  type EventEmitter = import('events').EventEmitter
  type Multiaddr = import('multiaddr')
  type PeerId = import('peer-id')
  type Upgrader = import('libp2p').Upgrader
  type LibP2P = import('libp2p').default
  type Stream = import('libp2p').Stream

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

  interface PeerDiscoveryEvents {
    peer: (arg: { id: PeerId; multiaddrs: Multiaddr[] }) => void
  }

  interface PeerDiscovery extends EventEmitter {
    on<U extends keyof PeerDiscoveryEvents>(event: U, listener: PeerDiscoveryEvents[U]): this
    emit<U extends keyof PeerDiscoveryEvents>(event: U, ...args: Parameters<PeerDiscoveryEvents[U]>): boolean

    tag: string
    start(): Promise<void>
    stop(): Promise<void>
  }

  export var PeerDiscovery: PeerDiscovery

  interface ListenerEvents {
    listening: () => void
    close: () => void
    connection: (conn: Connection) => void
    error: (err?: any) => void
  }

  interface Listener extends EventEmitter {
    on<U extends keyof ListenerEvents>(event: U, listener: ListenerEvents[U]): this
    emit<U extends keyof ListenerEvents>(event: U, ...args: Parameters<ListenerEvents[U]>): boolean

    listen(ma: Multiaddr): Promise<void>
    getAddrs(): Multiaddr[]
    close(args?: any): Promise<void>
  }

  interface Transport {
    [Symbol.toStringTag]: string
    filter(mas: Multiaddr[]): Multiaddr[]
    dial(ma: Multiaddr, opts?: { signal?: AbortSignal }): Promise<Connection>
    createListener(opts: any, handlerFunction: (conn: Connection) => void): Listener
  }
  interface TransportConstructor {
    new (args: { upgrader: Upgrader; libp2p: LibP2P }): Transport
  }

  export var Transport: TransportConstructor
}
