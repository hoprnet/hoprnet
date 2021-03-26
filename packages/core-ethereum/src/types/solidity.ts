import type { Types as Interfaces } from '@hoprnet/hopr-core-connector-interface'
import BN from 'bn.js'

class UINT256 implements Interfaces.UINT256 {
  constructor(private bn: BN) {}

  public toBN(): BN {
    return this.bn
  }
  public serialize(): Uint8Array {
    return new Uint8Array(this.bn.toBuffer('be', UINT256.SIZE))
  }

  static fromString(str: string) {
    return new UINT256(new BN(str))
  }

  static get SIZE(): number {
    return 32
  }
}

export { UINT256 }
