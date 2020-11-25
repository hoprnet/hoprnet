import { Crawler } from './crawler'
import { Heartbeat } from './heartbeat'
import { LibP2P } from '../../'
import PeerId from 'peer-id'
import type Multiaddr from 'multiaddr'

class NetworkInteractions {
  crawler: Crawler
  heartbeat: Heartbeat

  constructor(
    node: LibP2P,
    answerCrawl: (addr: Multiaddr) => Promise<Multiaddr[]>,
    heartbeat: (remotePeer: PeerId) => void
  ) {
    this.crawler = new Crawler(node, answerCrawl)
    this.heartbeat = new Heartbeat(node, heartbeat)
  }
}

export { NetworkInteractions }
