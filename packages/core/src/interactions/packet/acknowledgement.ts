import { durations, oneAtATime, debug, AcknowledgedTicket } from '@hoprnet/hopr-utils'
import type { HalfKey, UnacknowledgedTicket } from '@hoprnet/hopr-utils'
import { findCommitmentPreImage, bumpCommitment } from '@hoprnet/hopr-core-ethereum'

import type { SendMessage, Subscribe } from '../../index'
import type PeerId from 'peer-id'
import { PROTOCOL_ACKNOWLEDGEMENT } from '../../constants'
import { Acknowledgement, Packet } from '../../messages'
import { HoprDB } from '@hoprnet/hopr-utils'
const log = debug('hopr-core:acknowledgement')
const error = debug('hopr-core:acknowledgement:error')

const ACKNOWLEDGEMENT_TIMEOUT = durations.seconds(2)

/**
 * Reserve a preImage for the given ticket if it is a winning ticket.
 * @param ticket the acknowledged ticket
 */
async function acknowledge(
  unacknowledgedTicket: UnacknowledgedTicket,
  acknowledgement: HalfKey,
  db: HoprDB
): Promise<AcknowledgedTicket | null> {
  if (!unacknowledgedTicket.verifyChallenge(acknowledgement)) {
    throw Error(`The acknowledgement is not sufficient to solve the embedded challenge.`)
  }

  const channelId = (await db.getChannelFrom(unacknowledgedTicket.signer)).getId()
  const response = unacknowledgedTicket.getResponse(acknowledgement)

  const ticket = unacknowledgedTicket.ticket
  const opening = await findCommitmentPreImage(db, channelId)

  if (ticket.isWinningTicket(opening, response, ticket.winProb)) {
    const ack = new AcknowledgedTicket(ticket, response, opening, unacknowledgedTicket.signer)

    log(`Acknowledging ticket. Using opening ${opening.toHex()} and response ${response.toHex()}`)

    try {
      await bumpCommitment(db, channelId)
      return ack
    } catch (e) {
      log(`ERROR: commitment could not be bumped ${e}, thus dropping ticket`)
      return null
    }
  } else {
    log(`Got a ticket that is not a win. Dropping ticket.`)
    await db.markLosing(unacknowledgedTicket)
    return null
  }
}

export function subscribeToAcknowledgements(
  subscribe: Subscribe,
  db: HoprDB,
  pubKey: PeerId,
  onMessage: (ackMessage: Acknowledgement) => void
) {
  async function handleAcknowledgement(msg: Uint8Array, remotePeer: PeerId) {
    const ackMsg = Acknowledgement.deserialize(msg, pubKey, remotePeer)

    let unacknowledgedTicket = await db.getUnacknowledgedTicket(ackMsg.ackChallenge)
    const ackedTicket = await acknowledge(unacknowledgedTicket, ackMsg.ackKeyShare, db)
    if (ackedTicket) {
      log(`Storing winning ticket`)
      await db.replaceUnAckWithAck(ackMsg.ackChallenge, ackedTicket)
    }
    onMessage(ackMsg)
  }

  const limitConcurrency = oneAtATime()
  subscribe(
    PROTOCOL_ACKNOWLEDGEMENT,
    (msg: Uint8Array, remotePeer: PeerId) => limitConcurrency(() => handleAcknowledgement(msg, remotePeer)),
    false,
    (err: any) => {
      error(`Error while receiving acknowledgement`, err)
    }
  )
}

export function sendAcknowledgement(
  packet: Packet,
  destination: PeerId,
  sendMessage: SendMessage,
  privKey: PeerId
): void {
  ;(async () => {
    const ack = packet.createAcknowledgement(privKey)

    try {
      await sendMessage(destination, PROTOCOL_ACKNOWLEDGEMENT, ack.serialize(), false, {
        timeout: ACKNOWLEDGEMENT_TIMEOUT
      })
    } catch (err) {
      // Currently unclear how to proceed if sending acknowledgements
      // fails
      error(`could not send acknowledgement`, err)
    }
  })()
}
