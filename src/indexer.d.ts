import { AccountId } from './types'

type ChannelEntry = {
  partyA: AccountId
  partyB: AccountId
}

declare interface Indexer {
  has(partyA: AccountId, partyB: AccountId): Promise<boolean>
  get(query?: { partyA?: AccountId; partyB?: AccountId }): Promise<ChannelEntry[]>
}

export { ChannelEntry }
export default Indexer
