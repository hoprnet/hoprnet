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
      async (source: any) => {
        for await (const msg of source) {
          const acknowledgement = new Acknowledgement(this.node.paymentChannels, msg, undefined)
          let record: any

          const unAcknowledgedDbKey = this.node.dbKeys.UnAcknowledgedTickets(acknowledgement.responseSigningParty, acknowledgement.hashedKey)
          try {
            record = await this.node.db.get(unAcknowledgedDbKey)
          } catch (err) {
            if (err.notFound == true) {
              this.node.log(
                `received unknown acknowledgement from party ${chalk.blue(u8aToHex(acknowledgement.responseSigningParty))} for challenge ${chalk.yellow(
                  u8aToHex(acknowledgement.hashedKey)
                )} - response was ${chalk.green(u8aToHex(acknowledgement.responseSigningParty))}. ${chalk.red('Dropping acknowledgement')}.`
              )
            } else {
              this.node.log(`Database error: ${err.message}. ${chalk.red('Dropping acknowledgement')}.`)
            }
            continue
          }

          const acknowledgedDbKey = this.node.dbKeys.AcknowledgedTickets(acknowledgement.responseSigningParty, acknowledgement.key)

          await this.node.db
            .batch()
            .del(unAcknowledgedDbKey)
            .put(acknowledgedDbKey, record)
            .write()

          this.emit(u8aToHex(unAcknowledgedDbKey))
        }
      }
    )
  }

  async interact(counterparty: PeerId, acknowledgement: Acknowledgement<Chain>): Promise<void> {
    let struct: {
      stream: any
      protocol: string
    }

    try {
      struct = await this.node.dialProtocol(counterparty, this.protocols[0]).catch(async (err: Error) => {
        return this.node.peerRouting.findPeer(counterparty).then((peerInfo: PeerInfo) => this.node.dialProtocol(peerInfo, this.protocols[0]))
      })
    } catch (err) {
      this.node.log(`Could not transfer packet to ${counterparty.toB58String()}. Error was: ${chalk.red(err.message)}.`)
      return
    }

    return pipe(
      /* prettier-ignore */
      [acknowledgement],
      struct.stream
    )
  }
}

export { PacketAcknowledgementInteraction }

//   async function handleAcknowledgement(ack) {
//     if (!ack.challengeSigningParty.equals(node.peerInfo.id.pubKey.marshal())) {
//       console.log(
//         `peer ${node.peerInfo.id.toB58String()} channelId ${getId(
//           pubKeyToEthereumAddress(node.peerInfo.id.pubKey.marshal()),
//           pubKeyToEthereumAddress(ack.responseSigningParty)
//         ).toString('hex')}`
//       )

//       return node.paymentChannels.contractCall(
//         node.paymentChannels.contract.methods.wrongAcknowledgement(
//           ack.challengeSignature.slice(0, 32),
//           ack.challengeSignature.slice(32, 64),
//           ack.responseSignature.slice(0, 32),
//           ack.responseSignature.slice(32, 64),
//           ack.key,
//           ack.challengeSignatureRecovery,
//           ack.responseSignatureRecovery
//         ),
//         (err, receipt) => {
//           console.log(err, receipt)
//         }
//       )
//     }