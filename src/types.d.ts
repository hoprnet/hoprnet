import BN from 'bn.js'

export namespace Types {
  interface AccountId extends Uint8Array {}

  interface Balance extends BN {}

  interface Hash extends Uint8Array {}

  interface Moment extends Uint8Array {}

  interface State {}

  interface SignedTicket extends Uint8Array {
    ticket: Ticket
    signature: Uint8Array
  }

  interface Ticket extends Uint8Array {}
}

export default interface Constructors {
  Balance: new (...props: any[]) => Types.Balance
  Hash: new (...props: any[]) => Types.Hash
  Moment: new (...props: any[]) => Types.Moment
  Ticket: new (...props: any[]) => Types.Ticket
  AccountId: new (...props: any[]) => Types.AccountId
  State: new (...props: any[]) => Types.State
  SignedTicket: new (...props: any[]) => Types.SignedTicket
}
