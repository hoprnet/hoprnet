import debug from 'debug'
import { PublicKey, durations } from '@hoprnet/hopr-utils'
import PeerId from 'peer-id'
import HoprCoreEthereum from '@hoprnet/hopr-core-ethereum'
import { PROTOCOL_ACKNOWLEDGEMENT } from '../../constants'
import { Acknowledgement, Packet } from '../../messages'
import { HoprDB } from '@hoprnet/hopr-utils'
const log = debug('hopr-core:acknowledgement')

const ACKNOWLEDGEMENT_TIMEOUT = durations.seconds(2)

export function subscribeToAcknowledgements(
  subscribe: any,
  db: HoprDB,
  chain: HoprCoreEthereum,
  pubKey: PeerId,
  onMessage: (ackMessage: Acknowledgement) => void
) {
  subscribe(PROTOCOL_ACKNOWLEDGEMENT, async function (msg: Uint8Array, remotePeer: PeerId) {
    const ackMsg = Acknowledgement.deserialize(msg, pubKey, remotePeer)
    let unacknowledgedTicket = await db.getUnacknowledgedTicketsByKey(ackMsg.ackChallenge)

    if (!unacknowledgedTicket) {
      // Could be dummy, could be error.
      log('dropping unknown ticket')
      return await db.deleteTicket(ackMsg.ackChallenge)
    }

    const channel = chain.getChannel(new PublicKey(pubKey.pubKey.marshal()), new PublicKey(remotePeer.pubKey.marshal()))

    const ackedTicket = await channel.acknowledge(unacknowledgedTicket, ackMsg.ackKeyShare)

    if (ackedTicket === null) {
      log(`Got a ticket that is not a win. Dropping ticket.`)
      await db.deleteTicket(ackMsg.ackChallenge)
    } else {
      log(`Storing winning ticket`)
      await db.replaceTicketWithAcknowledgement(ackMsg.ackChallenge, ackedTicket)
    }

    onMessage(ackMsg)
  })
}

export function sendAcknowledgement(packet: Packet, destination: PeerId, sendMessage: any, privKey: PeerId): void {
  setImmediate(async () => {
    const ack = packet.createAcknowledgement(privKey)

    sendMessage(destination, PROTOCOL_ACKNOWLEDGEMENT, ack.serialize(), {
      timeout: ACKNOWLEDGEMENT_TIMEOUT
    })
  })
}
