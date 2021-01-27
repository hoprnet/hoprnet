/// <reference path="../@types/libp2p.ts" />

import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '..'
import PeerId from 'peer-id'

import { PaymentInteractions } from './payments'
import { NetworkInteractions } from './network'
import { PacketInteractions } from './packet'
import { Mixer } from '../mixer'
import { Handler } from 'libp2p'

class Interactions<Chain extends HoprCoreConnector> {
  public payments: PaymentInteractions<Chain>
  public network: NetworkInteractions
  public packet: PacketInteractions<Chain>

  constructor(
    node: Hopr<Chain>,
    mixer: Mixer<Chain>,
    heartbeat: (remotePeer: PeerId) => void,
    dialProtocol: (counterparty: PeerId, protocols: string[], seconds: number) => Promise<Handler | void>
  ) {
    this.payments = new PaymentInteractions(node)
    this.network = new NetworkInteractions(node._libp2p, heartbeat, dialProtocol)
    this.packet = new PacketInteractions(node, mixer)
  }
}

export { Interactions }
