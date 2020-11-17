import { Balance } from './types'

// Source -> Dest, stake
type Channel = [PeerId, PeerId, Balance]

declare interface Indexer {
  getChannelsFrom(source: PeerId): Promise<Channel[]>
}

export { Channel }
export default Indexer
