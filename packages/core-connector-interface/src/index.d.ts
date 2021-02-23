import type { LevelUp } from 'levelup'
import type * as Utils from './utils'
import type Channel from './channel'
import type * as Types from './types'
import type * as DbKeys from './dbKeys'
import type * as Constants from './constants'
import type Indexer, { RoutingChannel, ChannelUpdate } from './indexer'

export type Currencies = 'NATIVE' | 'HOPR'

export type ValidateResponse =
  | { status: 'SUCCESS'; ticket: AcknowledgedTicket }
  | { status: 'E_TICKET_FAILED' }
  | { status: 'E_CHALLENGE' }

export type RedeemStatus = { status: 'SUCCESS' } | { status: 'E_NO_GAS' } | { status: 'E_ALREADY_SUBMITTED' }

declare interface HoprCoreConnectorStatic {
  /**
   * Creates an uninitialised instance.
   *
   * @param db database instance
   * @param seed that is used to derive that on-chain identity
   * @param options.provider URI that is used to connect to the blockchain
   * @param options.debug run connector in debug mode if set to true
   */
  create(db: LevelUp, seed: Uint8Array, options?: { provider?: string; debug?: boolean }): Promise<HoprCoreConnector>

  readonly constants: typeof Constants
}

declare interface HoprCoreConnector {
  readonly started: boolean

  readonly account: {
    /**
     * Returns the current (token) balance of the account associated with this node.
     */
    balance: Promise<Types.Balance>
    /**
     * Returns the current native balance (ex: ETH) of the account associated with this node.
     */
    nativeBalance: Promise<Types.NativeBalance>
    /**
     * Returns the current value of the reset counter
     */
    ticketEpoch: Promise<Types.TicketEpoch>
    /**
     * Returns the accounts address
     */
    address: Promise<Types.AccountId>
    /**
     * The accounts nonce.
     */
    nonce: Promise<number>
    /**
     * The accounts keys:
     */
    keys: {
      onChain: {
        privKey: Uint8Array
        pubKey: Uint8Array
      }
      offChain: {
        privKey: Uint8Array
        pubKey: Uint8Array
      }
    }
  }

  readonly db: LevelUp

  /**
   * Initialises the connector, e.g. connect to a blockchain node.
   */
  start(): Promise<HoprCoreConnector>

  /**
   * Stops the connector, e.g. disconnect from a blockchain node and save all
   * relevant state properties.
   */
  stop(): Promise<void>

  /**
   * Initializes the on-chain values of our account.
   * @param nonce optional specify nonce of the account to run multiple queries simultaneously
   */
  initOnchainValues(nonce?: number): Promise<void>

  /**
   * Withdraw the connector's native currency or HOPR tokens.
   * @param currency specify currency to withdraw
   * @param recipient specify the recipient who will receive the native currency or HOPR tokens
   * @param amount specify the amount that will be withdrawn
   */
  withdraw(currency: Currencies, recipient: string, amount: string): Promise<string>

  hexAccountAddress(): Promise<string>

  smartContractInfo(): string

  validateTicket(ticket: UnacknowledgedTicket, response: Hash): Promise<AcknowledgedTicket | ValidateFailure>

  redeemTicket(ticket: AcknowledgedTicket): Promise<RedeemStatus>

  /**
   * (Static) utils to use in the connector module
   */
  readonly utils: typeof Utils

  /**
   * Export creator for all Types used on-chain.
   */
  readonly types: typeof Types

  /**
   * Export keys under which our data gets stored in the database.
   */
  readonly dbKeys: typeof DbKeys

  /**
   * Export chain-specific constants.
   */
  readonly constants: typeof Constants

  /**
   * Encapsulates payment channel between nodes.
   */
  readonly channel: typeof Channel

  /**
   * Returns an instance of Indexer.
   */
  readonly indexer: Indexer

  /**
   * Returns unique information about the connector.
   */
  readonly describe?: any
}

declare var HoprCoreConnector: HoprCoreConnectorStatic

export {
  Utils,
  Types,
  DbKeys,
  Constants,
  Channel,
  Indexer,
  RoutingChannel,
  ChannelUpdate,
  HoprCoreConnectorStatic,
  AcknowledgedTicket,
  UnacknowledgedTicket,
  SignedTicket,
  Ticket,
  Hash
}

export default HoprCoreConnector
