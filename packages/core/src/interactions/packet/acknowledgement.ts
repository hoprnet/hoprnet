import { oneAtATime, debug, AcknowledgedTicket, HoprDB } from '@hoprnet/hopr-utils'
import type { PendingAckowledgement, HalfKeyChallenge, Hash } from '@hoprnet/hopr-utils'
import { findCommitmentPreImage, bumpCommitment } from '@hoprnet/hopr-core-ethereum'
import type { SendMessage, Subscribe } from '../../index'
import type PeerId from 'peer-id'
import { ACKNOWLEDGEMENT_TIMEOUT } from '../../constants'
import { Acknowledgement, Packet } from '../../messages'
const log = debug('hopr-core:acknowledgement')

type OnAcknowledgement = (halfKey: HalfKeyChallenge) => void
type OnWinningTicket = (ackMessage: AcknowledgedTicket) => void
type OnOutOfCommitments = (channelId: Hash) => void

/**
 * Reserve a preImage for the given ticket if it is a winning ticket.
 */
async function handleAcknowledgement(
  msg: Uint8Array,
  remotePeer: PeerId,
  pubKey: PeerId,
  db: HoprDB,
  onAcknowledgement: OnAcknowledgement,
  onWinningTicket: OnWinningTicket,
  onOutOfCommitments: OnOutOfCommitments
): Promise<void> {
  const acknowledgement = Acknowledgement.deserialize(msg, pubKey, remotePeer)

  // There are three cases:
  // 1. There is an unacknowledged ticket and we are
  //    awaiting a half key.
  // 2. We were the creator of the packet, hence we
  //    do not wait for any half key
  // 3. The acknowledgement is unexpected and stems from
  //    a protocol bug or an attacker
  let pending: PendingAckowledgement
  try {
    pending = await db.getPendingAcknowledgement(acknowledgement.ackChallenge)
  } catch (err) {
    // Protocol bug?
    if (err.notFound) {
      log(
        `Received unexpected acknowledgement for half key challenge ${acknowledgement.ackChallenge.toHex()} - half key ${acknowledgement.ackKeyShare.toHex()}`
      )
    }
    throw err
  }

  // No pending ticket, nothing to do.
  if (pending.isMessageSender == true) {
    log(`Received acknowledgement as sender. First relayer has processed the packet.`)
    // Resolves `sendMessage()` promise
    onAcknowledgement(acknowledgement.ackChallenge)
    // nothing else to do
    return
  }

  // Try to unlock our incentive
  const unacknowledged = pending.ticket

  if (!unacknowledged.verifyChallenge(acknowledgement.ackKeyShare)) {
    throw Error(`The acknowledgement is not sufficient to solve the embedded challenge.`)
  }

  let channelId: Hash
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
  let opening
  try {
    opening = await findCommitmentPreImage(db, channelId)
  } catch (err) {
    log(`Channel ${channelId.toHex()} is out of commitments`)
    onOutOfCommitments(channelId)
    // TODO: How should we handle this ticket?
    return
  }

  if (!ticket.isWinningTicket(opening, response, ticket.winProb)) {
    log(`Got a ticket that is not a win. Dropping ticket.`)
    await db.markLosing(unacknowledged)
    return
  }

  // Ticket is a win, let's store it
  const ack = new AcknowledgedTicket(ticket, response, opening, unacknowledged.signer)
  log(`Acknowledging ticket. Using opening ${opening.toHex()} and response ${response.toHex()}`)

  try {
    await db.replaceUnAckWithAck(acknowledgement.ackChallenge, ack)
    log(`Stored winning ticket`)
  } catch (err) {
    log(`ERROR: commitment could not be bumped, thus dropping ticket`, err)
  }

  // store commitment in db
  await bumpCommitment(db, channelId, opening)

  onWinningTicket(ack)
}

export function subscribeToAcknowledgements(
  subscribe: Subscribe,
  db: HoprDB,
  pubKey: PeerId,
  onAcknowledgement: OnAcknowledgement,
  onWinningTicket: OnWinningTicket,
  onOutOfCommitments: OnOutOfCommitments,
  protocolAck: string
) {
  const limitConcurrency = oneAtATime<void>()
  await subscribe(
    protocolAck,
    (msg: Uint8Array, remotePeer: PeerId) =>
      limitConcurrency(
        (): Promise<void> =>
          handleAcknowledgement(msg, remotePeer, pubKey, db, onAcknowledgement, onWinningTicket, onOutOfCommitments)
      ),
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
  privKey: PeerId,
  protocolAck: string
): void {
  ;(async () => {
    const ack = packet.createAcknowledgement(privKey)

    try {
      await sendMessage(destination, protocolAck, ack.serialize(), false, {
        timeout: ACKNOWLEDGEMENT_TIMEOUT
      })
    } catch (err) {
      // Currently unclear how to proceed if sending acknowledgements
      // fails
      log(`Error: could not send acknowledgement`, err)
    }
  })()
}
