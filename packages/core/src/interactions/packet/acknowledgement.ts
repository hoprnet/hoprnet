import { durations, oneAtATime, debug, AcknowledgedTicket } from '@hoprnet/hopr-utils'
import type { UnacknowledgedTicket } from '@hoprnet/hopr-utils'
import { findCommitmentPreImage, bumpCommitment } from '@hoprnet/hopr-core-ethereum'

import type { SendMessage, Subscribe } from '../../index'
import type PeerId from 'peer-id'
import { PROTOCOL_ACKNOWLEDGEMENT } from '../../constants'
import { Acknowledgement, Packet } from '../../messages'
import { HoprDB } from '@hoprnet/hopr-utils'
const log = debug('hopr-core:acknowledgement')
const error = debug('hopr-core:acknowledgement:error')

const ACKNOWLEDGEMENT_TIMEOUT = durations.seconds(2)

async function acknowledge(
  unacknowledgedTicket: UnacknowledgedTicket,
  acknowledgement: Acknowledgement,
  db: HoprDB
): Promise<AcknowledgedTicket | null> {
  if (!unacknowledgedTicket.verifyChallenge(acknowledgement.ackKeyShare)) {
    throw Error(`The acknowledgement is not sufficient to solve the embedded challenge.`)
  }

  const channelId = (await db.getChannelFrom(unacknowledgedTicket.signer)).getId()
  const response = unacknowledgedTicket.getResponse(acknowledgement.ackKeyShare)

  const ticket = unacknowledgedTicket.ticket
  const opening = await findCommitmentPreImage(db, channelId)

  if (ticket.isWinningTicket(opening, response, ticket.winProb)) {
    const ack = new AcknowledgedTicket(ticket, response, opening, unacknowledgedTicket.signer)

    log(`Acknowledging ticket. Using opening ${opening.toHex()} and response ${response.toHex()}`)

    try {
      await bumpCommitment(db, channelId)
      await db.replaceUnAckWithAck(acknowledgement.ackChallenge, ack)
      log(`Stored winning ticket`)
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

/**
 * Reserve a preImage for the given ticket if it is a winning ticket.
 */
async function handleAcknowledgement(
  msg: Uint8Array,
  remotePeer: PeerId,
  pubKey: PeerId,
  db: HoprDB,
  onMessage: (ackMessage: Acknowledgement) => void
) {
  const acknowledgement = Acknowledgement.deserialize(msg, pubKey, remotePeer)
  const unacknowledgedTicket = await db.getUnacknowledgedTicket(acknowledgement.ackChallenge)
  await acknowledge(unacknowledgedTicket, acknowledgement, db)
  onMessage(acknowledgement)
}

export function subscribeToAcknowledgements(
  subscribe: Subscribe,
  db: HoprDB,
  pubKey: PeerId,
  onMessage: (ackMessage: Acknowledgement) => void
) {

  const limitConcurrency = oneAtATime()
  subscribe(
    PROTOCOL_ACKNOWLEDGEMENT,
    (msg: Uint8Array, remotePeer: PeerId) => limitConcurrency(() => handleAcknowledgement(msg, remotePeer, pubKey, db, onMessage)),
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
