import debug from 'debug'
import { PublicKey, durations, oneAtATime } from '@hoprnet/hopr-utils'
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
  subscribe(PROTOCOL_ACKNOWLEDGEMENT, (msg: Uint8Array, remotePeer: PeerId) =>
    limitConcurrency(() => handleAcknowledgement(msg, remotePeer))
  )
}

export function sendAcknowledgement(packet: Packet, destination: PeerId, sendMessage: any, privKey: PeerId): void {
  setImmediate(async () => {
    const ack = packet.createAcknowledgement(privKey)

    sendMessage(destination, PROTOCOL_ACKNOWLEDGEMENT, ack.serialize(), {
      timeout: ACKNOWLEDGEMENT_TIMEOUT
    })
  })
}
