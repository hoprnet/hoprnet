import AcknowledgedTicket from './types/acknowledgedTicket'
import type {
  AccountId,
  Balance,
  Channel as ChannelType,
  ChannelBalance,
  Hash,
  Moment,
  Public,
  Signature,
  SignedChannel,
  SignedTicket
} from './types'

declare interface ChannelStatic {
  /**
   * Creates a Channel instance from the database.
   * @param counterparty AccountId of the counterparty
   * @param props additional arguments
   */
  create(
    offChainCounterparty: Uint8Array,
    getOnChainPublicKey: (counterparty: Uint8Array) => Promise<Public>,
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
  createDummyChannelTicket(counterParty: AccountId, challenge: Hash, ...props: any[]): Promise<SignedTicket>

  /**
   * Checks whether the channel exists on-chain and off-chain, i.e. in our database.
   * Returns `true` if the channel exists on-chain AND off-chain.
   * @param counterparty AccountId of the counterparty
   */
  isOpen(counterparty: AccountId): Promise<boolean>

  /**
   * Opens a new payment channel and initializes the on-chain data.
   * @param amount how much should be staked
   * @param signature a signature over channel state
   * @param props additional arguments
   */
  // open(coreConnector: any, amount: Balance.Instance, signature: Promise<Uint8Array>, ...props: any[]): Promise<ChannelInstance>

  /**
   * Fetches all channel instances from the database and applies first `onData` and
   * then `onEnd` on the received instances.
   * @param onData applied on all channel instances
   * @param onEnd composes at the end the received data
   */
  getAll<T, R>(
    onData: (channel: Channel, ...props: any[]) => Promise<T>,
    onEnd: (promises: Promise<T>[], ...props: any[]) => R
  ): Promise<R>

  /**
   * Fetches all channel instances from the database and initiates a settlement on
   * each of them.
   * @param props additional arguments
   */
  closeChannels(): Promise<Balance>

  /**
   * Increases the balance of the payment channel with the given counterparty
   * by the given amount
   * @param counterParty the counterparty of the channel
   * @param amount the amount of tokens to put into the payment channel
   */
  increaseFunds(counterParty: AccountId, amount: Balance): Promise<void>

  /**
   * Handles a channel opening request.
   * @notice Takes the `coreConnector` instance and returns an async iterable duplex stream.
   * @param coreConnector coreConnector instance
   */
  handleOpeningRequest(source: AsyncIterable<Uint8Array>): AsyncIterable<Uint8Array>

  /**
   * Create a signedChannel instance.
   * @param arr array containing a signedChannel
   * @param struct desired content of the signedChannel
   */
  createSignedChannel(
    arr?: {
      bytes: ArrayBuffer
      offset: number
    },
    struct?: {
      channel: ChannelType
      signature?: Signature
    }
  ): Promise<SignedChannel>

  tickets: {
    /**
     * Submits a signed ticket to the blockchain.
     * @param signedTicket a signed ticket
     * @param secretA a signed ticket
     * @param secretB a signed ticket
     */
    submit(
      ticket: AcknowledgedTicket,
      ticketIndex: Uint8Array
    ): Promise<
      | {
          status: 'SUCCESS'
          receipt: string
        }
      | {
          status: 'FAILURE'
          message: string
        }
      | {
          status: 'ERROR'
          error: Error | string
        }
    >
  }
}

declare interface Channel {
  // Id of the channel
  readonly channelId: Promise<Hash>

  // Timestamp once the channel can be settled
  readonly settlementWindow: Promise<Moment>

  // Current status of the channel
  readonly status: Promise<'UNINITIALISED' | 'FUNDING' | 'OPEN' | 'PENDING'>

  // Current state of the channel, i.e. `FUNDED` with `1 HOPR / 3 HOPR`
  readonly state: Promise<ChannelType>

  // Current balance of partyA
  readonly balance_a: Promise<Balance>

  // Current total balance (sum of balance_a and balance_b)
  readonly balance: Promise<Balance>

  readonly currentBalance: Promise<Balance>

  readonly currentBalanceOfCounterparty: Promise<Balance>

  readonly ticket: {
    /**
     * Constructs a ticket to use in a probabilistic payment channel.
     * @param amount amount of funds to include
     * @param challenge a challenge that has to be solved be the redeemer
     * @param winProb probability for the generated ticket to be a win
     */
    create(
      amount: Balance,
      challenge: Hash,
      winProb: number | undefined,
      arr?: {
        bytes: ArrayBuffer
        offset: number
      }
    ): Promise<SignedTicket>

    /**
     * Checks a previously issued ticket for its validity.
     * @param signedTicket a previously issued ticket to check
     * @param props additional arguments
     */
    verify(signedTicket: SignedTicket): Promise<boolean>

    /**
     * BIG TODO
     * Aggregate previously issued tickets. Still under active development!
     * @param tickets array of tickets to aggregate
     * @param props additional arguments
     */
    // aggregate(channel: any, tickets: Ticket[], ...props: any[]): Promise<Ticket>
  }

  readonly counterparty: AccountId

  readonly offChainCounterparty: Promise<Uint8Array>

  /**
   * Initiates a settlement for this channel.
   */
  initiateSettlement(): Promise<string>
}

declare var Channel: ChannelStatic

export default Channel
