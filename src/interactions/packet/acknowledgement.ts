import { AbstractInteraction } from '../abstractInteraction'

import pipe from 'it-pipe'
import PeerId from 'peer-id'
import type PeerInfo from 'peer-info'

import chalk from 'chalk'

import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '../../'
import { Acknowledgement } from '../../messages/acknowledgement'

import type { Handler } from '../../network/transport/types'

import EventEmitter from 'events'

import { PROTOCOL_ACKNOWLEDGEMENT } from '../../constants'
import { u8aToHex } from '@hoprnet/hopr-utils'

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
    let struct: Handler

    try {
      struct = await this.node
        .dialProtocol(counterparty, this.protocols[0])
        .catch(async (err: Error) => {
          return this.node.peerRouting
            .findPeer(counterparty)
            .then((peerInfo: PeerInfo) => this.node.dialProtocol(peerInfo, this.protocols[0]))
        })
    } catch (err) {
      console.log(
        `Could not transfer acknowledgement to ${counterparty.toB58String()}. Error was: ${chalk.red(
          err.message
        )}.`
      )
      return
    }

    await pipe(
      /* prettier-ignore */
      [acknowledgement],
      struct.stream
    )
  }
}

async function handleHelper(source: any): Promise<void> {
  let self = this

  for await (const msg of source) {
    const arr = msg.slice()
    const acknowledgement = new Acknowledgement(self.node.paymentChannels, {
      bytes: arr.buffer,
      offset: arr.byteOffset,
    })

    let record: any

    const unAcknowledgedDbKey = u8aToHex(
      self.node.dbKeys.UnAcknowledgedTickets(
        await acknowledgement.responseSigningParty,
        await acknowledgement.hashedKey
      )
    )

    try {
      record = await self.node.db.get(unAcknowledgedDbKey)

      const acknowledgedDbKey = self.node.dbKeys.AcknowledgedTickets(
        await acknowledgement.responseSigningParty,
        acknowledgement.key
      )
      try {
        await self.node.db.batch().del(unAcknowledgedDbKey).put(acknowledgedDbKey, record).write()
      } catch (err) {
        console.log(`Error while writing to database. Error was ${chalk.red(err.message)}.`)
      }
    } catch (err) {
      if (err.notFound == true) {
        // console.log(
        //   `${chalk.blue(this.node.peerInfo.id.toB58String())} received unknown acknowledgement from party ${chalk.blue(
        //     (await pubKeyToPeerId(acknowledgement.responseSigningParty)).toB58String()
        //   )} for challenge ${chalk.yellow(u8aToHex(await acknowledgement.hashedKey))} - response was ${chalk.green(
        //     u8aToHex(await acknowledgement.hashedKey)
        //   )}. ${chalk.red('Dropping acknowledgement')}.`
        // )
      } else {
        self.node.log(`Database error: ${err.message}. ${chalk.red('Dropping acknowledgement')}.`)
      }
      continue
    } finally {
      self.emit(unAcknowledgedDbKey)
    }
  }
}

export { PacketAcknowledgementInteraction }
