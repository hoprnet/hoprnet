import type EventEmitter from 'events'
import { Public } from './types'

type ChannelEntry = {
  partyA: Public
  partyB: Public
}

type ChannelEvent = ChannelEntry & {
  id: Uint8Array
}

interface IndexerEvents {
  channelOpened: (channelId: ChannelEvent) => void
  channelClosed: (channelId: ChannelEvent) => void
}

declare interface Indexer extends EventEmitter {
  on<U extends keyof IndexerEvents>(event: U, listener: IndexerEvents[U]): this
  once<U extends keyof IndexerEvents>(event: U, listener: IndexerEvents[U]): this
  emit<U extends keyof IndexerEvents>(event: U, ...args: Parameters<IndexerEvents[U]>): boolean
  has(partyA: Public, partyB: Public): Promise<boolean>
  get(query?: { partyA?: Public; partyB?: Public }): Promise<ChannelEntry[]>
}

export { ChannelEntry }
export default Indexer
