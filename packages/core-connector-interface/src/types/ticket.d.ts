import Hash from './hash'
import Signature from './signature'
import { UINT256, Address, Balance } from '.' // TODO: cyclic

declare interface TicketStatic {
  readonly SIZE: number

  create(
    arr?: {
      bytes: ArrayBuffer
      offset: number
    },
    struct?: {
      counterparty: Address
      challenge: Hash
      epoch: UINT256
      amount: Balance
      winProb: Hash
      channelIteration: UINT256
    }
  ): Ticket
}
declare interface Ticket {
  counterparty: Address
  challenge: Hash
  epoch: UINT256
  amount: Balance
  winProb: Hash
  channelIteration: UINT256

  // computed properties
  hash: Promise<Hash>

  getEmbeddedFunds(): Balance

  toU8a(): Uint8Array

  sign(privKey: Uint8Array): Promise<Signature>
}

declare var Ticket: TicketStatic

export default Ticket
