import BN from 'bn.js'

declare interface toU8a {
  toU8a: (...props: any[]) => Uint8Array
}

interface length<Instance> {
  new (...props: any[]): Instance
  length: number
}

declare namespace AccountId {
  interface Static extends length<Instance> {}

  interface Instance extends Uint8Array {}
}

declare namespace Balance {
  interface Static extends length<Instance> {}

  interface Instance extends BN {}
}

declare namespace Channel {
  interface Static extends length<Instance> {}

  interface Instance extends toU8a {}
}

declare namespace Hash {
  interface Static extends length<Instance> {}

  interface Instance extends Uint8Array {}
}

declare namespace Moment {
  interface Static extends length<Instance> {}

  interface Instance extends BN {}
}

declare namespace Signature {
  interface Static extends length<Instance> {}

  interface Instance extends Uint8Array {
    onChainSignature: Uint8Array
    signature: Uint8Array
  }
}

declare namespace SignedTicket {
  interface Static extends length<Instance> {}

  interface Instance extends Uint8Array {
    ticket: Ticket.Instance
    signature: Signature.Instance
  }
}

declare namespace State {
  interface Static extends length<Instance> {}

  interface Instance extends toU8a {}
}

declare namespace Ticket {
  interface Static extends length<Instance> {
    /**
     * Constructs a ticket to use in a probabilistic payment channel.
     * @param amount amount of funds to include
     * @param challenge a challenge that has to be solved be the redeemer
     */
    create(amount: Balance.Instance, challenge: Hash.Instance, privKey: Uint8Array, pubKey: Uint8Array): Promise<SignedTicket.Instance>

    /**
     * Checks a previously issued ticket for its validity.
     * @param signedTicket a previously issued ticket to check
     * @param props additional arguments
     */
    verify(signedTicket: SignedTicket.Instance, ...props: any[]): Promise<boolean>

    /**
     * BIG TODO
     * Aggregate previously issued tickets. Still under active development!
     * @param tickets array of tickets to aggregate
     * @param props additional arguments
     */
    // aggregate(tickets: Ticket[], ...props: any[]): Promise<Ticket>

    /**
     * Submits a signed to the blockchain.
     * @param signedTicket a signed ticket
     */
    submit(signedTicket: SignedTicket.Instance): Promise<void>
  }

  interface Instance extends toU8a {
    channelId: Hash.Instance
    challenge: Hash.Instance
    epoch: TicketEpoch.Instance
    amount: Balance.Instance
    winProb: Hash.Instance
    onChainSecret: Hash.Instance

    getEmbeddedFunds(): Balance.Instance
  }
}

declare namespace TicketEpoch {
  interface Static extends length<Instance> {}

  interface Instance extends BN, toU8a {}
}

declare namespace Types {
  interface AccountId extends AccountId.Instance {}
  interface Balance extends Balance.Instance {}
  interface Channel extends Channel.Instance {}
  interface Hash extends Hash.Instance {}
  interface Moment extends Moment.Instance {}
  interface State extends State.Instance {}
  interface Signature extends Signature.Instance {}
  interface SignedTicket extends SignedTicket.Instance {}
  interface Ticket extends Ticket.Instance {}
  interface TicketEpoch extends TicketEpoch.Instance {}
}

declare interface TypeConstructors {
  AccountId: AccountId.Static
  Balance: Balance.Static
  Channel: Channel.Static
  Hash: Hash.Static
  Moment: Moment.Static
  State: State.Static
  SignedTicket: SignedTicket.Static
  Ticket: Ticket.Static
  TicketEpoch: TicketEpoch.Static
}

export { AccountId, Balance, Channel, Hash, Moment, State, Signature, SignedTicket, Ticket, TicketEpoch, Types, toU8a }

export default TypeConstructors
