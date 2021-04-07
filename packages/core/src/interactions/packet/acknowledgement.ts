import { AbstractInteraction } from '../abstractInteraction'
import type { Connection, MuxedStream } from 'libp2p'

import pipe from 'it-pipe'
import PeerId from 'peer-id'

import debug from 'debug'
const log = debug('hopr-core:acknowledgement')
import { green, red, blue, yellow } from 'chalk'
import type Hopr from '../../'
import { AcknowledgementMessage } from '../../messages/acknowledgement'
import { Hash } from '@hoprnet/hopr-core-ethereum'

import EventEmitter from 'events'

import { PROTOCOL_ACKNOWLEDGEMENT } from '../../constants'
import {
  dialHelper,
  u8aToHex,
  durations,
  u8aConcat,
  toU8a,
  u8aToNumber,
  pubKeyToPeerId,
  u8aAdd
} from '@hoprnet/hopr-utils'
import { UnacknowledgedTicket } from '../../messages/ticket'

import { ACKNOWLEDGED_TICKET_INDEX_LENGTH } from '../../dbKeys'

const ONE = toU8a(1, 8)

const ACKNOWLEDGEMENT_TIMEOUT = durations.seconds(2)

class PacketAcknowledgementInteraction extends EventEmitter implements AbstractInteraction {
  protocols: string[] = [PROTOCOL_ACKNOWLEDGEMENT]

  constructor(public node: Hopr) {
    super()
    this.node._libp2p.handle(this.protocols, this.handler.bind(this))
  }

  handler(struct: { connection: Connection; stream: MuxedStream; protocol: string }) {
    pipe(struct.stream, this.handleHelper.bind(this))
  }

  async interact(counterparty: PeerId, acknowledgementMsg: AcknowledgementMessage): Promise<void> {
    const struct = await dialHelper(this.node._libp2p, counterparty, this.protocols[0], {
      timeout: ACKNOWLEDGEMENT_TIMEOUT
    })

    if (struct == undefined) {
      log(`ERROR: Could not send acknowledgement to party ${counterparty.toB58String()}.`)
    }

    pipe([acknowledgementMsg], struct.stream)
  }

  async handleAcknowledgementMessage(ackMsg: AcknowledgementMessage) {
    const unAcknowledgedDbKey = this.node._dbKeys.UnAcknowledgedTickets(ackMsg.getHashedKey().serialize())

    let tmp: Uint8Array
    try {
      tmp = await this.node.db.get(Buffer.from(unAcknowledgedDbKey))
    } catch (err) {
      if (err.notFound) {
        log(
          `ERROR: received unknown acknowledgement from party ${blue(
            (await pubKeyToPeerId(await ackMsg.responseSigningParty)).toB58String()
          )} for ${yellow(ackMsg.getHashedKey().toHex())}. ${red('Dropping acknowledgement')}.`
        )
        return
      } else {
        throw err
      }
    }

    if (tmp.length == 0) {
      // Deleting dummy DB entry
      await this.node.db.del(Buffer.from(unAcknowledgedDbKey))
      this.emit(u8aToHex(unAcknowledgedDbKey))
      return
    }

    const unacknowledgedTicket = new UnacknowledgedTicket({
      bytes: tmp.buffer,
      offset: tmp.byteOffset
    })

    let ticketCounter: Uint8Array
    try {
      let tmpTicketCounter = await this.node.db.get(Buffer.from(this.node._dbKeys.AcknowledgedTicketCounter()))

      ticketCounter = u8aAdd(true, tmpTicketCounter, ONE)
    } catch (err) {
      // Set ticketCounter to initial value
      ticketCounter = toU8a(0, ACKNOWLEDGED_TICKET_INDEX_LENGTH)
    }

    const ticket = await unacknowledgedTicket.signedTicket
    const response = Hash.create(
      u8aConcat(unacknowledgedTicket.secretA.serialize(), ackMsg.getHashedKey().serialize())
    )
    const acknowledgement = await this.node.paymentChannels.account.acknowledge(ticket, response)
    if (acknowledgement === null) {
      log(`Got a ticket that is not a win. Dropping ticket.`)
      await this.node.db.del(Buffer.from(unAcknowledgedDbKey))
      // TODO store stats
      return
    }

    const acknowledgedDbKey = this.node._dbKeys.AcknowledgedTickets(ticketCounter)

    log(
      `Storing ticket #${u8aToNumber(ticketCounter)} from ${blue(
        (await pubKeyToPeerId(await ackMsg.responseSigningParty)).toB58String()
      )}. Ticket contains preImage for ${green(ackMsg.getHashedKey().toHex())}`
    )

    try {
      await this.node.db
        .batch()
        .del(Buffer.from(unAcknowledgedDbKey))
        .put(Buffer.from(acknowledgedDbKey), Buffer.from(acknowledgement.serialize()))
        .put(Buffer.from(this.node._dbKeys.AcknowledgedTicketCounter()), Buffer.from(ticketCounter))
        .write()
    } catch (err) {
      log(`ERROR: Error while writing to database. Error was ${red(err.message)}.`)
    }
    this.emit(u8aToHex(unAcknowledgedDbKey))
  }

  async handleHelper(source: AsyncIterable<Uint8Array>): Promise<void> {
    for await (const msg of source) {
      this.handleAcknowledgementMessage(AcknowledgementMessage.deserialize(msg))
    }
  }
}

export { PacketAcknowledgementInteraction }
