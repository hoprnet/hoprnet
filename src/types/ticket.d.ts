import Balance from './balance'
import Hash from './hash'
import Signature from './signature'
import TicketEpoch from './ticketEpoch'

declare namespace Ticket {
  const SIZE: number
}
declare interface Ticket {
  channelId: Hash
  challenge: Hash
  epoch: TicketEpoch
  amount: Balance
  winProb: Hash
  onChainSecret: Hash

  getEmbeddedFunds(): Balance

  toU8a(): Uint8Array

  sign(
    privKey: Uint8Array,
    pubKey: Uint8Array,
    arr?: {
      bytes: ArrayBuffer
      offset: number
    }
  ): Promise<Signature>
}

export default Ticket
