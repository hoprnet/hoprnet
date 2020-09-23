import type PeerId from 'peer-id'
import type PeerInfo from 'peer-info'
import type Multiaddr from 'multiaddr'

import type { EventEmitter } from 'events'
import type { Server } from 'net'

export interface DialOptions {
  signal?: AbortSignal
  relay?: PeerId | PeerInfo
}

export type Stream = {
  sink: (source: AsyncIterable<Uint8Array>) => Promise<void>
  source: AsyncIterable<Uint8Array>
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

export interface Upgrader {
  upgradeOutbound(multiaddrConnection: MultiaddrConnection): Promise<Connection>
  upgradeInbound(multiaddrConnection: MultiaddrConnection): Promise<Connection>
}

export interface PeerRouting {
  findPeer(peerId: PeerId): Promise<PeerInfo>
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

export interface PeerStore {
  has(peerInfo: PeerId): boolean
  put(peerInfo: PeerInfo, options?: { silent: boolean }): PeerInfo
  get(peerId: PeerId): PeerInfo
  peers: Map<string, PeerInfo>
  remove(peer: PeerId): void
}

export interface Registrar {
  getConnection(peer: PeerInfo): Connection | undefined
  handle(protocol: string, handler: Handler): void
}

export interface Dialer {
  connectToPeer(peer: PeerInfo, options?: any): Promise<Connection>
}

export type ConnHandler = (conn: Connection) => void

export interface Libp2pServer extends Server {
  __connections: MultiaddrConnection[]
}

export interface Listener extends EventEmitter {
  close(): void
  listen(ma: Multiaddr): Promise<void>
  getAddrs(): Multiaddr[]
}
