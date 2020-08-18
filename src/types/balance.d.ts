import BN from 'bn.js'

declare namespace Balance {
  const SIZE: number

  /**
   * Abbreviation of the currency, e.g. `HOPR`
   */
  const SYMBOL: string

  /**
   * Decimals of the currency, e.g. 18
   */
  const DECIMALS: number
}
declare interface Balance extends BN {
  new (balance: BN, ...props: any[]): Balance
}

export default Balance
