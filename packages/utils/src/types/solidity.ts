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

  public toHex(): string {
    return `0x${this.bn.toString('hex', 2 * UINT256.SIZE)}`
  }

  public eq(b: UINT256): boolean {
    return this.toBN().eq(b.toBN())
  }

  static fromString(str: string): UINT256 {
    return new UINT256(new BN(str))
  }

  static fromInverseProbability(inverseProb: BN): UINT256 {
    if (inverseProb.isNeg()) {
      throw Error('Inverse probability must not be negative')
    }

    const highestWinProb = new BN(new Uint8Array(UINT256.SIZE).fill(0xff))
    if (inverseProb.isZero()) return new UINT256(highestWinProb)
    return new UINT256(highestWinProb.div(inverseProb))
  }

  static get DUMMY_INVERSE_PROBABILITY(): UINT256 {
    return new UINT256(new BN(0))
  }

  static get SIZE(): number {
    return 32
  }
}

export { UINT256 }
