import debug from 'debug'
import { AcknowledgementMessage } from '../../messages/acknowledgement'
import { PROTOCOL_ACKNOWLEDGEMENT } from '../../constants'
import { HoprDB } from '@hoprnet/hopr-utils'
const log = debug('hopr-core:acknowledgement')

export function subscribeToAcknowledgements(
  subscribe: any,
  db: HoprDB,
  paymentChannels: any,
  onMessage: (ackMessage: AcknowledgementMessage) => void
) {
  subscribe(PROTOCOL_ACKNOWLEDGEMENT, async function (msg: Uint8Array) {
    const ackMsg = AcknowledgementMessage.deserialize(msg)
    let unacknowledgedTicket = await db.getUnacknowledgedTicketsByKey(ackMsg.getHashedKey())
    if (!unacknowledgedTicket) {
      // Could be dummy, could be error.
      log('dropping unknown ticket')
      return await db.deleteTicket(ackMsg.getHashedKey())
    }

    const acknowledgement = await paymentChannels.account.acknowledge(unacknowledgedTicket, ackMsg.getHashedKey())

    if (acknowledgement === null) {
      log(`Got a ticket that is not a win. Dropping ticket.`)
      await db.deleteTicket(ackMsg.getHashedKey())
    } else {
      log(`Storing winning ticket`)
      await db.replaceTicketWithAcknowledgement(ackMsg.getHashedKey(), acknowledgement)
    }
    onMessage(ackMsg)
  })
}
