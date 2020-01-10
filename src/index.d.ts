import { LevelUp } from 'levelup'
import BN from 'bn.js'

export { default as utils } from './utils'
export { default as Channel, ChannelClass, ChannelStatic } from './channel'
export * from './types'

declare class HoprCoreConnectorClass {
  private constructor(...props: any[])

  started: boolean

  readonly self: any

  readonly db: LevelUp

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
  create(db: LevelUp, keyPair: any, uri?: string): Promise<HoprCoreConnectorClass>
}

export { HoprCoreConnectorClass, HoprCoreConnectorStatic }

declare const HoprCoreConnector: HoprCoreConnectorClass & HoprCoreConnectorStatic

export default HoprCoreConnector
