/// <reference path="../@types/libp2p.ts" />
import { Connection, Stream } from 'libp2p'
import type PeerId from 'peer-id'
import type Multiaddr from 'multiaddr'
import type { EventEmitter } from 'events'
import type { Server } from 'net'

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

export interface Upgrader {
  upgradeOutbound(multiaddrConnection: MultiaddrConnection): Promise<Connection>
  upgradeInbound(multiaddrConnection: MultiaddrConnection): Promise<Connection>
}

export interface PeerRouting {
  findPeer(peerId: PeerId): Promise<PeerInfo>
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
