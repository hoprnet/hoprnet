import type { Account, Address, Public, Balance, ChannelEntry, Hash } from './types'

export type RoutingChannel = [source: PeerId, destination: PeerId, stake: Balance]

export interface IndexerEvents {
  channelOpened: (update: ChannelEntry) => void
  channelClosed: (update: ChannelEntry) => void
}

declare interface Indexer {
  start(): Promise<void>
  stop(): Promise<void>

  // events
  on<U extends keyof IndexerEvents>(event: U, listener: IndexerEvents[U]): this
  once<U extends keyof IndexerEvents>(event: U, listener: IndexerEvents[U]): this
  emit<U extends keyof IndexerEvents>(event: U, ...args: Parameters<IndexerEvents[U]>): boolean

  getAccount(address: Address): Promise<Account | undefined>
  getChannel(channelId: Hash): Promise<ChannelEntry | undefined>
  getChannels(filter?: (channel: ChannelEntry) => Promise<boolean>): Promise<ChannelEntry[]>
  getChannelsOf(address: Address): Promise<ChannelEntry[]>

  // routing
  getPublicKeyOf(address: Address): Promise<Public | undefined>
  getRandomChannel(): Promise<RoutingChannel | undefined>
  getChannelsFromPeer(source: PeerId): Promise<RoutingChannel[]>
}

export default Indexer
