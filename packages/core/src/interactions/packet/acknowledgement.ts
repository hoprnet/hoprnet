import type { LevelUp } from 'levelup'
import debug from 'debug'
import { Acknowledgement } from '../../messages/acknowledgement'
import { getUnacknowledgedTickets, deleteTicket, replaceTicketWithAcknowledgement } from '../../dbKeys'
import type HoprCoreEthereum from '@hoprnet/hopr-core-ethereum'
import { PROTOCOL_ACKNOWLEDGEMENT } from '../../constants'
import PeerId from 'peer-id'
import { PublicKey, durations, Hash } from '@hoprnet/hopr-utils'
import type { Packet } from '../../messages/packet'
const log = debug('hopr-core:acknowledgement')

const ACKNOWLEDGEMENT_TIMEOUT = durations.seconds(2)

export function subscribeToAcknowledgements(
  subscribe: any,
  db: LevelUp,
  chain: HoprCoreEthereum,
  pubKey: PeerId,
  onMessage: (ackMessage: Acknowledgement) => void
) {
  subscribe(PROTOCOL_ACKNOWLEDGEMENT, async function (msg: Uint8Array, remotePeer: PeerId) { 
    const ackMsg = Acknowledgement.deserialize(msg, pubKey, remotePeer)
    let unacknowledgedTicket = await getUnacknowledgedTickets(db, ackMsg.ackChallenge)

    if (!unacknowledgedTicket) {
      // Could be dummy, could be error.
      log('dropping unknown ticket')
      return await deleteTicket(db, ackMsg.ackChallenge)
    }

    const channel = chain.getChannel(new PublicKey(pubKey.pubKey.marshal()), new PublicKey(remotePeer.pubKey.marshal()))

    const ackedTicket = await channel.acknowledge(unacknowledgedTicket, new Hash(ackMsg.ackKeyShare))

    if (ackedTicket === null) {
      log(`Got a ticket that is not a win. Dropping ticket.`)
      await deleteTicket(db, ackMsg.ackChallenge)
    } else {
      log(`Storing winning ticket`)
      await replaceTicketWithAcknowledgement(db, ackMsg.ackChallenge, ackedTicket)
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
