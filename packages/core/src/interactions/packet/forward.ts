import { PROTOCOL_STRING } from '../../constants'
import { Packet } from '../../messages/packet'

import type PeerId from 'peer-id'

import type { AbstractInteraction } from '../abstractInteraction'
import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '../../'
import pipe from 'it-pipe'

import type { Connection, MuxedStream } from 'libp2p'
import { dialHelper, durations } from '@hoprnet/hopr-utils'
import { Mixer } from '../../mixer'

const FORWARD_TIMEOUT = durations.seconds(6)

class PacketForwardInteraction<Chain extends HoprCoreConnector> implements AbstractInteraction {
  protocols: string[] = [PROTOCOL_STRING]

  constructor(public node: Hopr<Chain>, private mixer: Mixer<Chain>) {
    this.node._libp2p.handle(this.protocols, this.handler.bind(this))
  }

  async interact(counterparty: PeerId, packet: Packet<Chain>): Promise<void> {
    const struct = await dialHelper(this.node._libp2p, counterparty, this.protocols, {
      timeout: FORWARD_TIMEOUT
    })

    if (struct == undefined) {
      throw Error(`Failed to send packet to ${counterparty.toB58String()}.`)
    }

    pipe([packet], struct.stream)
  }

  handler(struct: { connection: Connection; stream: MuxedStream; protocol: string }) {
    pipe(
      /* pretttier-ignore */
      struct.stream,
      async (source: AsyncIterable<Uint8Array>): Promise<void> => {
        for await (const msg of source) {
          const arr = msg.slice()
          const packet = new Packet(this.node, this.node._libp2p, {
            bytes: arr.buffer,
            offset: arr.byteOffset
          })

          this.mixer.push(packet)
        }
      }
    )
  }
}

export { PacketForwardInteraction }
