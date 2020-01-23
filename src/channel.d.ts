import { AccountId, Balance, Channel as ChannelType, Hash, Moment, SignedTicket, Ticket } from './types'
import { HoprCoreConnectorInstance } from '.'

declare interface ChannelInstance {
  readonly channelId: Promise<Hash.Instance>

  readonly settlementWindow: Promise<Moment.Instance>

  readonly state: Promise<ChannelType.Instance>

  readonly balance_a: Promise<Balance.Instance>

  readonly balance: Promise<Balance.Instance>

  readonly currentBalance: Promise<Balance.Instance>

  readonly currentBalanceOfCounterparty: Promise<Balance.Instance>

  readonly ticket: Ticket.Static

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

declare interface Channel {
  /**
   * Creates a Channel instance from the database.
   * @param counterparty AccountId of the counterparty
   * @param props additional arguments
   */
  create(coreConnector: any, counterparty: AccountId.Instance, ...props: any[]): Promise<ChannelInstance>

  /**
   * Opens a new payment channel and initializes the on-chain data.
   * @param amount how much should be staked
   * @param signature a signature over channel state
   * @param props additional arguments
   */
  open(coreConnector: any, amount: Balance.Instance, signature: Promise<Uint8Array>, ...props: any[]): Promise<ChannelInstance>

  /**
   * Fetches all channel instances from the database and applies first `onData` and
   * then `onEnd` on the received nstances.
   * @param onData applied on all channel instances
   * @param onEnd composes at the end the received data
   */
  getAll<T, R>(coreConnector: any, onData: (channel: ChannelInstance, ...props: any[]) => T, onEnd: (promises: Promise<T>[], ...props: any[]) => R, ...props: any[]): Promise<R>

  /**
   * Fetches all channel instances from the database and initiates a settlement on
   * each of them.
   * @param props additional arguments
   */
  closeChannels(coreConnector: any, ...props: any[]): Promise<Balance.Instance>
}

export { ChannelInstance }

export default Channel
