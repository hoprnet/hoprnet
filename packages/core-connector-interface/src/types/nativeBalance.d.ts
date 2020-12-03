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

  new (nativeBalance: BN | number, ...props: any[]): NativeBalance

  // Readable version of the balance
  toFormattedString(): string
}

declare interface NativeBalance extends BN {
  toU8a(): Uint8Array
}

declare var NativeBalance: NativeBalanceStatic

export default NativeBalance
