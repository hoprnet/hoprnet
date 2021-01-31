import type { Balance, Public, ChannelEntry } from './types'

export type RoutingChannel = [source: PeerId, destination: PeerId, stake: Balance]
export type ChannelUpdate = { partyA: Public; partyB: Public; channelEntry: ChannelEntry }

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
  getChannelEntry(partyA: Public, partyB: Public): Promise<ChannelEntry | undefined>
  getChannelEntries(party?: Public, filter?: (node: Public) => boolean): Promise<ChannelUpdate[]>

  // routing
  getRandomChannel(): Promise<RoutingChannel | undefined>
  getChannelsFromPeer(source: PeerId): Promise<RoutingChannel[]>
}

export default Indexer
