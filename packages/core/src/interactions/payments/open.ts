import type Hopr from '../../'
import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type { Types } from '@hoprnet/hopr-core-connector-interface'

import pipe from 'it-pipe'
import type { Connection, MuxedStream } from 'libp2p'
import type { AbstractInteraction } from '../abstractInteraction'
import { PROTOCOL_PAYMENT_CHANNEL } from '../../constants'
import type PeerId from 'peer-id'
import { dialHelper, durations } from '@hoprnet/hopr-utils'

const CHANNEL_OPEN_TIMEOUT = durations.seconds(4)
class Opening<Chain extends HoprCoreConnector> implements AbstractInteraction {
  protocols: string[] = [PROTOCOL_PAYMENT_CHANNEL]

  constructor(public node: Hopr<Chain>) {
    this.node._libp2p.handle(this.protocols, this.handler.bind(this))
  }

  handler(struct: { connection: Connection; stream: MuxedStream; protocol: string }) {
    pipe(
      struct.stream,
      this.node.paymentChannels.channel.handleOpeningRequest.bind(this.node.paymentChannels.channel),
      struct.stream
    )
  }

  async interact(counterparty: PeerId, channelBalance: Types.ChannelBalance): Promise<Types.SignedChannel> {
    const struct = await dialHelper(this.node._libp2p, counterparty, this.protocols[0], {
      timeout: CHANNEL_OPEN_TIMEOUT
    })

    if (struct == undefined) {
      throw Error(`Tried to open a payment channel but could not connect to ${counterparty.toB58String()}.`)
    }

    const channel = this.node.paymentChannels.types.Channel.createFunded(channelBalance)
    const signedChannel = await this.node.paymentChannels.types.SignedChannel.create(undefined, { channel })

    await channel.sign(this.node.paymentChannels.account.keys.onChain.privKey, undefined, {
      bytes: signedChannel.buffer,
      offset: signedChannel.signatureOffset
    })

    return await pipe([signedChannel], struct.stream, this.collect.bind(this))
  }

  private async collect(source: any) {
    let result: Uint8Array | undefined
    for await (const msg of source) {
      if (result != null) {
        continue
      } else {
        result = msg.slice()
      }
    }

    if (result == null) {
      throw Error('Empty stream')
    }

    return this.node.paymentChannels.types.SignedChannel.create({
      bytes: result.buffer,
      offset: result.byteOffset
    })
  }
}

export { Opening }
