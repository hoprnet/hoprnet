import type Hopr from '../..'
import { PacketForwardInteraction } from './forward'
import { PacketAcknowledgementInteraction } from './acknowledgement'

class PacketInteractions {
  acknowledgment: PacketAcknowledgementInteraction
  forward: PacketForwardInteraction

  constructor(node: Hopr) {
    this.acknowledgment = new PacketAcknowledgementInteraction(node)
    this.forward = new PacketForwardInteraction(node)
  }
}

export { PacketInteractions }
