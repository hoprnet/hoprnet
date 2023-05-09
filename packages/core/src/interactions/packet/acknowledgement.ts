import {
  debug,
  pickVersion,
  type HoprDB,
  create_counter
} from '@hoprnet/hopr-utils'

import {
  Acknowledgement,
  PendingAcknowledgement,
  AcknowledgedTicket,
  HalfKeyChallenge,
  PublicKey,
  Hash
} from '../../../lib/core_types.js'

import type { SendMessage } from '../../index.js'
import type { PeerId } from '@libp2p/interface-peer-id'
import { ACKNOWLEDGEMENT_TIMEOUT } from '../../constants.js'
import { Packet } from '../../messages/index.js'
import { Pushable, pushable } from 'it-pushable'
import type { ResolvedEnvironment } from '../../environment.js'
import type { Components } from '@libp2p/interfaces/components'

const log = debug('hopr-core:acknowledgement')

type OnAcknowledgement = (halfKey: HalfKeyChallenge) => void
type OnAckedTicket = (ackMessage: AcknowledgedTicket) => void

type Incoming = [msg: Uint8Array, remotePeer: PeerId]
type Outgoing = [ack: Uint8Array, destination: PeerId]

// Do not type-check JSON files
// @ts-ignore
import pkg from '../../../package.json' assert { type: 'json' }

const NORMALIZED_VERSION = pickVersion(pkg.version)

// Metrics
const metric_receivedSuccessfulAcks = create_counter(
  'core_counter_received_successful_acks',
  'Number of received successful message acknowledgements'
)
const metric_receivedFailedAcks = create_counter(
  'core_counter_received_failed_acks',
  'Number of received failed message acknowledgements'
)
const metric_sentAcks = create_counter('core_counter_sent_acks', 'Number of sent message acknowledgements')

const metric_ackedTickets = create_counter('core_counter_acked_tickets', 'Number of acknowledged tickets')

const PREIMAGE_PLACE_HOLDER = new Hash(new Uint8Array(Hash.SIZE).fill(0xff))

export class AcknowledgementInteraction {
  private incomingAcks: Pushable<Incoming>
  private outgoingAcks: Pushable<Outgoing>

  public readonly protocols: string | string[]

  constructor(
    private sendMessage: SendMessage,
    private libp2pComponents: Components,
    private privKey: PeerId,
    private db: HoprDB,
    private onAcknowledgement: OnAcknowledgement,
    private onAckedTicket: OnAckedTicket,
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
    await this.libp2pComponents.getRegistrar().handle(this.protocols, async ({ connection, stream }) => {
      try {
        for await (const chunk of stream.source) {
          this.incomingAcks.push([chunk, connection.remotePeer])
        }
      } catch (err) {
        log(`Error while receiving acknowledgement`, err)
      }
    })

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
    metric_sentAcks.increment()
    this.outgoingAcks.push([ack.serialize(), destination])
  }

  /**
   * Reserve a preImage for the given ticket if it is a winning ticket.
   */
  async handleAcknowledgement(msg: Uint8Array, remotePeer: PeerId): Promise<void> {
    const acknowledgement = Acknowledgement.deserialize(msg)
    acknowledgement.validate(PublicKey.from_peerid_str(this.privKey.toString()), PublicKey.from_peerid_str(remotePeer.toString()))

    // There are three cases:
    // 1. There is an unacknowledged ticket and we are
    //    awaiting a half key.
    // 2. We were the creator of the packet, hence we
    //    do not wait for any half key
    // 3. The acknowledgement is unexpected and stems from
    //    a protocol bug or an attacker
    let pending: PendingAcknowledgement
    try {
      pending = await this.db.getPendingAcknowledgement(acknowledgement.ackChallenge)
    } catch (err: any) {
      // Protocol bug?
      if (err != undefined && err.notFound) {
        log(
          `Received unexpected acknowledgement for half key challenge ${acknowledgement.ack_challenge().to_hex()} - half key ${acknowledgement.ack_key_share.to_hex()}`
        )
      }
      metric_receivedFailedAcks.increment()
      throw err
    }

    // No pending ticket, nothing to do.
    if (pending.is_msg_sender()) {
      log(`Received acknowledgement as sender. First relayer has processed the packet.`)
      // Resolves `sendMessage()` promise
      this.onAcknowledgement(acknowledgement.ack_challenge())
      metric_receivedSuccessfulAcks.increment()
      // nothing else to do
      return
    }

    // Try to unlock our incentive
    const unacknowledged = pending.ticket()

    if (!unacknowledged.verify_challenge(acknowledgement.ack_key_share)) {
      metric_receivedFailedAcks.increment()
      throw Error(`The acknowledgement is not sufficient to solve the embedded challenge.`)
    }

    try {
      await this.db.getChannelFrom(unacknowledged.signer)
    } catch (e) {
      // We are acknowledging a ticket for a channel we do not think exists?
      // Also we know about the unacknowledged ticket? This should never happen.
      // Something clearly screwy here. This is bad enough to be a fatal error
      // we should kill the node and debug.
      log('Error, acknowledgement received for channel that does not exist')
      metric_receivedFailedAcks.increment()
      throw e
    }
    const response = unacknowledged.get_response(acknowledgement.ack_key_share)

    // Store the acknowledged ticket, regardless if it's a win or a loss
    // create an acked ticket with a pre image place holder
    const ack = new AcknowledgedTicket(unacknowledged.ticket, response, PREIMAGE_PLACE_HOLDER, unacknowledged.signer)
    log(`Acknowledging ticket. Using response ${response.toHex()}`)
    // replace the unAcked ticket with Acked ticket.

    try {
      await this.db.replaceUnAckWithAck(acknowledgement.ack_challenge(), ack)
      log(`Stored acknowledged ticket`)
    } catch (err) {
      log(`ERROR: cannot replace an UnAck ticket with Ack ticket, thus dropping ticket`, err)
    }

    metric_ackedTickets.increment()

    // If auto-ticket-redemption is on, onAckedTicket, try to redeem
    this.onAckedTicket(ack)
  }
}
