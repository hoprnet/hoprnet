import BN from 'bn.js'

export class Balance extends BN {}

export class Hash extends Uint8Array {}

export class Moment {}

export class Ticket {}

export class AccountId extends Uint8Array {}

export class State {}

export class SignedTicket<Ticket> {
  lotteryTicket: Ticket
  signature: Uint8Array
}
