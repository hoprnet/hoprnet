import BN from 'bn.js'

declare interface BalanceStatic {
  readonly SIZE: number

  /**
   * Abbreviation of the currency, e.g. `HOPR`
   */
  readonly SYMBOL: string

  /**
   * Decimals of the currency, e.g. 18
   */
  readonly DECIMALS: number

  new (balance: BN | number, ...props: any[]): Balance
}

declare interface Balance extends BN {
  toU8a(): Uint8Array

  // Readable version of the balance
  toFormattedString(): string
}

declare var Balance: BalanceStatic

export default Balance
