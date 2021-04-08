import type { LevelUp } from 'levelup'
import type Channel, { SubmitTicketResponse } from './channel'
import type Indexer, { RoutingChannel } from './indexer'

export type Currencies = 'NATIVE' | 'HOPR'

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
}

declare interface HoprCoreConnector {
  readonly started: boolean

  readonly account: {
    /**
     * Returns the current (token) balance of the account associated with this node.
     */
    getBalance: (useCache?: boolean) => Promise<Types.Balance>
    /**
     * Returns the current native balance (ex: ETH) of the account associated with this node.
     */
    getNativeBalance: (useCache?: boolean) => Promise<Types.NativeBalance>

    keys: {
      onChain: {
        privKey: Uint8Array
        pubKey: PublicKey
      }
      offChain: {
        privKey: Uint8Array
        pubKey: PublicKey
      }
    }

    /**
     * Check whether the given ticket is winning.
     *
     * If the ticket is a win, the preImage is stored into the given acknowledged
     * ticket and its preImage will be used to check whether the next ticket is a
     * win.
     * @param ticket the ticket to check
     */
    acknowledge(ticket: Ticket, response: Hash): Promise<Acknowledgement | undefined>
  }

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

  /**
   * Encapsulates payment channel between nodes.
   */
  readonly channel: typeof Channel

  /**
   * Returns an instance of Indexer.
   */
  readonly indexer: Indexer
}

export { Constants, Channel, SubmitTicketResponse, Indexer, RoutingChannel, HoprCoreConnectorStatic }
export * from './types'
export default HoprCoreConnector
