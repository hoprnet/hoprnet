import type { AccountId, Balance, Channel as ChannelType, ChannelBalance, Hash, Moment, Ticket } from './types'
import type HoprCoreConnector from '.'

declare namespace Channel {
  /**
   * Creates a Channel instance from the database.
   * @param counterparty AccountId of the counterparty
   * @param props additional arguments
   */
  function create<ConcreteConnector extends HoprCoreConnector>(
    coreConnector: ConcreteConnector,
    offChainCounterparty: Uint8Array,
    getOnChainPublicKey: (counterparty: Uint8Array) => Promise<Uint8Array>,
    channelBalance?: ChannelBalance<ConcreteConnector>,
    sign?: (channelBalance: ChannelBalance<ConcreteConnector>) => Promise<Uint8Array>
  ): Promise<Channel>

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
  function getAll<T, R, ConcreteConnector extends HoprCoreConnector>(
    coreConnector: ConcreteConnector,
    onData: (channel: Channel, ...props: any[]) => Promise<T>,
    onEnd: (promises: Promise<T>[], ...props: any[]) => R,
  ): Promise<R>

  /**
   * Fetches all channel instances from the database and initiates a settlement on
   * each of them.
   * @param props additional arguments
   */
  function closeChannels<ConcreteConnector extends HoprCoreConnector>(coreConnector: ConcreteConnector): Promise<Balance>

  /**
   * Handles a channel opening request.
   * @notice Takes the `coreConnector` instance and returns an async iterable duplex stream.
   * @param coreConnector coreConnector instance
   */
  function handleOpeningRequest<ConcreteConnector extends HoprCoreConnector>(coreConnector: ConcreteConnector, ...props: any[]): (source: AsyncIterable<Uint8Array>) => AsyncIterator<Uint8Array>
}

declare interface Channel {
  readonly channelId: Promise<Hash>

  readonly settlementWindow: Promise<Moment>

  readonly state: Promise<ChannelType>

  readonly balance_a: Promise<Balance>

  readonly balance: Promise<Balance>

  readonly currentBalance: Promise<Balance>

  readonly currentBalanceOfCounterparty: Promise<Balance>

  readonly ticket: typeof Ticket

  readonly counterparty: AccountId

  readonly offChainCounterparty: Uint8Array

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
