import { AccountId, Balance, Channel as ChannelType, ChannelBalance, Hash, Moment, Signature, SignedTicket, Ticket, Types, SignedChannel } from './types'
import { HoprCoreConnectorInstance } from '.'

declare interface ChannelInstance {
  readonly channelId: Promise<Hash.Instance>

  readonly settlementWindow: Promise<Moment.Instance>

  readonly state: Promise<ChannelType.Instance>

  readonly balance_a: Promise<Balance.Instance>

  readonly balance: Promise<Balance.Instance>

  readonly currentBalance: Promise<Balance.Instance>

  readonly currentBalanceOfCounterparty: Promise<Balance.Instance>

  readonly ticket: Ticket.Static<any, any>

  readonly counterparty: Types.AccountId

  /**
   * Initiates a settlement for this channel.
   */
  initiateSettlement(): Promise<void>

  /**
   * Fetches all unresolved, previous challenges from the database that
   * have occured in this channel.
   */
  getPreviousChallenges(): Promise<Hash.Instance>
}

declare interface Channel<ConcreteConnector extends HoprCoreConnectorInstance> {
  /**
   * Creates a Channel instance from the database.
   * @param counterparty AccountId of the counterparty
   * @param props additional arguments
   */
  create(
    coreConnector: ConcreteConnector,
    offChainCounterparty: Uint8Array,
    getOnChainPublicKey: (counterparty: Uint8Array) => Promise<Uint8Array>,
    channelBalance?: ChannelBalance.Instance,
    sign?: (channelBalance: ChannelBalance.Instance) => Promise<Uint8Array>
  ): Promise<ChannelInstance>

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
  getAll<T, R>(
    coreConnector: ConcreteConnector,
    onData: (channel: ChannelInstance, ...props: any[]) => Promise<T>,
    onEnd: (promises: Promise<T>[], ...props: any[]) => R,
    ...props: any[]
  ): Promise<R>

  /**
   * Fetches all channel instances from the database and initiates a settlement on
   * each of them.
   * @param props additional arguments
   */
  closeChannels(coreConnector: ConcreteConnector, ...props: any[]): Promise<Balance.Instance>

  /**
   * Handles a channel opening request.
   * @notice Takes the `coreConnector` instance and returns an async iterable duplex stream.
   * @param coreConnector coreConnector instance
   */
  handleOpeningRequest(coreConnector: ConcreteConnector, ...props: any[]): (source: AsyncIterable<Uint8Array>) => AsyncIterator<Uint8Array>
}

export { ChannelInstance }

export default Channel
