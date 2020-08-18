import BN from 'bn.js'

declare interface NativeBalanceStatic {
  readonly SIZE: number

  /**
   * Abbreviation of the currency, e.g. `ETH`
   */
  readonly SYMBOL: string

  /**
   * Decimals of the currency, e.g. 18
   */
  readonly DECIMALS: number

  new (nativeBalance: BN, ...props: any[]): NativeBalance
}

declare interface NativeBalance extends BN {}

declare var NativeBalance: NativeBalanceStatic

export default NativeBalance
