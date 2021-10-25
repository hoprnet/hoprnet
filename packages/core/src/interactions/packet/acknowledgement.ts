import { debug } from '@hoprnet/hopr-utils'
import { PublicKey, durations, oneAtATime } from '@hoprnet/hopr-utils'
import type { SendMessage, Subscribe } from '../../index'
import type PeerId from 'peer-id'
import type HoprCoreEthereum from '@hoprnet/hopr-core-ethereum'
import { PROTOCOL_ACKNOWLEDGEMENT } from '../../constants'
import { Acknowledgement, Packet } from '../../messages'
import { HoprDB } from '@hoprnet/hopr-utils'
const log = debug('hopr-core:acknowledgement')
const error = debug('hopr-core:acknowledgement:error')

const ACKNOWLEDGEMENT_TIMEOUT = durations.seconds(2)

export function subscribeToAcknowledgements(
  subscribe: Subscribe,
  db: HoprDB,
  chain: HoprCoreEthereum,
  pubKey: PeerId,
  onMessage: (ackMessage: Acknowledgement) => void
) {
  async function handleAcknowledgement(msg: Uint8Array, remotePeer: PeerId) {
    const ackMsg = Acknowledgement.deserialize(msg, pubKey, remotePeer)

    try {
      let unacknowledgedTicket = await db.getUnacknowledgedTicket(ackMsg.ackChallenge)
      const channel = chain.getChannel(new PublicKey(pubKey.pubKey.marshal()), unacknowledgedTicket.signer)
      const ackedTicket = await channel.acknowledge(unacknowledgedTicket, ackMsg.ackKeyShare)
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
  subscribe(
    PROTOCOL_ACKNOWLEDGEMENT,
    (msg: Uint8Array, remotePeer: PeerId) => limitConcurrency(() => handleAcknowledgement(msg, remotePeer)),
    false,
    (err: any) => {
      error(`Error while receiving acknowledgement`, err)
    }
  )
}

export function sendAcknowledgement(
  packet: Packet,
  destination: PeerId,
  sendMessage: SendMessage,
  privKey: PeerId
): void {
  ;(async () => {
    const ack = packet.createAcknowledgement(privKey)

    try {
      await sendMessage(destination, PROTOCOL_ACKNOWLEDGEMENT, ack.serialize(), false, {
        timeout: ACKNOWLEDGEMENT_TIMEOUT
      })
    } catch (err) {
      // Currently unclear how to proceed if sending acknowledgements
      // fails
      error(`could not send acknowledgement`, err)
    }
  })()
}
