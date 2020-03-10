import { LevelUp } from 'levelup'
import BN from 'bn.js'

import Utils from './utils'
import Channel, { ChannelInstance } from './channel'
import TypeConstructors, { Types, AccountId, Balance, Ticket } from './types'
import DbKeys from './dbKeys'

import Constants from './constants'

declare interface HoprCoreConnectorInstance {
  readonly started: boolean
  readonly self: any
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
  accountBalance: Promise<Balance.Instance>

  /**
   * (Static) utils to use in the connector module
   */
  readonly utils: Utils

  /**
   * Export creator for all Types used on-chain.
   */
  readonly types: TypeConstructors

  /**
   * Export keys under which our data gets stored in the database.
   */
  readonly dbKeys: DbKeys

  /**
   * Export chain-specific constants.
   */
  readonly constants: Constants

  /**
   * Encapsulates payment channel between nodes.
   */
  readonly channel: Channel<any>
}

declare interface HoprCoreConnector {
  /**
   * Creates an uninitialised instance.
   *
   * @param db database instance
   * @param seed that is used to derive that on-chain identity
   * @param options.id Id of the demo account
   * @param options.uri URI that is used to connect to the blockchain
   */
  create(db: LevelUp, seed?: Uint8Array, options?: { id?: number; provider?: string }): Promise<HoprCoreConnectorInstance>

  readonly constants: Constants
}

export { HoprCoreConnectorInstance, Utils, DbKeys, Types, ChannelInstance, Constants, Ticket }

export default HoprCoreConnector
