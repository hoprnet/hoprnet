import { LevelUp } from 'levelup'
import BN from 'bn.js'

import Utils from './utils'
import Channel from './channel'
import Constructors, {Types} from './types'

export { Utils, Types, Channel }

export default class HoprCoreConnector {
  private constructor(...props: any[])

  readonly started: boolean
  readonly self: any
  readonly db: LevelUp
  readonly nonce: Promise<number>

  static readonly utils: Utils
  static readonly channel: Channel
  static readonly types: Constructors

  /**
   * Creates an uninitialised instance.
   *
   * @param db database instance
   */
  static create(db: LevelUp, keyPair: any, uri?: string): Promise<HoprCoreConnector>

  start(): Promise<void>

  stop(): Promise<void>

  initOnchainValues(nonce?: number): Promise<void>

  checkFreeBalance(newBalance: any): Promise<void>
}

