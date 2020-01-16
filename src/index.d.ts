import { LevelUp } from 'levelup'
import BN from 'bn.js'

import Utils from './utils'
import Channel, { ChannelClass } from './channel'
import Types, { TypeClasses } from './types'

export { Utils, TypeClasses, Channel, ChannelClass }

export interface HoprCoreConnectorClass {
  readonly started: boolean
  readonly self: any
  readonly db: LevelUp
  readonly nonce: Promise<number>

  start(): Promise<void>

  stop(): Promise<void>

  initOnchainValues(nonce?: number): Promise<void>

  checkFreeBalance(newBalance: any): Promise<void>
}

export default interface HoprCoreConnector {
  /**
   * Creates an uninitialised instance.
   *
   * @param db database instance
   */
  create(db: LevelUp, keyPair: any, uri?: string): Promise<HoprCoreConnectorClass>

  utils: Utils
  channel: Channel
  types: Types
}
