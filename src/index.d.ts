import { LevelUp } from 'levelup'
import BN from 'bn.js'

import Utils from './utils'
import Channel from './channel'
import Types from './types'

declare class HoprCoreConnector {
  private constructor(...props: any[])

  started: boolean

  readonly self: any

  readonly db: LevelUp

  readonly utils: Utils

  readonly channel: Channel

  readonly types: Types

  nonce: Promise<number>

  start(): Promise<void>

  stop(): Promise<void>

  initOnchainValues(nonce?: number): Promise<void>

  checkFreeBalance(newBalance: any): Promise<void>
}

declare interface HoprCoreConnectorStatic {
  /**
   * Creates an uninitialised instance.
   *
   * @param db database instance
   */
  create(db: LevelUp, keyPair: any, uri?: string): Promise<HoprCoreConnector>

  readonly types: Types
}

export { HoprCoreConnector }


export default HoprCoreConnectorStatic
