import type Hopr from '../../'
import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type { Types } from '@hoprnet/hopr-core-connector-interface'

import pipe from 'it-pipe'

import type { AbstractInteraction } from '../abstractInteraction'

import type { Handler } from '../../@types/transport'

import { PROTOCOL_PAYMENT_CHANNEL } from '../../constants'
import type PeerId from 'peer-id'

class Opening<Chain extends HoprCoreConnector> implements AbstractInteraction {
  protocols: string[] = [PROTOCOL_PAYMENT_CHANNEL]

  constructor(public node: Hopr<Chain>) {
    this.node._libp2p.handle(this.protocols, this.handler.bind(this))
  }

  async handler(struct: Handler) {
    pipe(
      struct.stream,
      this.node.paymentChannels.channel.handleOpeningRequest.bind(this.node.paymentChannels.channel),
      struct.stream
    )
  }

  async interact(counterparty: PeerId, channelBalance: Types.ChannelBalance): Promise<Types.SignedChannel> {
    let struct: Handler

    try {
      struct = await this.node._libp2p.dialProtocol(counterparty, this.protocols[0]).catch(async (_: Error) => {
        return this.node._libp2p.peerRouting
          .findPeer(counterparty)
          .then((peerRoute) => this.node._libp2p.dialProtocol(peerRoute.id, this.protocols[0]))
      })
    } catch (err) {
      throw Error(
        `Tried to open a payment channel but could not connect to ${counterparty.toB58String()}. Error was: ${
          err.message
        }`
      )
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
