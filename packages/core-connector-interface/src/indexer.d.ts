import type { Balance, Public, ChannelEntry } from './types'

export type RoutingChannel = [source: PeerId, destination: PeerId, stake: Balance]
export type ChannelUpdate = { partyA: Public; partyB: Public; channelEntry: ChannelEntry }

declare interface Indexer {
  start(): Promise<void>
  stop(): Promise<void>

  getChannelEntry(partyA: Public, partyB: Public): Promise<ChannelEntry | undefined>
  getChannelEntries(party?: Public, filter?: (node: Public) => boolean): Promise<ChannelUpdate[]>

  // routing
  getRandomChannel(): Promise<RoutingChannel | undefined>
  getChannelsFromPeer(source: PeerId): Promise<RoutingChannel[]>
}

declare interface Indexer {
  on(event: 'channelOpened', listener: (routingChannel: RoutingChannel, update: ChannelUpdate) => void): this
  emit(event: 'channelOpened', routingChannel: RoutingChannel, update: ChannelUpdate): boolean
  on(event: 'channelClosed', listener: (routingChannel: RoutingChannel, update: ChannelUpdate) => void): this
  emit(event: 'channelClosed', routingChannel: RoutingChannel, update: ChannelUpdate): boolean
}

export default Indexer
