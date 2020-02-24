import Hopr from '../../'
import { HoprCoreConnectorInstance } from '@hoprnet/hopr-core-connector-interface'

import { PROTOCOL_ONCHAIN_KEY } from '../../constants'
import { AbstractInteraction } from '../abstractInteraction'
import PeerInfo from 'peer-info'
import PeerId from 'peer-id'

import chalk from 'chalk'

import pipe from 'it-pipe'

class OnChainKey<Chain extends HoprCoreConnectorInstance> implements AbstractInteraction<Chain> {
  protocols: string[] = [PROTOCOL_ONCHAIN_KEY]

  constructor(public node: Hopr<Chain>) {
    this.node.handle(this.protocols, this.handler.bind(this))
  }

  handler(struct: { stream: any }) {
    pipe(
      /* prettier-ignore */
      [this.node.paymentChannels.self.keyPair.publicKey],
      struct.stream
    )
  }

  async interact(counterparty: PeerInfo | PeerId): Promise<Uint8Array> {
    let struct: {
      stream: any
      protocol: string
    }

    try {
      struct = await this.node.dialProtocol(counterparty, this.protocols[0]).catch(async (_: Error) => {
        return this.node.peerRouting
          .findPeer(PeerInfo.isPeerInfo(counterparty) ? counterparty.id : counterparty)
          .then((peerInfo: PeerInfo) => this.node.dialProtocol(peerInfo, this.protocols[0]))
      })
    } catch (err) {
      throw Error(
        `Tried to get onChain key from party ${(PeerInfo.isPeerInfo(counterparty)
          ? counterparty.id
          : counterparty
        ).toB58String()} but failed while trying to connect to that node. Error was: ${chalk.red(err.message)}`
      )
    }

    return pipe(
      /* prettier-ignore */
      struct.stream,
      onReception
    )
  }
}

async function onReception(source: any): Promise<Uint8Array> {
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

  return result
}

export { OnChainKey }
