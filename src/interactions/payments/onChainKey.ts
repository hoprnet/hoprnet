import Hopr from '../../'
import { HoprCoreConnectorInstance } from '@hoprnet/hopr-core-connector-interface'

import { PROTOCOL_ONCHAIN_KEY } from '../../constants'
import { AbstractInteraction, Duplex } from '../abstractInteraction'
import PeerInfo from 'peer-info'

import pipe from 'it-pipe'

class OnChainKey<Chain extends HoprCoreConnectorInstance> extends AbstractInteraction<Chain> {
  constructor(public node: Hopr<Chain>) {
    super(node, [PROTOCOL_ONCHAIN_KEY])
  }

  handler(struct: { stream: Duplex }) {
    pipe(
      /* prettier-ignore */
      this.node.paymentChannels.self.keyPair.publicKey,
      struct.stream
    )
  }

  async interact(counterparty: PeerInfo): Promise<Uint8Array> {
    let struct: {
      stream: Duplex
      protocol: string
    }

    try {
      struct = await this.node.dialProtocol(counterparty, PROTOCOL_ONCHAIN_KEY)
    } catch {
      throw Error(`Tried to get onChain key from party ${counterparty.id.toB58String()} but failed while trying to connect to that node.`)
    }

    return pipe(
      /* prettier-ignore */
      struct.stream,
      async function(source: any) {
        const msgs: Uint8Array[] = []
        for await (const msg of source) {
          if (msg == null || msg.length == 0) {
            throw Error(`received ${msg} but expected a public key`)
          }

          if (msgs.length > 0) {
            continue
          } else {
            msgs.push(msg)
          }
        }

        return msgs[0]
      }
    )
  }
}

export { OnChainKey }
