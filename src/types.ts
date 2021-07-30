import type { Multiaddr } from 'multiaddr'
import type PeerId from 'peer-id'
import type BL from 'bl'
import type { PromiseValue } from '@hoprnet/hopr-utils'

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

export type StreamType = Buffer | BL | Uint8Array

export type Stream<T = StreamType> = {
  sink: (source: Stream['source']) => Promise<void>
  source: AsyncGenerator<T, void>
}

export type StreamResult = PromiseValue<ReturnType<Stream['source']['next']>>

export type DialOptions = { signal?: AbortSignal }
