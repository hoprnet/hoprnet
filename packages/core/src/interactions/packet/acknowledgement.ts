import {
  debug,
  pickVersion,
  AcknowledgedTicket,
  type HoprDB,
  type PendingAckowledgement,
  type HalfKeyChallenge,
  type Hash
} from '@hoprnet/hopr-utils'
import { findCommitmentPreImage, bumpCommitment } from '@hoprnet/hopr-core-ethereum'
import type { SendMessage, Subscribe } from '../../index.js'
import type { PeerId } from '@libp2p/interface-peer-id'
import { ACKNOWLEDGEMENT_TIMEOUT } from '../../constants.js'
import { Acknowledgement, Packet } from '../../messages/index.js'
import { Pushable, pushable } from 'it-pushable'
import type { ResolvedEnvironment } from '../../environment.js'
const log = debug('hopr-core:acknowledgement')

type OnAcknowledgement = (halfKey: HalfKeyChallenge) => void
type OnWinningTicket = (ackMessage: AcknowledgedTicket) => void
type OnOutOfCommitments = (channelId: Hash) => void

type Incoming = [msg: Uint8Array, remotePeer: PeerId]
type Outgoing = [ack: Uint8Array, destination: PeerId]

// Do not type-check JSON files
// @ts-ignore
import pkg from '../../../package.json' assert { type: 'json' }

const NORMALIZED_VERSION = pickVersion(pkg.version)

export class AcknowledgementInteraction {
  private incomingAcks: Pushable<Incoming>
  private outgoingAcks: Pushable<Outgoing>

  public readonly protocols: string | string[]

  constructor(
    private sendMessage: SendMessage,
    private subscribe: Subscribe,
    private privKey: PeerId,
    private db: HoprDB,
    private onAcknowledgement: OnAcknowledgement,
    private onWinningTicket: OnWinningTicket,
    private onOutOfCommitments: OnOutOfCommitments,
    private environment: ResolvedEnvironment
  ) {
    this.incomingAcks = pushable<Incoming>({ objectMode: true })
    this.outgoingAcks = pushable<Outgoing>({ objectMode: true })

    this.protocols = [
      // current
      `/hopr/${this.environment.id}/ack/${NORMALIZED_VERSION}`,
      // deprecated
      `/hopr/${this.environment.id}/ack`
    ]

    this.handleAcknowledgement = this.handleAcknowledgement.bind(this)
  }
  async start() {
    await this.subscribe(
      this.protocols,
      (msg: Uint8Array, remotePeer: PeerId) => {
        this.incomingAcks.push([msg, remotePeer])
      },
      false,
      (err: any) => {
        log(`Error while receiving acknowledgement`, err)
      }
    )

    this.startHandleIncoming()
    this.startSendAcknowledgements()
  }

  stop() {
    // End the streams to avoid handing promises
    this.incomingAcks.end()
    this.outgoingAcks.end()
  }

  async startHandleIncoming() {
    for await (const incomingAck of this.incomingAcks) {
      await this.handleAcknowledgement(incomingAck[0], incomingAck[1])
    }
  }

  async startSendAcknowledgements() {
    for await (const outgoingAck of this.outgoingAcks) {
      try {
        await this.sendMessage(outgoingAck[1], this.protocols, outgoingAck[0], false, {
          timeout: ACKNOWLEDGEMENT_TIMEOUT
        })
      } catch (err) {
        // Currently unclear how to proceed if sending acknowledgements
        // fails
        log(`Error: could not send acknowledgement`, err)
      }
    }
  }
  sendAcknowledgement(packet: Packet, destination: PeerId): void {
    const ack = packet.createAcknowledgement(this.privKey)

    this.outgoingAcks.push([ack.serialize(), destination])
  }

  /**
   * Reserve a preImage for the given ticket if it is a winning ticket.
   */
  async handleAcknowledgement(msg: Uint8Array, remotePeer: PeerId): Promise<void> {
    const acknowledgement = Acknowledgement.deserialize(msg, this.privKey, remotePeer)

    // There are three cases:
    // 1. There is an unacknowledged ticket and we are
    //    awaiting a half key.
    // 2. We were the creator of the packet, hence we
    //    do not wait for any half key
    // 3. The acknowledgement is unexpected and stems from
    //    a protocol bug or an attacker
    let pending: PendingAckowledgement
    try {
      pending = await this.db.getPendingAcknowledgement(acknowledgement.ackChallenge)
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
      this.onAcknowledgement(acknowledgement.ackChallenge)
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
      channelId = (await this.db.getChannelFrom(unacknowledged.signer)).getId()
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
    let opening: Hash
    try {
      opening = await findCommitmentPreImage(this.db, channelId)
    } catch (err) {
      log(`Channel ${channelId.toHex()} is out of commitments`)
      this.onOutOfCommitments(channelId)
      // TODO: How should we handle this ticket?
      return
    }

    if (!ticket.isWinningTicket(opening, response, ticket.winProb)) {
      log(`Got a ticket that is not a win. Dropping ticket.`)
      await this.db.markLosing(unacknowledged)
      return
    }

    // Ticket is a win, let's store it
    const ack = new AcknowledgedTicket(ticket, response, opening, unacknowledged.signer)
    log(`Acknowledging ticket. Using opening ${opening.toHex()} and response ${response.toHex()}`)

    try {
      await this.db.replaceUnAckWithAck(acknowledgement.ackChallenge, ack)
      log(`Stored winning ticket`)
    } catch (err) {
      log(`ERROR: commitment could not be bumped, thus dropping ticket`, err)
    }

    // store commitment in db
    await bumpCommitment(this.db, channelId, opening)

    this.onWinningTicket(ack)
  }
}
