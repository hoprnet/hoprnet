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

  static get SIZE(): number {
    return 32
  }
}

export { UINT256 }
