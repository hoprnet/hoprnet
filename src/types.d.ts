import type BN from 'bn.js'
import type { ChannelInstance } from './channel'
import type { HoprCoreConnectorInstance } from '.'

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
  interface Static extends length<Instance> {
    createFunded(balance: ChannelBalance.Instance): Instance

    createActive(balance: ChannelBalance.Instance): Instance

    createPending(pending: Moment.Instance, balance: ChannelBalance.Instance): Instance
  }

  interface Instance extends toU8a {}
}

declare namespace ChannelBalance {
  interface Static extends length<Instance> {
    new (
      coreConnector: any,
      struct: {
        balance: BN
        balance_a: BN
      }
    ): Instance
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

declare namespace SignedChannel {
  interface Static<ConcreteSignature extends Signature.Instance, ConcreteChannel extends Channel.Instance>
    extends length<Instance<ConcreteSignature, ConcreteChannel>> {
    new (
      arr?: {
        bytes: ArrayBuffer
        offset: number
      },
      struct?: {
        signature: ConcreteSignature
        channel: ConcreteChannel
      }
    ): Instance<ConcreteSignature, ConcreteChannel>

    create<CoreConnector extends HoprCoreConnectorInstance>(
      coreConnector: HoprCoreConnectorInstance,
      channel: ConcreteChannel,
      arr?: { bytes: ArrayBuffer; offset: number }
    ): Promise<Instance<ConcreteSignature, ConcreteChannel>>
  }

  interface Instance<ConcreteSignature extends Signature.Instance, ConcreteChannel extends Channel.Instance> extends Uint8Array {
    channel: Channel.Instance
    signature: Signature.Instance
    signer: Uint8Array
  }
}

declare namespace SignedTicket {
  interface Static<ConcreteSignature extends Signature.Instance, ConcreteChannel extends ChannelInstance, ConcreteTicket extends Ticket.Instance>
    extends length<Instance<ConcreteSignature, ConcreteChannel, ConcreteTicket>> {}

  interface Instance<ConcreteSignature extends Signature.Instance, ConcreteChannel extends ChannelInstance, ConcreteTicket extends Ticket.Instance>
    extends Uint8Array {
    ticket: ConcreteTicket
    signature: ConcreteChannel
    signer: Promise<Uint8Array>
  }
}

declare namespace State {
  interface Static extends length<Instance> {}

  interface Instance extends toU8a {}
}

declare namespace Ticket {
  interface Static<ConcreteChannel extends ChannelInstance, ConcreteSignature extends Signature.Instance> extends length<Instance> {
    /**
     * Constructs a ticket to use in a probabilistic payment channel.
     * @param amount amount of funds to include
     * @param challenge a challenge that has to be solved be the redeemer
     */
    create(
      channel: ConcreteChannel,
      amount: Balance.Instance,
      challenge: Hash.Instance,
      ...props: any[]
    ): Promise<SignedTicket.Instance<ConcreteSignature, ConcreteChannel, any>>

    /**
     * Checks a previously issued ticket for its validity.
     * @param signedTicket a previously issued ticket to check
     * @param props additional arguments
     */
    verify(channel: ConcreteChannel, signedTicket: SignedTicket.Instance<ConcreteSignature, ConcreteChannel, any>, ...props: any[]): Promise<boolean>

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
    submit(channel: ConcreteChannel, signedTicket: SignedTicket.Instance<ConcreteSignature, ConcreteChannel, any>, ...props: any[]): Promise<void>
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
  interface SignedTicket extends SignedTicket.Instance<any, any, any> {}
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
  SignedTicket: SignedTicket.Static<any, any, any>
  Ticket: Ticket.Static<any, any>
  TicketEpoch: TicketEpoch.Static
}

export { AccountId, Balance, Channel, ChannelBalance, Hash, Moment, State, Signature, SignedChannel, SignedTicket, Ticket, TicketEpoch, Types, toU8a }

export default TypeConstructors
