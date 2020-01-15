import { LevelUp } from 'levelup'
import BN from 'bn.js'

import IUtils from './utils'
import Channel from './channel'
import Constructors, { Types } from './types'

export { IUtils, Types, Channel }

declare class HoprCoreConnector {
  private constructor(...props: any[])

  readonly started: boolean
  readonly self: any
  readonly db: LevelUp
  readonly nonce: Promise<number>

  start(): Promise<void>

  stop(): Promise<void>

  initOnchainValues(nonce?: number): Promise<void>

  checkFreeBalance(newBalance: any): Promise<void>
}

declare namespace HoprCoreConnector {
  /**
   * Creates an uninitialised instance.
   *
   * @param db database instance
   */
  function create(db: LevelUp, keyPair: any, uri?: string): Promise<HoprCoreConnector>

  const Utils: IUtils
  const channel: Channel
  const types: Constructors
}
