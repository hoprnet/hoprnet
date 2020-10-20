import type Hopr from '../../'
import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type { Types } from '@hoprnet/hopr-core-connector-interface'

import { PROTOCOL_ONCHAIN_KEY } from '../../constants'
import type { AbstractInteraction } from '../abstractInteraction'
import type PeerInfo from 'peer-info'
import type PeerId from 'peer-id'

import type { Handler } from '../../@types/transport'

import chalk from 'chalk'

import pipe from 'it-pipe'

class OnChainKey<Chain extends HoprCoreConnector> implements AbstractInteraction {
  protocols: string[] = [PROTOCOL_ONCHAIN_KEY]

  constructor(public node: Hopr<Chain>) {
    this.node._libp2p.handle(this.protocols, this.handler.bind(this))
  }

  handler(struct: Handler) {
    pipe([this.node.paymentChannels.account.keys.onChain.pubKey], struct.stream)
  }

  async interact(counterparty: PeerId): Promise<Types.Public> {
    let struct: Handler

    try {
      struct = await this.node._libp2p.dialProtocol(counterparty, this.protocols[0]).catch(async (_: Error) => {
        return this.node._libp2p.peerRouting
          .findPeer(counterparty)
          .then((peerInfo: PeerInfo) => this.node._libp2p.dialProtocol(peerInfo, this.protocols[0]))
      })
    } catch (err) {
      throw Error(
        `Tried to get onChain key from party ${counterparty.toB58String()} but failed while trying to connect to that node. Error was: ${chalk.red(
          err.message
        )}`
      )
    }

    return pipe(struct.stream, onReception)
  }
}

async function onReception(source: any): Promise<Types.Public> {
  let result: Uint8Array
  for await (const msg of source) {
    if (msg == null || msg.length == 0) {
      throw Error(`received ${msg} but expected a public key`)
    }

    if (result != null) {
      // ignore any further messages
      continue
    } else {
      result = msg.slice()
    }
  }

  return new this.node.paymentChannels.types.Public(result)
}

export { OnChainKey }
