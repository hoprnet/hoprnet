import BN from 'bn.js'

declare namespace NativeBalance {
  const SIZE: number

  /**
   * Abbreviation of the currency, e.g. `ETH`
   */
  const SYMBOL: string

  /**
   * Decimals of the currency, e.g. 18
   */
  const DECIMALS: number
}
declare interface NativeBalance extends BN {
  new (nativeBalance: BN, ...props: any[]): NativeBalance
}

export default NativeBalance
