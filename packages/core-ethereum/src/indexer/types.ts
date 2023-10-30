import type { PeerId } from '@libp2p/interface-peer-id'
import type { Multiaddr } from '@multiformats/multiaddr'
import type { Address, ChannelEntry, Balance } from '@hoprnet/hopr-utils'

export enum IndexerStatus {
  STARTING = 'starting',
  STARTED = 'started',
  RESTARTING = 'restarting',
  STOPPED = 'stopped'
}

// Manual event typing because Node.js EventEmitter is untyped

export type IndexerEventsNames =
  | `announce-`
  | `token-approved-`
  | `withdraw-hopr-`
  | `withdraw-native-`
  | `channel-updated-`
  | `on-provider-error-`
  | `on-new-block-`
  | `node-safe-registered-`

export type IndexerEventsType = `${IndexerEventsNames}${string}`

export const BlockEventName = 'block'
export const BlockProcessedEventName = 'block-processed'
export const StatusEventName = 'status'
export const PeerEventName = 'peer'
export const NetworkRegistryEligibilityChangedEventName = 'network-registry-eligibility-changed'
export const NetworkRegistryStatusChangedEventName = 'network-registry-status-changed'
export const NetworkRegistryNodeAllowedEventName = 'network-registry-node-allowed'
export const NetworkRegistryNodeNotAllowedEventName = 'network-registry-node-not-allowed'

export const ChannelUpdateEventNames = 'own-channel-updated'
export const TicketRedeemedEventNames = 'ticket-redeemed'

type BlockEventNameType = 'block'
type BlockProcessedEventNameType = 'block-processed'
type StatusEventNameType = 'status'
type PeerEventNameType = 'peer'
type NetworkRegistryEligibilityChangedEventNameType = 'network-registry-eligibility-changed'
type NetworkRegistryStatusChangedEventNameType = 'network-registry-status-changed'
type NetworkRegistryNodeAllowedEventNameType = 'network-registry-node-allowed'
type NetworkRegistryNodeNotAllowedEventNameType = 'network-registry-node-not-allowed'
type ChannelUpdateEventNamesType = 'own-channel-updated'
type TicketRedeemedEventNamesType = 'ticket-redeemed'

type IndexerEventNames =
  | BlockEventNameType
  | BlockProcessedEventNameType
  | StatusEventNameType
  | PeerEventNameType
  | ChannelUpdateEventNamesType
  | TicketRedeemedEventNamesType
  | IndexerEventsType
  | NetworkRegistryEligibilityChangedEventNameType
  | NetworkRegistryStatusChangedEventNameType
  | NetworkRegistryNodeAllowedEventNameType
  | NetworkRegistryNodeNotAllowedEventNameType

type BlockListener = (block: number) => void
type BlockProcessedListener = (block: number) => void
type StatusListener = (status: IndexerStatus) => void
type PeerListener = (peerData: { id: PeerId; address: Address; multiaddrs: Multiaddr[] }) => void
type ChannelUpdateListener = (channel: ChannelEntry) => void
type TicketRedeemedListener = (channel: ChannelEntry, ticketAmount: Balance) => void
type IndexerEventsListener = (txHash: string) => void
type NetworkRegistryEligibilityChangedListener = (account: Address, allowed: boolean) => void
type NetworkRegistryStatusChangedListener = (isEnabled: boolean) => void
type NetworkRegistryNodeAllowedListener = (node: Address) => void
type NetworkRegistryNodeNotAllowedListener = (node: Address) => void

export interface IndexerEventEmitter {
  addListener(event: IndexerEventNames, listener: () => void): this
  addListener(event: BlockEventNameType, listener: BlockListener): this
  addListener(event: BlockProcessedEventNameType, listener: BlockProcessedListener): this
  addListener(event: StatusEventNameType, listener: StatusListener): this
  addListener(event: PeerEventNameType, listener: PeerListener): this
  addListener(event: ChannelUpdateEventNamesType, listener: ChannelUpdateListener): this
  addListener(event: TicketRedeemedEventNamesType, listener: TicketRedeemedListener): this
  addListener(event: IndexerEventsType, listener: IndexerEventsListener): this
  addListener(
    event: NetworkRegistryEligibilityChangedEventNameType,
    listener: NetworkRegistryEligibilityChangedListener
  ): this
  addListener(event: NetworkRegistryStatusChangedEventNameType, listener: NetworkRegistryStatusChangedListener): this
  addListener(event: NetworkRegistryNodeNotAllowedEventNameType, listener: NetworkRegistryNodeAllowedListener): this
  addListener(event: NetworkRegistryNodeNotAllowedEventNameType, listener: NetworkRegistryNodeNotAllowedListener): this

  emit(event: IndexerEventNames): boolean
  emit(event: BlockEventNameType, block: number): boolean
  emit(event: BlockProcessedEventNameType, block: number): boolean
  emit(event: StatusEventNameType, status: IndexerStatus): boolean
  emit(event: PeerEventNameType, peerData: { id: PeerId; address: Address; multiaddrs: Multiaddr[] }): boolean
  emit(event: ChannelUpdateEventNamesType, channel: ChannelEntry): boolean
  emit(event: TicketRedeemedEventNamesType, channel: ChannelEntry, ticketAmount: Balance): boolean
  emit(event: IndexerEventsType, txHash: string): boolean
  emit(event: NetworkRegistryEligibilityChangedEventNameType, account: Address, allowed: boolean): boolean
  emit(event: NetworkRegistryStatusChangedEventNameType, isEnabled: boolean): boolean
  emit(event: NetworkRegistryNodeAllowedEventNameType, node: Address): boolean
  emit(event: NetworkRegistryNodeNotAllowedEventNameType, node: Address): boolean

  on(event: IndexerEventNames, listener: () => void): this
  on(event: BlockEventNameType, listener: BlockListener): this
  on(event: BlockProcessedEventNameType, listener: BlockProcessedListener): this
  on(event: StatusEventNameType, listener: StatusListener): this
  on(event: PeerEventNameType, listener: PeerListener): this
  on(event: ChannelUpdateEventNamesType, listener: ChannelUpdateListener): this
  on(event: TicketRedeemedEventNamesType, listener: TicketRedeemedListener): this
  on(event: IndexerEventsType, listener: IndexerEventsListener): this
  on(event: NetworkRegistryEligibilityChangedEventNameType, listener: NetworkRegistryEligibilityChangedListener): this
  on(event: NetworkRegistryStatusChangedEventNameType, listener: NetworkRegistryStatusChangedListener): this
  on(event: NetworkRegistryNodeAllowedEventNameType, listener: NetworkRegistryNodeAllowedListener): this
  on(event: NetworkRegistryNodeNotAllowedEventNameType, listener: NetworkRegistryNodeNotAllowedListener): this

  once(event: IndexerEventNames, listener: () => void): this
  once(event: BlockEventNameType, listener: BlockListener): this
  once(event: BlockProcessedEventNameType, listener: BlockProcessedListener): this
  once(event: StatusEventNameType, listener: StatusListener): this
  once(event: PeerEventNameType, listener: PeerListener): this
  once(event: ChannelUpdateEventNamesType, listener: ChannelUpdateListener): this
  once(event: TicketRedeemedEventNamesType, listener: TicketRedeemedListener): this
  once(event: IndexerEventsType, listener: IndexerEventsListener): this
  once(event: NetworkRegistryEligibilityChangedEventNameType, listener: NetworkRegistryEligibilityChangedListener): this
  once(event: NetworkRegistryStatusChangedEventNameType, listener: NetworkRegistryStatusChangedListener): this
  once(event: NetworkRegistryNodeNotAllowedEventNameType, listener: NetworkRegistryNodeAllowedListener): this
  once(event: NetworkRegistryNodeNotAllowedEventNameType, listener: NetworkRegistryNodeNotAllowedListener): this

  prependListener(event: IndexerEventNames, listener: () => void): this
  prependListener(event: BlockEventNameType, listener: BlockListener): this
  prependListener(event: BlockProcessedEventNameType, listener: BlockProcessedListener): this
  prependListener(event: StatusEventNameType, listener: StatusListener): this
  prependListener(event: PeerEventNameType, listener: PeerListener): this
  prependListener(event: ChannelUpdateEventNamesType, listener: ChannelUpdateListener): this
  prependListener(event: TicketRedeemedEventNamesType, listener: TicketRedeemedListener): this
  prependListener(event: IndexerEventsType, listener: IndexerEventsListener): this
  prependListener(
    event: NetworkRegistryEligibilityChangedEventNameType,
    listener: NetworkRegistryEligibilityChangedListener
  ): this
  prependListener(
    event: NetworkRegistryStatusChangedEventNameType,
    listener: NetworkRegistryStatusChangedListener
  ): this
  prependListener(event: NetworkRegistryNodeNotAllowedEventNameType, listener: NetworkRegistryNodeAllowedListener): this
  prependListener(
    event: NetworkRegistryNodeNotAllowedEventNameType,
    listener: NetworkRegistryNodeNotAllowedListener
  ): this

  prependOnceListener(event: IndexerEventNames, listener: () => void): this
  prependOnceListener(event: BlockEventNameType, listener: BlockListener): this
  prependOnceListener(event: BlockProcessedEventNameType, listener: BlockProcessedListener): this
  prependOnceListener(event: StatusEventNameType, listener: StatusListener): this
  prependOnceListener(event: PeerEventNameType, listener: PeerListener): this
  prependOnceListener(event: ChannelUpdateEventNamesType, listener: ChannelUpdateListener): this
  prependOnceListener(event: TicketRedeemedEventNamesType, listener: TicketRedeemedListener): this
  prependOnceListener(event: IndexerEventsType, listener: IndexerEventsListener): this
  prependOnceListener(
    event: NetworkRegistryEligibilityChangedEventNameType,
    listener: NetworkRegistryEligibilityChangedListener
  ): this
  prependOnceListener(
    event: NetworkRegistryStatusChangedEventNameType,
    listener: NetworkRegistryStatusChangedListener
  ): this
  prependOnceListener(
    event: NetworkRegistryNodeNotAllowedEventNameType,
    listener: NetworkRegistryNodeAllowedListener
  ): this
  prependOnceListener(
    event: NetworkRegistryNodeNotAllowedEventNameType,
    listener: NetworkRegistryNodeNotAllowedListener
  ): this

  removeListener(event: IndexerEventNames, listener: () => void): this
  removeListener(event: BlockEventNameType, listener: BlockListener): this
  removeListener(event: BlockProcessedEventNameType, listener: BlockProcessedListener): this
  removeListener(event: StatusEventNameType, listener: StatusListener): this
  removeListener(event: PeerEventNameType, listener: PeerListener): this
  removeListener(event: ChannelUpdateEventNamesType, listener: ChannelUpdateListener): this
  removeListener(event: TicketRedeemedEventNamesType, listener: TicketRedeemedListener): this
  removeListener(event: IndexerEventsType, listener: IndexerEventsListener): this
  removeListener(
    event: NetworkRegistryEligibilityChangedEventNameType,
    listener: NetworkRegistryEligibilityChangedListener
  ): this
  removeListener(event: NetworkRegistryStatusChangedEventNameType, listener: NetworkRegistryStatusChangedListener): this
  removeListener(event: NetworkRegistryNodeNotAllowedEventNameType, listener: NetworkRegistryNodeAllowedListener): this
  removeListener(
    event: NetworkRegistryNodeNotAllowedEventNameType,
    listener: NetworkRegistryNodeNotAllowedListener
  ): this

  off(event: IndexerEventNames, listener: () => void): this
  off(event: BlockEventNameType, listener: BlockListener): this
  off(event: BlockProcessedEventNameType, listener: BlockProcessedListener): this
  off(event: StatusEventNameType, listener: StatusListener): this
  off(event: PeerEventNameType, listener: PeerListener): this
  off(event: ChannelUpdateEventNamesType, listener: ChannelUpdateListener): this
  off(event: TicketRedeemedEventNamesType, listener: TicketRedeemedListener): this
  off(event: IndexerEventsType, listener: IndexerEventsListener): this
  off(event: NetworkRegistryEligibilityChangedEventNameType, listener: NetworkRegistryEligibilityChangedListener): this
  off(event: NetworkRegistryStatusChangedEventNameType, listener: NetworkRegistryStatusChangedListener): this
  off(event: NetworkRegistryNodeNotAllowedEventNameType, listener: NetworkRegistryNodeAllowedListener): this
  off(event: NetworkRegistryNodeNotAllowedEventNameType, listener: NetworkRegistryNodeNotAllowedListener): this

  listeners(event: IndexerEventNames): Function[]
}
