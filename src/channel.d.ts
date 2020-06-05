import type { AccountId, Balance, Channel as ChannelType, ChannelBalance, Hash, Moment, Ticket, Signature, SignedChannel, SignedTicket } from './types'

declare namespace Channel {
  /**
   * Creates a Channel instance from the database.
   * @param counterparty AccountId of the counterparty
   * @param props additional arguments
   */
  function create(
    offChainCounterparty: Uint8Array,
    getOnChainPublicKey: (counterparty: Uint8Array) => Promise<Uint8Array>,
    channelBalance?: ChannelBalance,
    sign?: (channelBalance: ChannelBalance) => Promise<SignedChannel>
  ): Promise<Channel>

  /**
   * Creates a dummy ticket that is sent to the final recipient.
   * The ticket MUST not have any value.
   * 
   * @param counterParty AccountId of the counterparty
   * @param challenge Challenge for this ticket
   */
  function createDummyChannelTicket(counterParty: AccountId, challenge: Hash, ...props: any[]): Promise<SignedTicket>

  /**
   * Checks whether the channel exists on-chain and off-chain, i.e. in our database.
   * Returns `true` if the channel exists on-chain AND off-chain.
   * @param counterparty AccountId of the counterparty
   */
  function isOpen(
    counterparty: AccountId
  ): Promise<boolean>

  /**
   * Opens a new payment channel and initializes the on-chain data.
   * @param amount how much should be staked
   * @param signature a signature over channel state
   * @param props additional arguments
   */
  // open(coreConnector: any, amount: Balance.Instance, signature: Promise<Uint8Array>, ...props: any[]): Promise<ChannelInstance>

  /**
   * Fetches all channel instances from the database and applies first `onData` and
   * then `onEnd` on the received nstances.
   * @param onData applied on all channel instances
   * @param onEnd composes at the end the received data
   */
  function getAll<T, R>(
    onData: (channel: Channel, ...props: any[]) => Promise<T>,
    onEnd: (promises: Promise<T>[], ...props: any[]) => R,
  ): Promise<R>

  /**
   * Fetches all channel instances from the database and initiates a settlement on
   * each of them.
   * @param props additional arguments
   */
  function closeChannels(): Promise<Balance>
  
  /**
   * 
   * @param counterParty 
   * @param amount 
   */
  function increaseFunds(counterParty: AccountId, amount: Balance): Promise<void>

  /**
   * Handles a channel opening request.
   * @notice Takes the `coreConnector` instance and returns an async iterable duplex stream.
   * @param coreConnector coreConnector instance
   */
  function handleOpeningRequest(...props: any[]): (source: AsyncIterable<Uint8Array>) => AsyncIterable<Uint8Array>
}

declare interface Channel {
  // Id of the channel
  readonly channelId: Promise<Hash>

  // Timestamp once the channel can be settled
  readonly settlementWindow: Promise<Moment>

  // Current state of the channel, i.e. `FUNDED`
  readonly state: Promise<ChannelType>

  // Current balance of partyA
  readonly balance_a: Promise<Balance>

  // Current total balance (sum of balance_a and balance_b)
  readonly balance: Promise<Balance>

  readonly currentBalance: Promise<Balance>

  readonly currentBalanceOfCounterparty: Promise<Balance>

  readonly ticket: typeof Ticket

  readonly counterparty: AccountId

  readonly offChainCounterparty: Promise<Uint8Array>

  /**
   * Initiates a settlement for this channel.
   */
  initiateSettlement(): Promise<void>

  /**
   * Fetches all unresolved, previous challenges from the database that
   * have occured in this channel.
   */
  getPreviousChallenges(): Promise<Hash>
}

export default Channel
