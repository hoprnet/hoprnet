import BN from 'bn.js'

declare class Balance extends BN {}

declare class Hash extends Uint8Array {}

declare class Moment {}

declare class Ticket {}

declare class AccountId extends Uint8Array {}

declare class State {}

declare class SignedTicket {
  lotteryTicket: Ticket
  signature: Uint8Array
}

export default interface Types {
  Balance: Balance
  Hash: Hash
  Moment: Moment
  Ticket: Ticket
  AccountId: AccountId
  State: State
  SignedTicket: SignedTicket
}
