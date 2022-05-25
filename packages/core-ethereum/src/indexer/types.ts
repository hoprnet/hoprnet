import type { HoprToken, HoprChannels, HoprNetworkRegistry, TypedEventFilter } from '@hoprnet/hopr-ethereum'
import type PeerId from 'peer-id'
import type { Multiaddr } from 'multiaddr'
import type { ChannelEntry } from '@hoprnet/hopr-utils'

/**
 * Typechain does not provide us with clean event types, in the lines below we infer
 * the generic type from the 'HoprChannels.filters'.
 * This allows us to retrieve HoprChannel's events.
 */
type extractEventArgs<Type> = Type extends TypedEventFilter<infer A> ? A : null

export type EventNames = keyof HoprChannels['filters']
export type Event<T extends EventNames> = extractEventArgs<ReturnType<Pick<HoprChannels['filters'], T>[T]>>
export type TokenEventNames = keyof HoprToken['filters']
export type TokenEvent<T extends TokenEventNames> = extractEventArgs<ReturnType<Pick<HoprToken['filters'], T>[T]>>
export type RegistryEventNames = keyof HoprNetworkRegistry['filters']
export type RegistryEvent<T extends RegistryEventNames> = extractEventArgs<
  ReturnType<Pick<HoprNetworkRegistry['filters'], T>[T]>
>

export enum IndexerStatus {
  STARTING = 'starting',
  STARTED = 'started',
  RESTARTING = 'restarting',
  STOPPED = 'stopped'
}

export type IndexerEvents =
  | `announce-${string}`
  | `withdraw-hopr-${string}`
  | `withdraw-native-${string}`
  | `channel-updated-${string}`

type BlockEventName = 'block'
type BlockProcessedEventName = 'block-processed'
type StatusEventName = 'status'
type PeerEventName = 'peer'
type ChannelUpdateEventName = 'channel-update'
type OwnChannelUpdatedEventName = 'own-channel-updated'
type ChannelWaitingForCommitmentEventName = 'channel-waiting-for-commitment'

type IndexerEventNames =
  | BlockEventName
  | BlockProcessedEventName
  | StatusEventName
  | PeerEventName
  | ChannelUpdateEventName
  | OwnChannelUpdatedEventName
  | ChannelWaitingForCommitmentEventName

type BlockListener = (block: number) => void
type BlockProcessedListener = (block: number) => void
type StatusListener = (status: IndexerStatus) => void
type PeerListener = (peerData: { id: PeerId; multiaddrs: Multiaddr[] }) => void
type ChannelUpdateListener = (channel: ChannelEntry) => void
type OwnChannelUpdatedListener = (channel: ChannelEntry) => void
type ChannelWaitingForCommitmentListener = (channel: ChannelEntry) => void

export interface IndexerEventEmitter {
  addListener(event: IndexerEventNames, listener: () => void): this
  addListener(event: BlockEventName, listener: BlockListener): this
  addListener(event: BlockProcessedEventName, listener: BlockProcessedListener): this
  addListener(event: StatusEventName, listener: StatusListener): this
  addListener(event: PeerEventName, listener: PeerListener): this
  addListener(event: ChannelUpdateEventName, listener: ChannelUpdateListener): this
  addListener(event: OwnChannelUpdatedEventName, listener: OwnChannelUpdatedListener): this
  addListener(event: ChannelWaitingForCommitmentEventName, listener: ChannelWaitingForCommitmentListener): this

  emit(event: IndexerEventNames): boolean
  emit(event: BlockEventName, listener: BlockListener): boolean
  emit(event: BlockProcessedEventName, listener: BlockProcessedListener): boolean
  emit(event: StatusEventName, listener: StatusListener): boolean
  emit(event: PeerEventName, listener: PeerListener): boolean
  emit(event: ChannelUpdateEventName, listener: ChannelUpdateListener): boolean
  emit(event: OwnChannelUpdatedEventName, listener: OwnChannelUpdatedListener): boolean
  emit(event: ChannelWaitingForCommitmentEventName, listener: ChannelWaitingForCommitmentListener): boolean

  on(event: IndexerEventNames, listener: () => void): this
  on(event: BlockEventName, listener: BlockListener): this
  on(event: BlockProcessedEventName, listener: BlockProcessedListener): this
  on(event: StatusEventName, listener: StatusListener): this
  on(event: PeerEventName, listener: PeerListener): this
  on(event: ChannelUpdateEventName, listener: ChannelUpdateListener): this
  on(event: OwnChannelUpdatedEventName, listener: OwnChannelUpdatedListener): this
  on(event: ChannelWaitingForCommitmentEventName, listener: ChannelWaitingForCommitmentListener): this

  once(event: IndexerEventNames, listener: () => void): this
  once(event: BlockEventName, listener: BlockListener): this
  once(event: BlockProcessedEventName, listener: BlockProcessedListener): this
  once(event: StatusEventName, listener: StatusListener): this
  once(event: PeerEventName, listener: PeerListener): this
  once(event: ChannelUpdateEventName, listener: ChannelUpdateListener): this
  once(event: OwnChannelUpdatedEventName, listener: OwnChannelUpdatedListener): this
  once(event: ChannelWaitingForCommitmentEventName, listener: ChannelWaitingForCommitmentListener): this

  prependListener(event: IndexerEventNames, listener: () => void): this
  prependListener(event: BlockEventName, listener: BlockListener): this
  prependListener(event: BlockProcessedEventName, listener: BlockProcessedListener): this
  prependListener(event: StatusEventName, listener: StatusListener): this
  prependListener(event: PeerEventName, listener: PeerListener): this
  prependListener(event: ChannelUpdateEventName, listener: ChannelUpdateListener): this
  prependListener(event: OwnChannelUpdatedEventName, listener: OwnChannelUpdatedListener): this
  prependListener(event: ChannelWaitingForCommitmentEventName, listener: ChannelWaitingForCommitmentListener): this

  prependOnceListener(event: IndexerEventNames, listener: () => void): this
  prependOnceListener(event: BlockEventName, listener: BlockListener): this
  prependOnceListener(event: BlockProcessedEventName, listener: BlockProcessedListener): this
  prependOnceListener(event: StatusEventName, listener: StatusListener): this
  prependOnceListener(event: PeerEventName, listener: PeerListener): this
  prependOnceListener(event: ChannelUpdateEventName, listener: ChannelUpdateListener): this
  prependOnceListener(event: OwnChannelUpdatedEventName, listener: OwnChannelUpdatedListener): this
  prependOnceListener(event: ChannelWaitingForCommitmentEventName, listener: ChannelWaitingForCommitmentListener): this

  removeListener(event: IndexerEventNames, listener: () => void): this
  removeListener(event: BlockEventName, listener: BlockListener): this
  removeListener(event: BlockProcessedEventName, listener: BlockProcessedListener): this
  removeListener(event: StatusEventName, listener: StatusListener): this
  removeListener(event: PeerEventName, listener: PeerListener): this
  removeListener(event: ChannelUpdateEventName, listener: ChannelUpdateListener): this
  removeListener(event: OwnChannelUpdatedEventName, listener: OwnChannelUpdatedListener): this
  removeListener(event: ChannelWaitingForCommitmentEventName, listener: ChannelWaitingForCommitmentListener): this

  off(event: IndexerEventNames, listener: () => void): this
  off(event: BlockEventName, listener: BlockListener): this
  off(event: BlockProcessedEventName, listener: BlockProcessedListener): this
  off(event: StatusEventName, listener: StatusListener): this
  off(event: PeerEventName, listener: PeerListener): this
  off(event: ChannelUpdateEventName, listener: ChannelUpdateListener): this
  off(event: OwnChannelUpdatedEventName, listener: OwnChannelUpdatedListener): this
  off(event: ChannelWaitingForCommitmentEventName, listener: ChannelWaitingForCommitmentListener): this
}
