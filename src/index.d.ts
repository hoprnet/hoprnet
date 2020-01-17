import { LevelUp } from 'levelup'
import BN from 'bn.js'

import Utils from './utils'
import Channel, { ChannelClass } from './channel'
import Types, { TypeClasses } from './types'
import DbKeys from './dbKeys'

export { Utils, TypeClasses, Channel, ChannelClass }

export interface HoprCoreConnectorClass {
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
   * Check whether our account possesses more than `newBalance` coin.
   * @param newBalance balance after update
   */
  checkFreeBalance(newBalance: any): Promise<void>

  /**
   * Creates and send a transaction that transfers `amount` coins to `to`.
   * @param to account of the recipient
   * @param amount how much to transfer
   * @param props additional arguments
   */
  transfer(to: TypeClasses.AccountId, amount: TypeClasses.Balance, ...props: any[]): Promise<void>
}

export default interface HoprCoreConnector {
  /**
   * Creates an uninitialised instance.
   *
   * @param db database instance
   * @param keyPair public key and private key of the account
   * @param uri URI of the blockchain node, e.g. `ws://localhost:9944`
   */
  create(db: LevelUp, keyPair: any, uri?: string): Promise<HoprCoreConnectorClass>

  /**
   * (Static) utils to use in the connector module
   */
  utils: Utils

  /**
   * Channel submodule that encapsulates all functionality relevant for
   * payment channels between to parties.
   */
  channel: Channel

  /**
   * Export creator for all Types used on-chain.
   */
  types: Types

  /**
   * Export keys under which our data gets stored in the database.
   */
  dbKeys: DbKeys
}
