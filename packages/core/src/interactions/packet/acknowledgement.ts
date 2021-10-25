import { debug } from '@hoprnet/hopr-utils'
import { HalfKey, durations, oneAtATime, AcknowledgedTicket, UnacknowledgedTicket } from '@hoprnet/hopr-utils'
import PeerId from 'peer-id'
import { findCommitmentPreImage, bumpCommitment } from '@hoprnet/hopr-core-ethereum'
import { PROTOCOL_ACKNOWLEDGEMENT } from '../../constants'
import { Acknowledgement, Packet } from '../../messages'
import { HoprDB } from '@hoprnet/hopr-utils'
import { EventEmitter } from 'events'
const log = debug('hopr-core:acknowledgement')

const ACKNOWLEDGEMENT_TIMEOUT = durations.seconds(2)

/**
 * Reserve a preImage for the given ticket if it is a winning ticket.
 * @param ticket the acknowledged ticket
 */
async function acknowledge(
  unacknowledgedTicket: UnacknowledgedTicket,
  acknowledgement: HalfKey,
  db: HoprDB,
  events: EventEmitter
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
      events.emit('ticket:win', ack)
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
  subscribe: any,
  db: HoprDB,
  events: EventEmitter,
  pubKey: PeerId,
  onMessage: (ackMessage: Acknowledgement) => void
) {
  async function handleAcknowledgement(msg: Uint8Array, remotePeer: PeerId) {
    const ackMsg = Acknowledgement.deserialize(msg, pubKey, remotePeer)

    try {
      let unacknowledgedTicket = await db.getUnacknowledgedTicket(ackMsg.ackChallenge)
      const ackedTicket = await acknowledge(unacknowledgedTicket, ackMsg.ackKeyShare, db, events)
      if (ackedTicket) {
        log(`Storing winning ticket`)
        await db.replaceUnAckWithAck(ackMsg.ackChallenge, ackedTicket)
      }
    } catch (err) {
      if (!err.notFound) {
        throw err
      }
    }
    onMessage(ackMsg)
  }

  const limitConcurrency = oneAtATime()
  subscribe(PROTOCOL_ACKNOWLEDGEMENT, (msg: Uint8Array, remotePeer: PeerId) =>
    limitConcurrency(() => handleAcknowledgement(msg, remotePeer))
  )
}

export function sendAcknowledgement(packet: Packet, destination: PeerId, sendMessage: any, privKey: PeerId): void {
  setImmediate(async () => {
    const ack = packet.createAcknowledgement(privKey)

    sendMessage(destination, PROTOCOL_ACKNOWLEDGEMENT, ack.serialize(), {
      timeout: ACKNOWLEDGEMENT_TIMEOUT
    })
  })
}
