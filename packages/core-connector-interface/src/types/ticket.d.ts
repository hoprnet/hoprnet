import Signature from './signature'
import { UINT256, Address, Balance, Hash } from '.' // TODO: cyclic

declare interface TicketStatic {
  readonly SIZE: number

  new (
    counterparty: Address,
    challenge: Hash,
    epoch: UINT256,
    amount: Balance,
    winProb: Hash,
    channelIteration: UINT256
  ): Ticket
}
declare interface Ticket {
  counterparty: Address
  challenge: Hash
  epoch: UINT256
  amount: Balance
  winProb: Hash
  channelIteration: UINT256

  getHash(): Hash
  getEmbeddedFunds(): Balance
  serialize(): Uint8Array
  sign(privKey: Uint8Array): Promise<Signature>
}

declare var Ticket: TicketStatic

export default Ticket
