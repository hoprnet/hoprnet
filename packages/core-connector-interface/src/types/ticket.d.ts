import Signature from './signature'
import { UINT256, Address, Balance, Hash } from '.' // TODO: cyclic

declare interface TicketStatic {
  readonly SIZE: number
  create(
    counterparty: Address,
    challenge: Hash,
    epoch: UINT256,
    amount: Balance,
    winProb: Hash,
    channelIteration: UINT256,
    signPriv: Uint8Array
  ): Ticket
}

declare interface Ticket {
  counterparty: Address
  challenge: Hash
  epoch: UINT256
  amount: Balance
  winProb: Hash
  channelIteration: UINT256
  signature: Signature

  getHash(): Hash
  getEmbeddedFunds(): Balance
  serialize(): Uint8Array
  verify(pubKey: PublicKey): Promise<boolean>
  getSigner(): PublicKey
}

declare var Ticket: TicketStatic

export default Ticket
