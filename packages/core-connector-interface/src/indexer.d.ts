import type { Balance, Public, ChannelEntry, Hash } from './types'

export type RoutingChannel = [source: PeerId, destination: PeerId, stake: Balance]
export type ChannelUpdate = { channelId: Hash; channelEntry: ChannelEntry }

export interface IndexerEvents {
  channelOpened: (update: ChannelUpdate) => void
  channelClosed: (update: ChannelUpdate) => void
}

declare interface Indexer {
  start(): Promise<void>
  stop(): Promise<void>

  // events
  on<U extends keyof IndexerEvents>(event: U, listener: IndexerEvents[U]): this
  once<U extends keyof IndexerEvents>(event: U, listener: IndexerEvents[U]): this
  emit<U extends keyof IndexerEvents>(event: U, ...args: Parameters<IndexerEvents[U]>): boolean

  // get saved channel entries
  getChannel(channelId: Hash): Promise<ChannelEntry | undefined>
  getChannelsOf(accountId: AccountId): Promise<ChannelEntry[]>

  // routing
  getRandomChannel(): Promise<RoutingChannel | undefined>
  getChannelsFromPeer(source: PeerId): Promise<RoutingChannel[]>
}

export default Indexer
