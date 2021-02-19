import { AbstractInteraction } from '../abstractInteraction'
import type { Connection, MuxedStream } from 'libp2p'

import pipe from 'it-pipe'
import PeerId from 'peer-id'

import debug from 'debug'
const log = debug('hopr-core:acknowledgement')
const error = debug('hopr-core:acknowledgement:error')

import { green, red, blue, yellow } from 'chalk'

import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '../../'
import { Acknowledgement } from '../../messages/acknowledgement'

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

class PacketAcknowledgementInteraction<Chain extends HoprCoreConnector>
  extends EventEmitter
  implements AbstractInteraction {
  protocols: string[] = [PROTOCOL_ACKNOWLEDGEMENT]

  constructor(public node: Hopr<Chain>) {
    super()
    this.node._libp2p.handle(this.protocols, this.handler.bind(this))
  }

  handler(struct: { connection: Connection; stream: MuxedStream; protocol: string }) {
    pipe(struct.stream, this.handleHelper.bind(this))
  }

  async interact(counterparty: PeerId, acknowledgement: Acknowledgement<Chain>): Promise<void> {
    const struct = await dialHelper(this.node._libp2p, counterparty, this.protocols, {
      timeout: ACKNOWLEDGEMENT_TIMEOUT
    })

    if (struct == undefined) {
      error(`Could not send acknowledgement to party ${counterparty.toB58String()}.`)
    }

    pipe([acknowledgement], struct.stream)
  }

  async handleHelper(source: AsyncIterable<Uint8Array>): Promise<void> {
    for await (const msg of source) {
      const arr = msg.slice()
      const acknowledgement = new Acknowledgement(this.node.paymentChannels, {
        bytes: arr.buffer,
        offset: arr.byteOffset
      })

      const unAcknowledgedDbKey = this.node._dbKeys.UnAcknowledgedTickets(await acknowledgement.hashedKey)

      let tmp: Uint8Array
      try {
        tmp = await this.node.db.get(Buffer.from(unAcknowledgedDbKey))
      } catch (err) {
        if (err.notFound) {
          error(
            `received unknown acknowledgement from party ${blue(
              (await pubKeyToPeerId(await acknowledgement.responseSigningParty)).toB58String()
            )} for challenge ${yellow(u8aToHex(await acknowledgement.hashedKey))} - response was ${green(
              u8aToHex(await acknowledgement.hashedKey)
            )}. ${red('Dropping acknowledgement')}.`
          )

          continue
        } else {
          throw err
        }
      }

      if (tmp.length > 0) {
        const unacknowledgedTicket = new UnacknowledgedTicket(this.node.paymentChannels, {
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

        const resp = await this.node.paymentChannels.validateTicket(
          await unacknowledgedTicket.signedTicket,
          await this.node.paymentChannels.utils.hash(
            u8aConcat(unacknowledgedTicket.secretA, await acknowledgement.hashedKey)
          )
        )
        if (resp.status === 'SUCCESS') {
          const acknowledgedDbKey = this.node._dbKeys.AcknowledgedTickets(ticketCounter)

          log(
            `Storing ticket #${u8aToNumber(ticketCounter)} from ${blue(
              (await pubKeyToPeerId(await acknowledgement.responseSigningParty)).toB58String()
            )}. Ticket contains preImage for ${green(u8aToHex(await acknowledgement.hashedKey))}`
          )

          try {
            await this.node.db
              .batch()
              .del(Buffer.from(unAcknowledgedDbKey))
              .put(Buffer.from(acknowledgedDbKey), Buffer.from(resp.ticket.serialize()))
              .put(Buffer.from(this.node._dbKeys.AcknowledgedTicketCounter()), Buffer.from(ticketCounter))
              .write()
          } catch (err) {
            error(`Error while writing to database. Error was ${red(err.message)}.`)
          }
        } else {
          log('Bad ticket:' + resp.status)
          await this.node.db.del(Buffer.from(unAcknowledgedDbKey))
        }

        this.emit(u8aToHex(unAcknowledgedDbKey))
      
      }
    }
  }
}

export { PacketAcknowledgementInteraction }
