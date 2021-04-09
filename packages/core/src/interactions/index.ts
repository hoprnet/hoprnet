import type Hopr from '..'
import PeerId from 'peer-id'

import { Heartbeat } from './network/heartbeat'
import { PacketForwardInteraction } from './packet/forward'
import { PacketAcknowledgementInteraction } from './packet/acknowledgement'

class Interactions {
  acknowledgment: PacketAcknowledgementInteraction
  forward: PacketForwardInteraction

  constructor(private node: Hopr, heartbeat: Heartbeat) {
    this.acknowledgment = new PacketAcknowledgementInteraction(node._libp2p, node.db, node.paymentChannels)
    this.forward = new PacketForwardInteraction(node)
  }
}

export { Interactions }
