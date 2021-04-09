import { AbstractInteraction } from '../abstractInteraction'
import type { LevelUp } from 'levelup'
import type { Connection, MuxedStream } from 'libp2p'
import EventEmitter from 'events'
import pipe from 'it-pipe'
import PeerId from 'peer-id'
import debug from 'debug'
const log = debug('hopr-core:acknowledgement')
import { AcknowledgementMessage } from '../../messages/acknowledgement'
import { LibP2P } from '../../'
import { u8aToHex } from '@hoprnet/hopr-utils'
import {
  getUnacknowledgedTickets,
  deleteTicket,
  replaceTicketWithAcknowledgement,
  UnAcknowledgedTickets
} from '../../dbKeys'
import { PROTOCOL_ACKNOWLEDGEMENT } from '../../constants'
import { dialHelper, durations } from '@hoprnet/hopr-utils'

const ACKNOWLEDGEMENT_TIMEOUT = durations.seconds(2)

class PacketAcknowledgementInteraction extends EventEmitter implements AbstractInteraction {
  protocols: string[] = [PROTOCOL_ACKNOWLEDGEMENT]

  constructor(private libp2p: LibP2P, private db: LevelUp, private paymentChannels: any) {
    super()
    this.libp2p.handle(this.protocols, this.handler.bind(this))
  }

  handler(struct: { connection: Connection; stream: MuxedStream; protocol: string }) {
    pipe(struct.stream, this.handleHelper.bind(this))
  }

  async interact(counterparty: PeerId, acknowledgementMsg: AcknowledgementMessage): Promise<void> {
    const struct = await dialHelper(this.libp2p, counterparty, this.protocols[0], {
      timeout: ACKNOWLEDGEMENT_TIMEOUT
    })

    if (struct == undefined) {
      log(`ERROR: Could not send acknowledgement to party ${counterparty.toB58String()}.`)
    }

    pipe([acknowledgementMsg], struct.stream)
  }

  async handleAcknowledgementMessage(ackMsg: AcknowledgementMessage) {
    let unacknowledgedTicket = await getUnacknowledgedTickets(this.db, ackMsg.getHashedKey())
    if (!unacknowledgedTicket) {
      // Could be dummy, could be error.
      log('dropping unknown ticket')
      return await deleteTicket(this.db, ackMsg.getHashedKey())
    }

    const acknowledgement = await this.paymentChannels.account.acknowledge(unacknowledgedTicket, ackMsg.getHashedKey())

    if (acknowledgement === null) {
      log(`Got a ticket that is not a win. Dropping ticket.`)
      await deleteTicket(this.db, ackMsg.getHashedKey())
    } else {
      log(`Storing winning ticket`)
      await replaceTicketWithAcknowledgement(this.db, ackMsg.getHashedKey(), acknowledgement)
    }
    this.emit(u8aToHex(UnAcknowledgedTickets(ackMsg.getHashedKey().serialize())))
  }

  async handleHelper(source: AsyncIterable<Uint8Array>): Promise<void> {
    for await (const msg of source) {
      this.handleAcknowledgementMessage(AcknowledgementMessage.deserialize(msg))
    }
  }
}

export { PacketAcknowledgementInteraction }
