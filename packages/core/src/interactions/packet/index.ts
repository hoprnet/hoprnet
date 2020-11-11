import type Hopr from '../..'
import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'

import { PacketForwardInteraction } from './forward'
import { PacketAcknowledgementInteraction } from './acknowledgement'
import { Mixer } from '../../mixer'

class PacketInteractions<Chain extends HoprCoreConnector> {
  acknowledgment: PacketAcknowledgementInteraction<Chain>
  forward: PacketForwardInteraction<Chain>

  constructor(node: Hopr<Chain>, mixer: Mixer<Chain>) {
    this.acknowledgment = new PacketAcknowledgementInteraction(node)
    this.forward = new PacketForwardInteraction(node, mixer)
  }
}

export { PacketInteractions }
