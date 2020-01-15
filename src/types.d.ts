import BN from 'bn.js'

interface toU8a {
  toU8a: (...props: any[]) => Uint8Array
}

export namespace Types {
  interface AccountId extends Uint8Array {}

  interface Balance extends BN {}

  interface Hash extends Uint8Array {}

  interface Moment extends BN {}

  interface State extends toU8a {}

  interface SignedTicket extends toU8a {
    ticket: Ticket
    signature: Uint8Array
  }

  interface Ticket extends toU8a {
    channelId: Hash
    challenge: Hash
    epoch: TicketEpoch
    amount: Balance
    winProb: Hash
    onChainSecret: Hash
  }

  interface TicketEpoch extends BN, toU8a {}
}

export default interface Constructors {
  Balance: new (...props: any[]) => Types.Balance
  Hash: new (...props: any[]) => Types.Hash
  Moment: new (...props: any[]) => Types.Moment
  Ticket: new (...props: any[]) => Types.Ticket
  AccountId: new (...props: any[]) => Types.AccountId
  State: new (...props: any[]) => Types.State
  SignedTicket: new (...props: any[]) => Types.SignedTicket
  TicketEpoch: new (...props: any[]) => Types.TicketEpoch
}
