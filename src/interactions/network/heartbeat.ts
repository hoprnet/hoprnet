import pull from 'pull-stream'
import lp from 'pull-length-prefixed'
import Hopr from '../../'
import { HoprCoreConnectorInstance, Types } from '@hoprnet/hopr-core-connector-interface'

import { AbstractInteraction } from '../abstractInteraction'

import { PROTOCOL_HEARTBEAT } from '../../constants'
import PeerId from 'peer-id'

class Opening<Chain extends HoprCoreConnectorInstance> extends AbstractInteraction<Chain> {
  constructor(public node: Hopr<Chain>) {
    super(node, PROTOCOL_HEARTBEAT)
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

  interact(counterparty: PeerId, channelBalance: Types.ChannelBalance): Promise<Uint8Array> {
    return new Promise<Uint8Array>((resolve, reject) => {
        this.node.dialProtocol(counterparty, PROTOCOL_HEARTBEAT, (err: Error | null, conn: pull.Source<Buffer>) => {
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
