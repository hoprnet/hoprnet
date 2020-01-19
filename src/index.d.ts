import { LevelUp } from 'levelup'
import BN from 'bn.js'

import Utils from './utils'
import { ChannelClass } from './channel'
import Types, { TypeClasses } from './types'
import DbKeys from './dbKeys'
import Constants from './constants'

export { Utils, DbKeys, TypeClasses, ChannelClass, Constants }

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

  /**
   * (Static) utils to use in the connector module
   */
  utils: Utils

  /**
   * Export creator for all Types used on-chain.
   */
  types: Types

  /**
   * Export keys under which our data gets stored in the database.
   */
  dbKeys: DbKeys

  /**
   * Export chain-specific constants.
   */
  constants: Constants

  channel: {
    /**
     * Creates a Channel instance from the database.
     * @param props additional arguments
     */
    create(...props: any[]): Promise<ChannelClass>

    /**
     * Opens a new payment channel and initializes the on-chain data.
     * @param amount how much should be staked
     * @param signature a signature over channel state
     * @param props additional arguments
     */
    open(amount: TypeClasses.Balance, signature: Promise<Uint8Array>, ...props: any[]): Promise<ChannelClass>

    /**
     * Fetches all channel instances from the database and applies first `onData` and
     * then `onEnd` on the received nstances.
     * @param onData applied on all channel instances
     * @param onEnd composes at the end the received data
     */
    getAll<T, R>(onData: (channel: ChannelClass, ...props: any[]) => T, onEnd: (promises: Promise<T>[], ...props: any[]) => R, ...props: any[]): Promise<R>

    /**
     * Fetches all channel instances from the database and initiates a settlement on
     * each of them.
     * @param props additional arguments
     */
    closeChannels(...props: any[]): Promise<TypeClasses.Balance>
  }
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
}
