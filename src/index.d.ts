import { LevelUp } from 'levelup'
import BN from 'bn.js'

import IUtils from './utils'
import Channel from './channel'
import Constructors, { Types } from './types'

export { IUtils, Types, Channel }

export declare class HoprCoreConnectorClass {
  protected constructor(...props: any[])

  readonly started: boolean
  readonly self: any
  readonly db: LevelUp
  readonly nonce: Promise<number>

  start(): Promise<void>

  stop(): Promise<void>

  initOnchainValues(nonce?: number): Promise<void>

  checkFreeBalance(newBalance: any): Promise<void>
}

declare interface HoprCoreConnector {
  /**
   * Creates an uninitialised instance.
   *
   * @param db database instance
   */
  create<T extends HoprCoreConnectorClass>(db: LevelUp, keyPair: any, uri?: string): Promise<T>

  utils: IUtils
  channel: Channel
  types: Constructors
}

export default HoprCoreConnector
