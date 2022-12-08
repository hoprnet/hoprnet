import type { Multiaddr } from '@multiformats/multiaddr'
import type { PeerId } from '@libp2p/interface-peer-id'

type Suffix = 'PublicNode'
type AddEventName = `add${Suffix}`
type RemoveEventName = `remove${Suffix}`

export type PeerStoreType = { id: PeerId; multiaddrs: Multiaddr[] }
type NewNodeListener = (peer: PeerStoreType) => void

export interface PublicNodesEmitter {
  addListener(event: AddEventName | RemoveEventName, listener: () => void): this
  addListener(event: AddEventName, listener: NewNodeListener): this
  addListener(event: RemoveEventName, listener: (removeNode: PeerId) => void): this

  emit(event: AddEventName | RemoveEventName): boolean
  emit(event: AddEventName, newNode: PeerStoreType): boolean
  emit(event: RemoveEventName, removeNode: PeerId): boolean

  on(event: AddEventName | RemoveEventName, listener: () => void): this
  on(event: AddEventName, listener: NewNodeListener): this
  on(event: RemoveEventName, listener: (removeNode: PeerId) => void): this

  once(event: AddEventName | RemoveEventName, listener: () => void): this
  once(event: AddEventName, listener: NewNodeListener): this
  once(event: RemoveEventName, listener: (removeNode: PeerId) => void): this

  prependListener(event: AddEventName | RemoveEventName, listener: () => void): this
  prependListener(event: AddEventName, listener: NewNodeListener): this
  prependListener(event: RemoveEventName, listener: (removeNode: PeerId) => void): this

  prependOnceListener(event: AddEventName | RemoveEventName, listener: () => void): this
  prependOnceListener(event: AddEventName, listener: NewNodeListener): this
  prependOnceListener(event: RemoveEventName, listener: (removeNode: PeerId) => void): this

  removeListener(event: AddEventName | RemoveEventName, listener: () => void): this
  removeListener(event: AddEventName, listener: NewNodeListener): this
  removeListener(event: RemoveEventName, listener: (removeNode: PeerId) => void): this

  off(event: AddEventName | RemoveEventName, listener: () => void): this
  off(event: AddEventName, listener: NewNodeListener): this
  off(event: RemoveEventName, listener: (removeNode: PeerId) => void): this
}

export type StreamType = Uint8Array

export type StreamSourceAsync<T = StreamType> = AsyncIterable<T>
export type StreamSource<T = StreamType> = AsyncIterable<T> | Iterable<T>
export type StreamSink<T = StreamType> = (source: StreamSource<T>) => Promise<void>

export type Stream<T = StreamType> = {
  sink: StreamSink<T>
  source: StreamSource<T>
}

export type StreamResult = IteratorResult<StreamType, any>

export type Environment = {
  id: string
  versionRange: string
}

export enum PeerConnectionType {
  DIRECT = 'DIRECT',
  // @TODO to be implemented in https://github.com/hoprnet/hoprnet/pull/4171
  DIRECT_TO_ENTRY = 'DIRECT_TO_ENTRY',
  RELAYED = 'RELAYED',
  WEBRTC_RELAYED = 'WEBRTC_RELAYED',
  WEBRTC_DIRECT = 'WEBRTC_DIRECT'
}

export type HoprConnectOptions = {
  publicNodes?: PublicNodesEmitter
  allowLocalConnections?: boolean
  allowPrivateConnections?: boolean
  initialNodes?: PeerStoreType[]
  interface?: string
  maxRelayedConnections?: number
  environment?: string
  supportedEnvironments?: Environment[]
  relayFreeTimeout?: number
  dhtRenewalTimeout?: number
  entryNodeReconnectBaseTimeout?: number
  entryNodeReconnectBackoff?: number
  // To be removed once NR got removed
  isAllowedToAccessNetwork?: (id: PeerId) => Promise<boolean>
}

export type HoprConnectTestingOptions = {
  // @TODO implement this
  __useLocalAddresses?: boolean
  // Simulated NAT: only connect directly to relays
  __noDirectConnections?: boolean
  // Simulated NAT: ignore WebRTC upgrade
  __noWebRTCUpgrade?: boolean
  // Local mode: only use local address, i.e. don't try to
  // determine any external / public IP addresses
  __preferLocalAddresses?: boolean
  // Disable UPNP support
  __noUPNP?: boolean
}
