import Hopr from '../../'
import { HoprCoreConnectorInstance } from '@hoprnet/hopr-core-connector-interface'

import pipe from 'it-pipe'

import { AbstractInteraction, Duplex } from '../abstractInteraction'

import { PROTOCOL_CRAWLING } from '../../constants'
import PeerInfo from 'peer-info'
import PeerId from 'peer-id'

import fs from 'fs'
import protons = require('protons')

import path from 'path'
import { pubKeyToPeerId } from '../../utils'
const { CrawlResponse, Status } = protons(fs.readFileSync(path.resolve(__dirname, './protos/response.proto')))

class Crawler<Chain extends HoprCoreConnectorInstance> extends AbstractInteraction<Chain> {
  constructor(public node: Hopr<Chain>) {
    super(node, [PROTOCOL_CRAWLING])
  }

  async handler(struct: { stream: Duplex }) {
    pipe(
      /* prettier-ignore */
      struct.stream,
      this.node.network.crawler.handleCrawlRequest(CrawlResponse, Status),
      struct.stream
    )
  }

  async interact(counterparty: PeerId): Promise<PeerId[]> {
    let struct: {
      stream: Duplex
      protocol: string
    }

    try {
      struct = await this.node.dialProtocol(counterparty, this.protocols[0])
    } catch (err) {
      try {
        struct = await this.node.peerRouting.findPeer(counterparty).then((peerInfo: PeerInfo) => this.node.dialProtocol(peerInfo, PROTOCOL_CRAWLING))
      } catch (err) {
        return []
      }
    }

    return pipe(
      /** prettier-ignore */
      struct.stream,
      async function collect(source: AsyncIterable<Uint8Array>) {
        const peerIds = []
        for await (const encodedResponse of source) {
          let decodedResponse: any
          try {
            decodedResponse = CrawlResponse.decode(encodedResponse)
          } catch {
            continue
          }

          if (decodedResponse.status !== Status.OK) {
            continue
          }

          peerIds.push(Promise.all(decodedResponse.pubKeys.map((pubKey: Uint8Array) => pubKeyToPeerId(pubKey))))
        }

        return peerIds
      }
    )
  }
}

export { Crawler }
