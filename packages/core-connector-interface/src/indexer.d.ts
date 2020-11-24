import { Balance } from './types'

// Source -> Dest, stake
type Channel = [PeerId, PeerId, Balance]

declare interface Indexer {
  getRandomChannel(): Promise<Channel>
  getChannelsFromPeer(source: PeerId): Promise<Channel[]>
  onNewChannels(handler: (newChannels: Channel[]) => void): void
}

export { Channel }
export default Indexer
