import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '@hoprnet/hopr-core'
import { AbstractCommand } from './abstractCommand'
import type PeerID from 'peer-id'

export default class TraverseChannels extends AbstractCommand {
  constructor(public node: Hopr<HoprCoreConnector>) {
    super()
  }

  public name() {
    return 'traverse'
  }

  public help() {
    return 'traverse the channels graph'
  }

  private async iter(depth: number, maxDepth: number, id: PeerID, prev: string, parent: string): Promise<string> {
    if (depth >= maxDepth) {
      return '' 
    }
    const chans = await this.node.paymentChannels.indexer.getChannelsFromPeer(id)
    if (chans.length == 0) {
      return `\n${prev} - ${id.toB58String()}*`
    } else {
      let out = ''
      for (let x of chans) {
        const [, peerId, weight] = x
        if (peerId.toB58String() === parent) {
          out += `\n${prev} - ${id.toB58String()} - [${weight}, BIDIRECTIONAL]`
        } else {
          if (depth + 1 < maxDepth) {
            out += await this.iter(
              depth + 1,
              maxDepth,
              peerId,
              `\n${prev} - ${id.toB58String()} - [${weight}]`,
              id.toB58String()
            )
          } else {
            out += `\n${prev} - ${id.toB58String()} - [${weight}] - ${peerId.toB58String()}...` 
          }
        }
      }
      return out
    }
  }

  public async execute(query: string): Promise<string> {
    let maxDepth = 2
    if (parseInt(query.trim(), 10)) {
      maxDepth = parseInt(query, 10)
    }
    return await this.iter(0, maxDepth, this.node.getId(), '', '')
  }
}
