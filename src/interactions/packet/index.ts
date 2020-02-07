import Hopr from '../..'
import { HoprCoreConnectorInstance } from '@hoprnet/hopr-core-connector-interface'

import { PacketForwardInteraction } from './forward'
import { PacketAcknowledgementInteraction } from './acknowledgement'

class PacketInteractions<Chain extends HoprCoreConnectorInstance> {
  acknowledgment: PacketAcknowledgementInteraction<Chain>
  forward: PacketForwardInteraction<Chain>

  constructor(node: Hopr<Chain>) {
    this.acknowledgment = new PacketAcknowledgementInteraction(node)
    this.forward = new PacketForwardInteraction(node)
  }
}

export { PacketInteractions }
