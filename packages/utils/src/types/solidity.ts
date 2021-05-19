import BN from 'bn.js'

class UINT256 {
  constructor(private bn: BN) {}

  public toBN(): BN {
    return this.bn
  }

  static deserialize(arr: Uint8Array): UINT256 {
    return new UINT256(new BN(arr))
  }

  public serialize(): Uint8Array {
    return new Uint8Array(this.bn.toBuffer('be', UINT256.SIZE))
  }

  static fromString(str: string): UINT256 {
    return new UINT256(new BN(str))
  }

  static fromProbability(n: number): UINT256 {
    if (n > 1) {
      throw Error('Probability input cannot be larger than 1')
    }

    // Represent number as a decimal number string, then slice
    // the integer part `0.` and compute the length of rational
    // part which gives the number of decimal places required to
    // represent the number.
    const decimalPlaces = n.toString().replace(/[0]{0,}\./, '').length

    const divisor = 10 ** decimalPlaces

    return new UINT256(new BN(new Uint8Array(32).fill(0xff)).muln(n * divisor).divn(divisor))
  }

  static get SIZE(): number {
    return 32
  }
}

export { UINT256 }
