import type { LevelUp } from 'levelup'
import debug from 'debug'
import { Acknowledgement } from '../../messages/acknowledgement'
import { getUnacknowledgedTickets, deleteTicket, replaceTicketWithAcknowledgement } from '../../dbKeys'
import { PROTOCOL_ACKNOWLEDGEMENT } from '../../constants'
import PeerId from 'peer-id'
const log = debug('hopr-core:acknowledgement')

export function subscribeToAcknowledgements(
  subscribe: any,
  db: LevelUp,
  paymentChannels: any,
  pubKey: PeerId,
  onMessage: (ackMessage: Acknowledgement) => void
) {
  subscribe(PROTOCOL_ACKNOWLEDGEMENT, async function (msg: Uint8Array, remotePeer: PeerId) {
    const ackMsg = Acknowledgement.deserialize(msg, pubKey, remotePeer)
    let unacknowledgedTicket = await getUnacknowledgedTickets(db, ackMsg.ackChallenge)

    if (!unacknowledgedTicket) {
      // Could be dummy, could be error.
      log('dropping unknown ticket')
      return await deleteTicket(db, ackMsg.ackChallenge)
    }

    const acknowledgement = await paymentChannels.account.acknowledge(unacknowledgedTicket, ackMsg.ackChallenge)

    if (acknowledgement === null) {
      log(`Got a ticket that is not a win. Dropping ticket.`)
      await deleteTicket(db, ackMsg.ackChallenge)
    } else {
      log(`Storing winning ticket`)
      await replaceTicketWithAcknowledgement(db, ackMsg.ackChallenge, acknowledgement)
    }
    onMessage(ackMsg)
  })
}
