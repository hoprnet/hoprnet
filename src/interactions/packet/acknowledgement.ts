import { AbstractInteraction } from '../abstractInteraction'

import pipe from 'it-pipe'
import PeerId from 'peer-id'
import PeerInfo from 'peer-info'

import chalk from 'chalk'

import { HoprCoreConnectorInstance } from '@hoprnet/hopr-core-connector-interface'
import Hopr from '../../'
import { Acknowledgement } from '../../messages/acknowledgement'

import EventEmitter from 'events'

import { PROTOCOL_ACKNOWLEDGEMENT } from '../../constants'
import { u8aToHex } from '../../utils'

class PacketAcknowledgementInteraction<Chain extends HoprCoreConnectorInstance> extends EventEmitter implements AbstractInteraction<Chain> {
  protocols: string[] = [PROTOCOL_ACKNOWLEDGEMENT]

  constructor(public node: Hopr<Chain>) {
    super()
    this.node.handle(this.protocols, this.handler.bind(this))
  }

  handler(struct: { stream: any }) {
    pipe(
      /* prettier-ignore */
      struct.stream,
      handleHelper.bind(this)
    )
  }

  async interact(counterparty: PeerId, acknowledgement: Acknowledgement<Chain>): Promise<void> {
    console.log(`sending acknowledgement to `, counterparty.toB58String())
    let struct: {
      stream: any
      protocol: string
    }

    try {
      struct = await this.node.dialProtocol(counterparty, this.protocols[0]).catch(async (err: Error) => {
        return this.node.peerRouting.findPeer(counterparty).then((peerInfo: PeerInfo) => this.node.dialProtocol(peerInfo, this.protocols[0]))
      })
    } catch (err) {
      console.log(err)

      console.log(`Could not transfer acknowledgement to ${counterparty.toB58String()}. Error was: ${chalk.red(err.message)}.`)
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
  for await (const msg of source) {
    const arr = msg.slice()
    const acknowledgement = new Acknowledgement(this.node.paymentChannels, {
      bytes: arr.buffer,
      offset: arr.byteOffset
    })

    console.log(`received acknowledgement`, u8aToHex(acknowledgement))

    let record: any

    const unAcknowledgedDbKey = u8aToHex(this.node.dbKeys.UnAcknowledgedTickets(acknowledgement.responseSigningParty, await acknowledgement.hashedKey))

    try {
      record = await this.node.db.get(unAcknowledgedDbKey)

      const acknowledgedDbKey = this.node.dbKeys.AcknowledgedTickets(acknowledgement.responseSigningParty, acknowledgement.key)
      try {
        await this.node.db
          .batch()
          .del(unAcknowledgedDbKey)
          .put(acknowledgedDbKey, record)
          .write()
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
        this.node.log(`Database error: ${err.message}. ${chalk.red('Dropping acknowledgement')}.`)
      }
      continue
    } finally {
      this.emit(unAcknowledgedDbKey)
    }
  }
}

export { PacketAcknowledgementInteraction }