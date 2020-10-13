import { AbstractInteraction } from '../abstractInteraction'

import pipe from 'it-pipe'
import PeerId from 'peer-id'

import AbortController from 'abort-controller'

import debug from 'debug'
const log = debug('hopr-core:acknowledgement')
const error = debug('hopr-core:acknowledgement:error')

import chalk from 'chalk'

import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '../../'
import { Acknowledgement } from '../../messages/acknowledgement'

import type { Handler } from '../../@types/transport'

import EventEmitter from 'events'

import { PROTOCOL_ACKNOWLEDGEMENT } from '../../constants'
import { u8aToHex, durations, u8aConcat, toU8a, u8aToNumber } from '@hoprnet/hopr-utils'
import { pubKeyToPeerId } from '../../utils'
import { UnacknowledgedTicket } from '../../messages/ticket'

import { ACKNOWLEDGED_TICKET_INDEX_LENGTH } from '../../dbKeys'

const ACKNOWLEDGEMENT_TIMEOUT = durations.seconds(2)

class PacketAcknowledgementInteraction<Chain extends HoprCoreConnector>
  extends EventEmitter
  implements AbstractInteraction<Chain> {
  protocols: string[] = [PROTOCOL_ACKNOWLEDGEMENT]

  constructor(public node: Hopr<Chain>) {
    super()
    this.node.handle(this.protocols, this.handler.bind(this))
  }

  handler(struct: Handler) {
    pipe(struct.stream, this.handleHelper.bind(this))
  }

  async interact(counterparty: PeerId, acknowledgement: Acknowledgement<Chain>): Promise<void> {
    return new Promise<void>(async (resolve) => {
      let struct: Handler
      let aborted = false
      const abort = new AbortController()

      const timeout = setTimeout(() => {
        aborted = true
        abort.abort()
        error(`Timeout while trying to send acknowledgement to ${counterparty.toB58String()}.`)
        resolve()
      }, ACKNOWLEDGEMENT_TIMEOUT)

      try {
        struct = await this.node.dialProtocol(counterparty, this.protocols[0]).catch(async (err: Error) => {
          const result = await this.node.peerRouting.findPeer(counterparty)

          return await this.node.dialProtocol(result, this.protocols[0])
        })
      } catch (err) {
        clearTimeout(timeout)
        error(
          `Could not transfer acknowledgement to ${counterparty.toB58String()}. Error was: ${chalk.red(err.message)}.`
        )
        return
      }

      clearTimeout(timeout)

      pipe([acknowledgement], struct.stream)

      if (!aborted) {
        resolve()
      }
    })
  }

  async handleHelper(source: AsyncIterable<Uint8Array>): Promise<void> {
    for await (const msg of source) {
      const arr = msg.slice()
      const acknowledgement = new Acknowledgement(this.node.paymentChannels, {
        bytes: arr.buffer,
        offset: arr.byteOffset
      })

      const unAcknowledgedDbKey = this.node.dbKeys.UnAcknowledgedTickets(await acknowledgement.hashedKey)

      let tmp: Uint8Array
      try {
        tmp = await this.node.db.get(Buffer.from(unAcknowledgedDbKey))
      } catch (err) {
        if (err.notFound) {
          error(
            `received unknown acknowledgement from party ${chalk.blue(
              (await pubKeyToPeerId(await acknowledgement.responseSigningParty)).toB58String()
            )} for challenge ${chalk.yellow(u8aToHex(await acknowledgement.hashedKey))} - response was ${chalk.green(
              u8aToHex(await acknowledgement.hashedKey)
            )}. ${chalk.red('Dropping acknowledgement')}.`
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
          ticketCounter = toU8a(
            u8aToNumber(await this.node.db.get(Buffer.from(this.node.dbKeys.AcknowledgedTicketCounter()))) + 1,
            ACKNOWLEDGED_TICKET_INDEX_LENGTH
          )
        } catch (err) {
          // Set ticketCounter to initial value
          ticketCounter = toU8a(0, ACKNOWLEDGED_TICKET_INDEX_LENGTH)
        }

        let acknowledgedTicket = this.node.paymentChannels.types.AcknowledgedTicket.create(
          this.node.paymentChannels,
          undefined,
          {
            signedTicket: await unacknowledgedTicket.signedTicket,
            response: await this.node.paymentChannels.utils.hash(
              u8aConcat(unacknowledgedTicket.secretA, await acknowledgement.hashedKey)
            ),
            redeemed: false
          }
        )

        if (!(await this.node.paymentChannels.account.reservePreImageIfIsWinning(acknowledgedTicket))) {
          log(`Got a ticket that is not a win. Dropping ticket.`)
          await this.node.db.del(Buffer.from(unAcknowledgedDbKey))
        }

        const acknowledgedDbKey = this.node.dbKeys.AcknowledgedTickets(ticketCounter)

        log(`storing ticket`, ticketCounter, `we are`, this.node.peerInfo.id.toB58String())

        try {
          await this.node.db
            .batch()
            .del(Buffer.from(unAcknowledgedDbKey))
            .put(Buffer.from(acknowledgedDbKey), Buffer.from(acknowledgedTicket))
            .put(Buffer.from(this.node.dbKeys.AcknowledgedTicketCounter()), Buffer.from(ticketCounter))
            .write()
        } catch (err) {
          error(`Error while writing to database. Error was ${chalk.red(err.message)}.`)
        }
      } else {
        // Deleting dummy DB entry
        await this.node.db.del(Buffer.from(unAcknowledgedDbKey))
      }

      this.emit(u8aToHex(unAcknowledgedDbKey))
    }
  }
}

export { PacketAcknowledgementInteraction }
