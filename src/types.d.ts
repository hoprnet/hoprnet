import type BN from 'bn.js'
import type ChannelInstance from './channel'
import type HoprCoreConnector from '.'

declare namespace AccountId {
  const SIZE: number
}

declare class AccountId extends Uint8Array {}

declare namespace Balance {
  const SIZE: number

  /**
   * Abbreviation of the currency, e.g. `ETH`
   */
  const SYMBOL: string

  /**
   * Decimals of the currency, e.g. 18
   */
  const DECIMALS: number
}
declare class Balance extends BN {}

declare namespace Channel {
  function createFunded<CoreConnector extends HoprCoreConnector>(channelBalance: ChannelBalance<CoreConnector>): Channel

  function createActive<CoreConnector extends HoprCoreConnector>(channelBalance: ChannelBalance<CoreConnector>): Channel

  function createPending<CoreConnector extends HoprCoreConnector>(pending: Moment, balance: ChannelBalance<CoreConnector>): Channel
}
declare class Channel {
  toU8a(): Uint8Array
}

declare namespace ChannelBalance {
  const SIZE: number
}
declare class ChannelBalance<CoreConnector extends HoprCoreConnector> {
  balance: Balance
  balance_a: Balance

  constructor(coreConnector: CoreConnector, struct: {
    balance: Balance | BN,
    balance_a: Balance | BN
  })
}

declare namespace Hash {
  const SIZE: number
}
declare class Hash extends Uint8Array {}

declare namespace Moment {
  const SIZE: number
}
declare class Moment extends BN {}

declare namespace Signature {
  const SIZE: number
}
declare class Signature {
  onChainSignature: Uint8Array
  signature: Uint8Array
  recovery: number
  msgPrefix: Uint8Array

  constructor(
    arr?: {
      bytes: ArrayBuffer
      offset: number
    },
    struct?: {
      secp256k1Signature: Uint8Array
      secp256k1Recovery: number
    }
  )
}

declare namespace SignedChannel {
  const SIZE: number

  function create<CoreConnector extends HoprCoreConnector, ConcreteChannel extends Channel, ConcreteSignature extends Signature>(
    coreConnector: CoreConnector,
    channel: ConcreteChannel,
    arr?: { bytes: ArrayBuffer; offset: number }
  ): Promise<SignedChannel<ConcreteSignature, ConcreteChannel>>
}
declare class SignedChannel<ConcreteSignature extends Signature, ConcreteChannel extends Channel> {
  channel: ConcreteChannel
  signature: ConcreteSignature
  signer: Uint8Array

  constructor(
    arr?: {
      bytes: ArrayBuffer
      offset: number
    },
    struct?: {
      signature: ConcreteSignature
      channel: ConcreteChannel
    }
  )
}

declare namespace SignedTicket {
  const SIZE: number
}
declare class SignedTicket<ConcreteSignature extends Signature, ConcreteTicket extends Ticket> extends Uint8Array {
  ticket: ConcreteTicket
  signature: ConcreteSignature
  signer: Promise<Uint8Array>
}

declare namespace State {
  const SIZE: number
}
declare class State {
  toU8a(): Uint8Array
}

declare namespace Ticket {
  const SIZE: number

  /**
   * Constructs a ticket to use in a probabilistic payment channel.
   * @param amount amount of funds to include
   * @param challenge a challenge that has to be solved be the redeemer
   */
  function create<ConcreteChannel extends ChannelInstance, ConcreteSignature extends Signature>(
    channel: ConcreteChannel,
    amount: Balance,
    challenge: Hash,
  ): Promise<SignedTicket<ConcreteSignature, Ticket>>

  /**
   * Checks a previously issued ticket for its validity.
   * @param signedTicket a previously issued ticket to check
   * @param props additional arguments
   */
  function verify<ConcreteChannel extends ChannelInstance, ConcreteSignature extends Signature>(channel: ConcreteChannel, signedTicket: SignedTicket<ConcreteSignature, Ticket>): Promise<boolean>

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
  function submit<ConcreteChannel extends ChannelInstance, ConcreteSignature extends Signature>(channel: ConcreteChannel, signedTicket: SignedTicket<ConcreteSignature, Ticket>): Promise<void>

}
declare class Ticket {
  channelId: Hash
  challenge: Hash
  epoch: TicketEpoch
  amount: Balance
  winProb: Hash
  onChainSecret: Hash

  getEmbeddedFunds(): Balance
}

declare namespace TicketEpoch {
  const SIZE: number
}
declare class TicketEpoch extends BN {
  toU8a(): Uint8Array
}

export { AccountId, Balance, Channel, ChannelBalance, Hash, Moment, State, Signature, SignedChannel, SignedTicket, Ticket, TicketEpoch }