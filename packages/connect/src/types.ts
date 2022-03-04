import type { Multiaddr } from 'multiaddr'
import type PeerId from 'peer-id'
import type BufferList from 'bl/BufferList'

type Suffix = 'PublicNode'
type AddEventName = `add${Suffix}`
type RemoveEventName = `remove${Suffix}`

export type PeerStoreType = { id: PeerId; multiaddrs: Multiaddr[] }
type NewNodeListener = (peer: PeerStoreType) => void

export interface PublicNodesEmitter {
  addListener(event: AddEventName | RemoveEventName, listener: () => void): this
  addListener(event: AddEventName, listener: NewNodeListener): this
  addListener(event: RemoveEventName, listener: (removeNode: PeerId) => void): this
  addListener(event: string | symbol, listener: (...args: any[]) => void): this

  emit(event: AddEventName | RemoveEventName): boolean
  emit(event: AddEventName, newNode: PeerStoreType): boolean
  emit(event: RemoveEventName, removeNode: PeerId): boolean
  emit(event: string | symbol, ...args: any[]): boolean

  on(event: AddEventName | RemoveEventName, listener: () => void): this
  on(event: AddEventName, listener: NewNodeListener): this
  on(event: RemoveEventName, listener: (removeNode: PeerId) => void): this
  on(event: string | symbol, listener: (...args: any[]) => void): this

  once(event: AddEventName | RemoveEventName, listener: () => void): this
  once(event: AddEventName, listener: NewNodeListener): this
  once(event: RemoveEventName, listener: (removeNode: PeerId) => void): this
  once(event: string | symbol, listener: (...args: any[]) => void): this

  prependListener(event: AddEventName | RemoveEventName, listener: () => void): this
  prependListener(event: AddEventName, listener: NewNodeListener): this
  prependListener(event: RemoveEventName, listener: (removeNode: PeerId) => void): this
  prependListener(event: string | symbol, listener: (...args: any[]) => void): this

  prependOnceListener(event: AddEventName | RemoveEventName, listener: () => void): this
  prependOnceListener(event: AddEventName, listener: NewNodeListener): this
  prependOnceListener(event: RemoveEventName, listener: (removeNode: PeerId) => void): this
  prependOnceListener(event: string | symbol, listener: (...args: any[]) => void): this

  removeListener(event: AddEventName | RemoveEventName, listener: () => void): this
  removeListener(event: AddEventName, listener: NewNodeListener): this
  removeListener(event: RemoveEventName, listener: (removeNode: PeerId) => void): this
  removeListener(event: string | symbol, listener: (...args: any[]) => void): this
}

export type StreamType = BufferList | Uint8Array

export type StreamSource<T = StreamType> = AsyncIterable<T>

export type Stream<T = StreamType> = {
  sink: (source: StreamSource<T>) => Promise<void>
  source: StreamSource<T>
}

export type StreamResult = IteratorResult<StreamType, any>

export type HoprConnectOptions = {
  publicNodes?: PublicNodesEmitter
  allowLocalConnections?: boolean
  allowPrivateConnections?: boolean
  initialNodes?: PeerStoreType[]
  interface?: string
  maxRelayedConnections?: number
  environment?: string
  relayFreeTimeout?: number
  dhtRenewalTimeout?: number
}

export type HoprConnectTestingOptions = {
  // Simulated NAT: only connect directly to relays
  __noDirectConnections?: boolean
  // Simulated NAT: ignore WebRTC upgrade
  __noWebRTCUpgrade?: boolean
  // Local mode: only use local address, i.e. don't try to
  // determine any external / public IP addresses
  __preferLocalAddresses?: boolean
  // Local mode: running a local testnet on the same machine
  // hence the local interface is treated as an exposed host
  __runningLocally?: boolean
  // Disable UPNP support
  __noUPNP?: boolean
}

export type HoprConnectListeningOptions = undefined

export type HoprConnectDialOptions = {
  signal?: AbortSignal
}
