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

  static fromString(str: string): UINT256 {
    return new UINT256(new BN(str))
  }

  static fromInverseProbability(inverseProb: BN): UINT256 {
    if (inverseProb.isZero() || inverseProb.isNeg()) {
      throw Error('Inverse probability must be strictly greater than zero')
    }

    return new UINT256(new BN(new Uint8Array(UINT256.SIZE).fill(0xff)).div(inverseProb))
  }

  static get DUMMY_INVERSE_PROBABILITY(): UINT256 {
    return new UINT256(new BN(0))
  }

  static get SIZE(): number {
    return 32
  }
}

export { UINT256 }
