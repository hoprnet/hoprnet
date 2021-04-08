import type Hopr from '..'
import PeerId from 'peer-id'

import { Heartbeat } from './network/heartbeat'
import { PacketForwardInteraction } from './packet/forward'
import { PacketAcknowledgementInteraction } from './packet/acknowledgement'

class Interactions {
  heartbeat: Heartbeat
  acknowledgment: PacketAcknowledgementInteraction
  forward: PacketForwardInteraction

  constructor(node: Hopr, heartbeat: (remotePeer: PeerId) => void) {
    this.heartbeat = new Heartbeat(node._libp2p, heartbeat)
    this.acknowledgment = new PacketAcknowledgementInteraction(node._libp2p, node.db, node.paymentChannels)
    this.forward = new PacketForwardInteraction(node)
  }
}

export { Interactions }
