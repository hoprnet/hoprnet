import { Balance } from './types'

type Channel = [source: PeerId, destination: PeerId, stake: Balance]

declare interface Indexer {
  getRandomChannel(): Promise<Channel | undefined>
  getChannelsFromPeer(source: PeerId): Promise<Channel[]>
  onNewChannels(handler: (newChannels: Channel[]) => void): void
}

export { Channel }
export default Indexer
