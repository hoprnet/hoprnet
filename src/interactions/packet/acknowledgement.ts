import { AbstractInteraction } from '../abstractInteraction'

import pipe from 'it-pipe'
import PeerId from 'peer-id'

import AbortController from 'abort-controller'

import debug from 'debug'
const error = debug('hopr-core:acknowledgement:error')

import chalk from 'chalk'

import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '../../'
import { Acknowledgement } from '../../messages/acknowledgement'

import type { Handler } from '../../network/transport/types'

import EventEmitter from 'events'

import { PROTOCOL_ACKNOWLEDGEMENT } from '../../constants'
import { u8aToHex, durations } from '@hoprnet/hopr-utils'
import { pubKeyToPeerId } from '../../utils'

const ACKNOWLEDGEMENT_TIMEOUT = durations.seconds(2)

class PacketAcknowledgementInteraction<Chain extends HoprCoreConnector> extends EventEmitter
  implements AbstractInteraction<Chain> {
  protocols: string[] = [PROTOCOL_ACKNOWLEDGEMENT]

  constructor(public node: Hopr<Chain>) {
    super()
    this.node.handle(this.protocols, this.handler.bind(this))
  }

  handler(struct: Handler) {
    pipe(
      /* prettier-ignore */
      struct.stream,
      handleHelper.bind(this)
    )
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

      pipe(
        /* prettier-ignore */
        [acknowledgement],
        struct.stream
      )

      if (!aborted) {
        resolve()
      }
    })
  }
}

async function handleHelper(source: any): Promise<void> {
  for await (const msg of source) {
    const arr = msg.slice()
    const acknowledgement = new Acknowledgement(this.node.paymentChannels, {
      bytes: arr.buffer,
      offset: arr.byteOffset,
    })

    const unAcknowledgedDbKey = this.node.dbKeys.UnAcknowledgedTickets(
      await acknowledgement.responseSigningParty,
      await acknowledgement.hashedKey
    )

    //  console.log(unAcknowledgedDbKey, this.node.db)

    let record: any
    try {
      record = await this.node.db.get(Buffer.from(unAcknowledgedDbKey))
    } catch (err) {
      if (err.notFound) {
        error(
          `received unknown acknowledgement from party ${chalk.blue(
            (await pubKeyToPeerId(await acknowledgement.responseSigningParty)).toB58String()
          )} for challenge ${chalk.yellow(u8aToHex(await acknowledgement.hashedKey))} - response was ${chalk.green(
            u8aToHex(await acknowledgement.hashedKey)
          )}. ${chalk.red('Dropping acknowledgement')}.`
        )
      } else {
        error(`Database error. Error was: ${err}`)
      }

      // Dropping ack and handling next message
      continue
    }

    const acknowledgedDbKey = this.node.dbKeys.AcknowledgedTickets(
      await acknowledgement.responseSigningParty,
      acknowledgement.key
    )

    try {
      await this.node.db
        /* prettier-ignore */
        .batch()
        .del(Buffer.from(unAcknowledgedDbKey))
        .put(acknowledgedDbKey, record)
        .write()
    } catch (err) {
      error(`Error while writing to database. Error was ${chalk.red(err.message)}.`)
    }

    this.emit(u8aToHex(unAcknowledgedDbKey))
  }
}

export { PacketAcknowledgementInteraction }
