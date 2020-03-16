import BN from 'bn.js'

class BNE extends BN {
  toU8a(length?: number) {
    return new Uint8Array(this.toBuffer('be', length))
  }
}

export default BNE
