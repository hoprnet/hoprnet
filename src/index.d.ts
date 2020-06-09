import type { LevelUp } from 'levelup'
import type * as Utils from './utils'
import type Channel from './channel'
import type * as Types from './types'
import type * as DbKeys from './dbKeys'
import type * as Constants from './constants'
import type Indexer from './indexer'
import type Tickets from './tickets'

declare namespace HoprCoreConnector {
  /**
   * Creates an uninitialised instance.
   *
   * @param db database instance
   * @param seed that is used to derive that on-chain identity
   * @param options.id Id of the demo account
   * @param options.uri URI that is used to connect to the blockchain
   */
  function create(db: LevelUp, seed?: Uint8Array, options?: { id?: number; provider?: string; debug?: boolean }): Promise<HoprCoreConnector>

  const constants: typeof Constants
}

declare interface HoprCoreConnector {
  readonly started: boolean
  readonly self: {
    privateKey: Uint8Array
    publicKey: Uint8Array
    onChainKeyPair: {
      privateKey?: Uint8Array
      publicKey?: Uint8Array
    }
  }
  readonly db: LevelUp
  readonly nonce: Promise<number>

  /**
   * Initialises the connector, e.g. connect to a blockchain node.
   */
  start(): Promise<void>

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
   * Returns the current balances of the account associated with this node.
   */
  accountBalance: Promise<Types.Balance>

  /**
   * Returns the current native balance (ex: ETH) of the account associated with this node.
   */
  accountNativeBalance: Promise<Types.NativeBalance>

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
   * Store and query tickets.
   */
  readonly tickets: typeof Tickets

  /**
   * Returns an instance of Indexer.
   */
  readonly indexer?: Indexer

  /**
   * Returns unique information about the connector.
   */
  readonly describe?: any
}

type HoprCoreConnectorStatic = typeof HoprCoreConnector

export { Utils, DbKeys, Types, Channel, Constants, Indexer, HoprCoreConnectorStatic }

export default HoprCoreConnector
