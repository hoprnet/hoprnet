import {Crawler} from './crawler'
import {Heartbeat} from './heartbeat'
import {LibP2P} from '../../'
import type {Connection} from 'libp2p'
import PeerId from 'peer-id'

class NetworkInteractions {
  crawler: Crawler
  heartbeat: Heartbeat

  constructor(node: LibP2P, handleCrawlRequest: (conn: Connection) => void, heartbeat: (remotePeer: PeerId) => void) {
    this.crawler = new Crawler(node, handleCrawlRequest)
    this.heartbeat = new Heartbeat(node, heartbeat)
  }
}

export {NetworkInteractions}
