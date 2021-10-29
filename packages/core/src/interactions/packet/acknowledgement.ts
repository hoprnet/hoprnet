import { oneAtATime, debug, AcknowledgedTicket, HoprDB } from '@hoprnet/hopr-utils'
import type { UnacknowledgedTicket, WaitingAsRelayer } from '@hoprnet/hopr-utils'
import { findCommitmentPreImage, bumpCommitment } from '@hoprnet/hopr-core-ethereum'
import type { SendMessage, Subscribe } from '../../index'
import type PeerId from 'peer-id'
import { PROTOCOL_ACKNOWLEDGEMENT, ACKNOWLEDGEMENT_TIMEOUT } from '../../constants'
import { Acknowledgement, Packet } from '../../messages'
const log = debug('hopr-core:acknowledgement')

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

  let unacknowledged: UnacknowledgedTicket
  try {
    const pending = (await db.getPendingAcknowledgement(acknowledgement.ackChallenge)) as WaitingAsRelayer
    unacknowledged = pending.ticket
  } catch (err) {
    if (err.notFound) {
      log(
        `Received unexpected acknowledgement for half key challenge ${acknowledgement.ackChallenge.toHex()} - half key ${acknowledgement.ackKeyShare.toHex()}`
      )
    }
    throw err
  }

  if (!unacknowledged.verifyChallenge(acknowledgement.ackKeyShare)) {
    throw Error(`The acknowledgement is not sufficient to solve the embedded challenge.`)
  }

  let channelId
  try {
    channelId = (await db.getChannelFrom(unacknowledged.signer)).getId()
  } catch (e) {
    // We are acknowledging a ticket for a channel we do not think exists?
    // Also we know about the unacknowledged ticket? This should never happen.
    // Something clearly screwy here. This is bad enough to be a fatal error
    // we should kill the node and debug.
    log('Error, acknowledgement received for channel that does not exist')
    throw e
  }
  const response = unacknowledged.getResponse(acknowledgement.ackKeyShare)
  const ticket = unacknowledged.ticket
  const opening = await findCommitmentPreImage(db, channelId)

  if (ticket.isWinningTicket(opening, response, ticket.winProb)) {
    const ack = new AcknowledgedTicket(ticket, response, opening, unacknowledged.signer)
    log(`Acknowledging ticket. Using opening ${opening.toHex()} and response ${response.toHex()}`)
    try {
      await bumpCommitment(db, channelId)
      await db.replaceUnAckWithAck(acknowledgement.ackChallenge, ack)
      log(`Stored winning ticket`)
    } catch (e) {
      log(`ERROR: commitment could not be bumped ${e}, thus dropping ticket`)
    }
  } else {
    log(`Got a ticket that is not a win. Dropping ticket.`)
    await db.markLosing(unacknowledged)
  }
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
    (msg: Uint8Array, remotePeer: PeerId) =>
      limitConcurrency(() => handleAcknowledgement(msg, remotePeer, pubKey, db, onMessage)),
    false,
    (err: any) => {
      log(`Error while receiving acknowledgement`, err)
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
      log(`Error: could not send acknowledgement`, err)
    }
  })()
}
