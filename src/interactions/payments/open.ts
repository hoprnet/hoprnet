import pull from 'pull-stream'
import lp from 'pull-length-prefixed'
import Hopr from '../../'
import { HoprCoreConnectorInstance, Types } from '@hoprnet/hopr-core-connector-interface'

import { AbstractInteraction } from '../abstractInteraction'

import { PROTOCOL_PAYMENT_CHANNEL } from '../../constants'
import PeerId from 'peer-id'

class Opening<Chain extends HoprCoreConnectorInstance> extends AbstractInteraction<Chain> {
  constructor(public node: Hopr<Chain>) {
    super(node, PROTOCOL_PAYMENT_CHANNEL)
  }

  handle(protocol: any, conn: pull.Through<Buffer, Buffer>) {
    pull(
      conn,
      lp.decode(),
      pull.asyncMap((data: Buffer, cb: (err: Error | null, result?: Buffer) => void) => {
        this.node.paymentChannels.channel.handleOpeningRequest(this.node.paymentChannels, data).then(
          (data: Uint8Array) => cb(null, Buffer.from(data)),
          (err: Error) => cb(err)
        )
      }),
      lp.encode(),
      conn
    )
  }

  interact(counterparty: PeerId, channelBalance: Types.ChannelBalance): Promise<Types.SignedChannel> {
    return new Promise<Types.SignedChannel>((resolve, reject) => {
        this.node.dialProtocol(counterparty, PROTOCOL_PAYMENT_CHANNEL, (err: Error | null, conn: pull.Source<Buffer>) => {
            pull(
                pull.once(channelBalance.toU8a()),
                lp.encode(),
                conn,
                lp.decode(),
                pull.drain((data: Buffer) => resolve(new this.node.paymentChannels.types.SignedChannel()))
            )
        })
    })
  }
}

export { Opening }
