import type { LevelUp } from 'levelup'
import debug from 'debug'
import { AcknowledgementMessage } from '../../messages/acknowledgement'
import {
  getUnacknowledgedTickets,
  deleteTicket,
  replaceTicketWithAcknowledgement,
} from '../../dbKeys'
import { PROTOCOL_ACKNOWLEDGEMENT } from '../../constants'
const log = debug('hopr-core:acknowledgement')

export function subscribeToAcknowledgements(
  subscribe: any,
  db: LevelUp,
  paymentChannels: any,
  onMessage: (ackMessage: AcknowledgementMessage) => void
) {
  subscribe(PROTOCOL_ACKNOWLEDGEMENT, async function(msg: Uint8Array){
    const ackMsg = AcknowledgementMessage.deserialize(msg)
    let unacknowledgedTicket = await getUnacknowledgedTickets(db, ackMsg.getHashedKey())
    if (!unacknowledgedTicket) {
      // Could be dummy, could be error.
      log('dropping unknown ticket')
      return await deleteTicket(db, ackMsg.getHashedKey())
    }

    const acknowledgement = await paymentChannels.account.acknowledge(unacknowledgedTicket, ackMsg.getHashedKey())

    if (acknowledgement === null) {
      log(`Got a ticket that is not a win. Dropping ticket.`)
      await deleteTicket(db, ackMsg.getHashedKey())
    } else {
      log(`Storing winning ticket`)
      await replaceTicketWithAcknowledgement(db, ackMsg.getHashedKey(), acknowledgement)
    }
    onMessage(ackMsg)
  })
}
