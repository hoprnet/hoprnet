import { AcknowledgementMessage } from '../../messages/acknowledgement'
import { Logger } from '@hoprnet/hopr-utils'
import { PROTOCOL_ACKNOWLEDGEMENT } from '../../constants'
import { HoprDB } from '@hoprnet/hopr-utils'
const log = Logger.getLogger('hopr-core.acknowledgement')

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
      log.info('Dropping unknown ticket')
      return await db.deleteTicket(ackMsg.getHashedKey())
    }

    const acknowledgement = await paymentChannels.account.acknowledge(unacknowledgedTicket, ackMsg.getHashedKey())

    if (acknowledgement === null) {
      log.info(`Got a ticket that is not a win. Dropping ticket.`)
      await db.deleteTicket(ackMsg.getHashedKey())
    } else {
      log.info(`Storing winning ticket`)
      await db.replaceTicketWithAcknowledgement(ackMsg.getHashedKey(), acknowledgement)
    }
    onMessage(ackMsg)
  })
}
