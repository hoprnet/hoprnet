import type { PeerId } from '@libp2p/interface-peer-id'
import type { Multiaddr } from '@multiformats/multiaddr'
import type { Address, ChannelEntry } from '@hoprnet/hopr-utils'

export enum IndexerStatus {
  STARTING = 'starting',
  STARTED = 'started',
  RESTARTING = 'restarting',
  STOPPED = 'stopped'
}

// Manual event typing because Node.js EventEmitter is untyped

export type IndexerEvents =
  | `announce-${string}`
  | `token-approved-${string}`
  | `withdraw-hopr-${string}`
  | `withdraw-native-${string}`
  | `channel-updated-${string}`
  | `on-provider-error-${string}`
  | `on-new-block-${string}`
  | `node-safe-registered-${string}`

type BlockEventName = 'block'
type BlockProcessedEventName = 'block-processed'
type StatusEventName = 'status'
type PeerEventName = 'peer'
type NetworkRegistryEligibilityChangedEventName = 'network-registry-eligibility-changed'
type NetworkRegistryStatusChangedEventName = 'network-registry-status-changed'

type ChannelUpdateEventNames = 'own-channel-updated'

type IndexerEventNames =
  | BlockEventName
  | BlockProcessedEventName
  | StatusEventName
  | PeerEventName
  | ChannelUpdateEventNames
  | IndexerEvents
  | NetworkRegistryEligibilityChangedEventName
  | NetworkRegistryStatusChangedEventName

type BlockListener = (block: number) => void
type BlockProcessedListener = (block: number) => void
type StatusListener = (status: IndexerStatus) => void
type PeerListener = (peerData: { id: PeerId; multiaddrs: Multiaddr[] }) => void
type ChannelUpdateListener = (channel: ChannelEntry) => void
type IndexerEventsListener = (txHash: string) => void
type NetworkRegistryEligibilityChangedListener = (account: Address, allowed: boolean) => void
type NetworkRegistryStatusChangedListener = (isEnabled: boolean) => void

export interface IndexerEventEmitter {
  addListener(event: IndexerEventNames, listener: () => void): this
  addListener(event: BlockEventName, listener: BlockListener): this
  addListener(event: BlockProcessedEventName, listener: BlockProcessedListener): this
  addListener(event: StatusEventName, listener: StatusListener): this
  addListener(event: PeerEventName, listener: PeerListener): this
  addListener(event: ChannelUpdateEventNames, listener: ChannelUpdateListener): this
  addListener(event: IndexerEvents, listener: IndexerEventsListener): this
  addListener(
    event: NetworkRegistryEligibilityChangedEventName,
    listener: NetworkRegistryEligibilityChangedListener
  ): this
  addListener(event: NetworkRegistryStatusChangedEventName, listener: NetworkRegistryStatusChangedListener): this

  emit(event: IndexerEventNames): boolean
  emit(event: BlockEventName, block: number): boolean
  emit(event: BlockProcessedEventName, block: number): boolean
  emit(event: StatusEventName, status: IndexerStatus): boolean
  emit(event: PeerEventName, peerData: { id: PeerId; multiaddrs: Multiaddr[] }): boolean
  emit(event: ChannelUpdateEventNames, channel: ChannelEntry): boolean
  emit(event: IndexerEvents, txHash: string): boolean
  emit(event: NetworkRegistryEligibilityChangedEventName, account: Address, allowed: boolean): boolean
  emit(event: NetworkRegistryStatusChangedEventName, isEnabled: boolean): boolean

  on(event: IndexerEventNames, listener: () => void): this
  on(event: BlockEventName, listener: BlockListener): this
  on(event: BlockProcessedEventName, listener: BlockProcessedListener): this
  on(event: StatusEventName, listener: StatusListener): this
  on(event: PeerEventName, listener: PeerListener): this
  on(event: ChannelUpdateEventNames, listener: ChannelUpdateListener): this
  on(event: IndexerEvents, listener: IndexerEventsListener): this
  on(event: NetworkRegistryEligibilityChangedEventName, listener: NetworkRegistryEligibilityChangedListener): this
  on(event: NetworkRegistryStatusChangedEventName, listener: NetworkRegistryStatusChangedListener): this

  once(event: IndexerEventNames, listener: () => void): this
  once(event: BlockEventName, listener: BlockListener): this
  once(event: BlockProcessedEventName, listener: BlockProcessedListener): this
  once(event: StatusEventName, listener: StatusListener): this
  once(event: PeerEventName, listener: PeerListener): this
  once(event: ChannelUpdateEventNames, listener: ChannelUpdateListener): this
  once(event: IndexerEvents, listener: IndexerEventsListener): this
  once(event: NetworkRegistryEligibilityChangedEventName, listener: NetworkRegistryEligibilityChangedListener): this
  once(event: NetworkRegistryStatusChangedEventName, listener: NetworkRegistryStatusChangedListener): this

  prependListener(event: IndexerEventNames, listener: () => void): this
  prependListener(event: BlockEventName, listener: BlockListener): this
  prependListener(event: BlockProcessedEventName, listener: BlockProcessedListener): this
  prependListener(event: StatusEventName, listener: StatusListener): this
  prependListener(event: PeerEventName, listener: PeerListener): this
  prependListener(event: ChannelUpdateEventNames, listener: ChannelUpdateListener): this
  prependListener(event: IndexerEvents, listener: IndexerEventsListener): this
  prependListener(
    event: NetworkRegistryEligibilityChangedEventName,
    listener: NetworkRegistryEligibilityChangedListener
  ): this
  prependListener(event: NetworkRegistryStatusChangedEventName, listener: NetworkRegistryStatusChangedListener): this

  prependOnceListener(event: IndexerEventNames, listener: () => void): this
  prependOnceListener(event: BlockEventName, listener: BlockListener): this
  prependOnceListener(event: BlockProcessedEventName, listener: BlockProcessedListener): this
  prependOnceListener(event: StatusEventName, listener: StatusListener): this
  prependOnceListener(event: PeerEventName, listener: PeerListener): this
  prependOnceListener(event: ChannelUpdateEventNames, listener: ChannelUpdateListener): this
  prependOnceListener(event: IndexerEvents, listener: IndexerEventsListener): this
  prependOnceListener(
    event: NetworkRegistryEligibilityChangedEventName,
    listener: NetworkRegistryEligibilityChangedListener
  ): this
  prependOnceListener(
    event: NetworkRegistryStatusChangedEventName,
    listener: NetworkRegistryStatusChangedListener
  ): this

  removeListener(event: IndexerEventNames, listener: () => void): this
  removeListener(event: BlockEventName, listener: BlockListener): this
  removeListener(event: BlockProcessedEventName, listener: BlockProcessedListener): this
  removeListener(event: StatusEventName, listener: StatusListener): this
  removeListener(event: PeerEventName, listener: PeerListener): this
  removeListener(event: ChannelUpdateEventNames, listener: ChannelUpdateListener): this
  removeListener(event: IndexerEvents, listener: IndexerEventsListener): this
  removeListener(
    event: NetworkRegistryEligibilityChangedEventName,
    listener: NetworkRegistryEligibilityChangedListener
  ): this
  removeListener(event: NetworkRegistryStatusChangedEventName, listener: NetworkRegistryStatusChangedListener): this

  off(event: IndexerEventNames, listener: () => void): this
  off(event: BlockEventName, listener: BlockListener): this
  off(event: BlockProcessedEventName, listener: BlockProcessedListener): this
  off(event: StatusEventName, listener: StatusListener): this
  off(event: PeerEventName, listener: PeerListener): this
  off(event: ChannelUpdateEventNames, listener: ChannelUpdateListener): this
  off(event: IndexerEvents, listener: IndexerEventsListener): this
  off(event: NetworkRegistryEligibilityChangedEventName, listener: NetworkRegistryEligibilityChangedListener): this
  off(event: NetworkRegistryStatusChangedEventName, listener: NetworkRegistryStatusChangedListener): this

  listeners(event: IndexerEventNames): Function[]
}
