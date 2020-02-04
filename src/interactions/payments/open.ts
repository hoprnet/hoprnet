import Hopr from '../../'
import { HoprCoreConnectorInstance, Types } from '@hoprnet/hopr-core-connector-interface'

import pipe from 'it-pipe'

import { AbstractInteraction, Duplex } from '../abstractInteraction'

import { PROTOCOL_PAYMENT_CHANNEL } from '../../constants'
import PeerInfo from 'peer-info'

class Opening<Chain extends HoprCoreConnectorInstance> extends AbstractInteraction<Chain> {
  constructor(public node: Hopr<Chain>) {
    super(node, [PROTOCOL_PAYMENT_CHANNEL])
  }

  async handler(struct: { stream: Duplex }) {
    pipe(
      /** prettier-ignore */
      struct.stream,
      this.node.paymentChannels.channel.handleOpeningRequest(this.node),
      struct.stream
    )
  }

  async interact(counterparty: PeerInfo, channelBalance: Types.ChannelBalance): Promise<Types.SignedChannel> {
    let struct: {
      stream: Duplex
      protocol: string
    }

    try {
      struct = await this.node.dialProtocol(counterparty, this.protocols[0])
    } catch (err) {
      console.log(struct)
      throw Error(`Tried to open a payment channel but could not connect to ${counterparty.id.toB58String()}. Error was: ${err.message}`)
    }

    return pipe(
      /* prettier-ignore */
      [channelBalance.toU8a()],
      struct.stream,
      async function collect(source: any) {
        let msgs: Uint8Array[] = []
        for await (const msg of source) {
          msgs.push(msg)
        }
        return msgs
      }
    )
  }
}

export { Opening }
