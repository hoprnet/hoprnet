import BN from 'bn.js'

interface toU8a {
  toU8a: (...props: any[]) => Uint8Array
}

export namespace TypeClasses {
  interface AccountId extends Uint8Array {}

  interface Channel {}

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

export default interface Types {
  Balance: new (...props: any[]) => TypeClasses.Balance
  Channel: new (...props: any[]) => TypeClasses.Channel
  Hash: new (...props: any[]) => TypeClasses.Hash
  Moment: new (...props: any[]) => TypeClasses.Moment
  Ticket: new (...props: any[]) => TypeClasses.Ticket
  AccountId: new (...props: any[]) => TypeClasses.AccountId
  State: new (...props: any[]) => TypeClasses.State
  SignedTicket: new (...props: any[]) => TypeClasses.SignedTicket
  TicketEpoch: new (...props: any[]) => TypeClasses.TicketEpoch
}
