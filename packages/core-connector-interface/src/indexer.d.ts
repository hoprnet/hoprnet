import type { Balance, Public, AccountId, ChannelEntry, AccountEntry } from './types'

export type RoutingChannel = [source: PeerId, destination: PeerId, stake: Balance]
export type ChannelUpdate = { partyA: Public; partyB: Public; channelEntry: ChannelEntry }
export type AccountUpdate = { account: Public; accountEntry: AccountEntry }

export interface IndexerEvents {
  channelOpened: (update: ChannelUpdate) => void
  channelClosed: (update: ChannelUpdate) => void
  accountUpdated: (update: AccountUpdate) => void
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

  // get saved account entries
  getAccountEntry(party: AccountId): Promise<AccountEntry | undefined>

  // routing
  getRandomChannel(): Promise<RoutingChannel | undefined>
  getChannelsFromPeer(source: PeerId): Promise<RoutingChannel[]>
}

export default Indexer
