type Multiaddr = import('multiaddr').Multiaddr
type PeerId = import('peer-id')

type Suffix = 'PublicNode'
type AddEventName = `add${Suffix}`
type RemoveEventName = `remove${Suffix}`

export interface PublicNodesEmitter {
  addListener(event: AddEventName | RemoveEventName, listener: () => void): this
  addListener(event: AddEventName, listener: (newNode: Multiaddr) => void): this
  addListener(event: RemoveEventName, listener: (removeNode: PeerId) => void): this
  addListener(event: string | symbol, listener: (...args: any[]) => void): this

  emit(event: AddEventName | RemoveEventName): boolean
  emit(event: AddEventName, newNode: Multiaddr): boolean
  emit(event: RemoveEventName, removeNode: PeerId): boolean
  emit(event: string | symbol, ...args: any[]): boolean

  on(event: AddEventName | RemoveEventName, listener: () => void): this
  on(event: AddEventName, listener: (newNode: Multiaddr) => void): this
  on(event: RemoveEventName, listener: (removeNode: PeerId) => void): this
  on(event: string | symbol, listener: (...args: any[]) => void): this

  once(event: AddEventName | RemoveEventName, listener: () => void): this
  once(event: AddEventName, listener: (newNode: Multiaddr) => void): this
  once(event: RemoveEventName, listener: (removeNode: PeerId) => void): this
  once(event: string | symbol, listener: (...args: any[]) => void): this

  prependListener(event: AddEventName | RemoveEventName, listener: () => void): this
  prependListener(event: AddEventName, listener: (newNode: Multiaddr) => void): this
  prependListener(event: RemoveEventName, listener: (removeNode: PeerId) => void): this
  prependListener(event: string | symbol, listener: (...args: any[]) => void): this

  prependOnceListener(event: AddEventName | RemoveEventName, listener: () => void): this
  prependOnceListener(event: AddEventName, listener: (newNode: Multiaddr) => void): this
  prependOnceListener(event: RemoveEventName, listener: (removeNode: PeerId) => void): this
  prependOnceListener(event: string | symbol, listener: (...args: any[]) => void): this

  removeListener(event: AddEventName | RemoveEventName, listener: () => void): this
  removeListener(event: AddEventName, listener: (newNode: Multiaddr) => void): this
  removeListener(event: RemoveEventName, listener: (removeNode: PeerId) => void): this
  removeListener(event: string | symbol, listener: (...args: any[]) => void): this
}
