import {Public} from './types'

type ChannelEntry = {
  partyA: Public
  partyB: Public
}

declare interface Indexer {
  has(partyA: Public, partyB: Public): Promise<boolean>
  get(query?: {partyA?: Public; partyB?: Public}): Promise<ChannelEntry[]>
}

export {ChannelEntry}
export default Indexer
