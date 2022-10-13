import { oneAtATime, debug, AcknowledgedTicket, HoprDB, create_counter } from '@hoprnet/hopr-utils'
import type { PendingAckowledgement, HalfKeyChallenge, Hash } from '@hoprnet/hopr-utils'
import { findCommitmentPreImage, bumpCommitment } from '@hoprnet/hopr-core-ethereum'
import type { SendMessage, Subscribe } from '../../index.js'
import type { PeerId } from '@libp2p/interface-peer-id'
import { ACKNOWLEDGEMENT_TIMEOUT } from '../../constants.js'
import { Acknowledgement, Packet } from '../../messages/index.js'
const log = debug('hopr-core:acknowledgement')

type OnAcknowledgement = (halfKey: HalfKeyChallenge) => void
type OnWinningTicket = (ackMessage: AcknowledgedTicket) => void
type OnOutOfCommitments = (channelId: Hash) => void

// Metrics
const metric_receivedSuccessfulAcks = create_counter(
  'core_counter_received_successful_acks',
  'Number of received successful message acknowledgements'
)
const metric_receivedFailedAcks = create_counter(
  'core_counter_received_failed_acks',
  'Number of received successful message acknowledgements'
)
const metric_sentAcks = create_counter('core_counter_sent_acks', 'Number of sent message acknowledgements')

const metric_winningTickets = create_counter('core_counter_winning_tickets', 'Number of winning tickets')
const metric_losingTickets = create_counter('core_counter_losing_tickets', 'Number of losing tickets')

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
    metric_receivedFailedAcks.increment()
    throw err
  }

  // No pending ticket, nothing to do.
  if (pending.isMessageSender == true) {
    log(`Received acknowledgement as sender. First relayer has processed the packet.`)
    // Resolves `sendMessage()` promise
    onAcknowledgement(acknowledgement.ackChallenge)
    metric_receivedSuccessfulAcks.increment()
    // nothing else to do
    return
  }

  // Try to unlock our incentive
  const unacknowledged = pending.ticket

  if (!unacknowledged.verifyChallenge(acknowledgement.ackKeyShare)) {
    metric_receivedFailedAcks.increment()
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
    metric_receivedFailedAcks.increment()
    throw e
  }
  const response = unacknowledged.getResponse(acknowledgement.ackKeyShare)
  const ticket = unacknowledged.ticket
  let opening: Hash
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
    metric_losingTickets.increment()
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

  metric_winningTickets.increment()
  onWinningTicket(ack)
}

export async function subscribeToAcknowledgements(
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

export async function sendAcknowledgement(
  packet: Packet,
  destination: PeerId,
  sendMessage: SendMessage,
  privKey: PeerId,
  protocolAck: string
): Promise<void> {
  const ack = packet.createAcknowledgement(privKey)

  try {
    await sendMessage(destination, protocolAck, ack.serialize(), false, {
      timeout: ACKNOWLEDGEMENT_TIMEOUT
    })
    metric_sentAcks.increment()
  } catch (err) {
    // Currently unclear how to proceed if sending acknowledgements
    // fails
    log(`Error: could not send acknowledgement`, err)
  }
}
