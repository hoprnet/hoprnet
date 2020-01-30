import BN from 'bn.js'
import { ChannelInstance } from './channel'

declare interface toU8a {
  toU8a: (...props: any[]) => Uint8Array
}

interface length<Instance> {
  new (...props: any[]): Instance
  SIZE: number
}

declare namespace AccountId {
  interface Static extends length<Instance> {}

  interface Instance extends Uint8Array {}
}

declare namespace Balance {
  interface Static extends length<Instance> {
    /**
     * Abbreviation of the currency, e.g. `ETH`
     */
    readonly SYMBOL: string

    /**
     * Decimals of the currency, e.g. 18
     */
    readonly DECIMALS: number
  }

  interface Instance extends BN {}
}

declare namespace Channel {
  interface Static extends length<Instance> {}

  interface Instance extends toU8a {}
}

declare namespace ChannelBalance {
  interface Static extends length<Instance> {
    new (coreConnector: any, struct: {
      balance: BN,
      balance_a: BN
    }): Instance
  }

  interface Instance extends toU8a {
    balance: Balance.Instance
    balance_a: Balance.Instance
  }
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
    recovery: number
    msgPrefix: Uint8Array
  }
}

declare namespace SignedChannel  {
  interface Static<ConcreteSignature extends Signature.Instance, ConcreteChannel extends Channel.Instance> extends length<Instance<ConcreteSignature, ConcreteChannel>> {
    new (
      coreConnector: any,
      arr?: Uint8Array,
      struct?: {
        signature: ConcreteSignature
        channel: ConcreteChannel
      }
    ): Instance<ConcreteSignature, ConcreteChannel>
  }

  interface Instance<ConcreteSignature extends Signature.Instance, ConcreteChannel extends Channel.Instance> extends Uint8Array, toU8a {
    channel: Channel.Instance
    signature: Signature.Instance
    signer: Uint8Array
  }
}

declare namespace SignedTicket {
  interface Static extends length<Instance> {}

  interface Instance extends Uint8Array {
    ticket: Ticket.Instance
    signature: Signature.Instance
    signer: Uint8Array
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
    create(
      channel: any,
      amount: Balance.Instance,
      challenge: Hash.Instance,
      privKey: Uint8Array,
      pubKey: Uint8Array,
      ...props: any[]
    ): Promise<SignedTicket.Instance>

    /**
     * Checks a previously issued ticket for its validity.
     * @param signedTicket a previously issued ticket to check
     * @param props additional arguments
     */
    verify(channel: any, signedTicket: SignedTicket.Instance, ...props: any[]): Promise<boolean>

    /**
     * BIG TODO
     * Aggregate previously issued tickets. Still under active development!
     * @param tickets array of tickets to aggregate
     * @param props additional arguments
     */
    // aggregate(channel: any, tickets: Ticket[], ...props: any[]): Promise<Ticket>

    /**
     * Submits a signed to the blockchain.
     * @param signedTicket a signed ticket
     */
    submit(channel: any, signedTicket: SignedTicket.Instance, ...props: any[]): Promise<void>
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
  interface ChannelBalance extends ChannelBalance.Instance {}
  interface Hash extends Hash.Instance {}
  interface Moment extends Moment.Instance {}
  interface State extends State.Instance {}
  interface Signature extends Signature.Instance {}
  interface SignedChannel extends SignedChannel.Instance<any, any> {}
  interface SignedTicket extends SignedTicket.Instance {}
  interface Ticket extends Ticket.Instance {}
  interface TicketEpoch extends TicketEpoch.Instance {}
}

declare interface TypeConstructors {
  AccountId: AccountId.Static
  Balance: Balance.Static
  Channel: Channel.Static
  ChannelBalance: ChannelBalance.Static
  Hash: Hash.Static
  Moment: Moment.Static
  State: State.Static
  Signature: Signature.Static
  SignedChannel: SignedChannel.Static<any, any>
  SignedTicket: SignedTicket.Static
  Ticket: Ticket.Static
  TicketEpoch: TicketEpoch.Static
}

export { AccountId, Balance, Channel, ChannelBalance, Hash, Moment, State, Signature, SignedChannel, SignedTicket, Ticket, TicketEpoch, Types, toU8a }

export default TypeConstructors
