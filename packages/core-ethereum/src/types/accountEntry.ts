import type { Types as Interfaces } from '@hoprnet/hopr-core-connector-interface'
import BN from 'bn.js'
import { u8aSplit, serializeToU8a } from '@hoprnet/hopr-utils'
import { Address } from '.' // TODO: cyclic dep
import Public from './public'
import Hash from './hash'
import { UINT256 } from './solidity'

class AccountEntry implements Interfaces.AccountEntry {
  constructor(
    public readonly address: Address,
    public readonly publicKey?: Public,
    public readonly secret?: Hash,
    public readonly counter?: BN
  ) {}

  static get SIZE(): number {
    return Address.SIZE + Public.SIZE + Hash.SIZE + UINT256.SIZE
  }

  static deserialize(arr: Uint8Array) {
    const [a, b, c, d] = u8aSplit(arr, [Address.SIZE, Public.SIZE, Hash.SIZE, UINT256.SIZE])
    const address = new Address(a)
    const publicKey = new Public(b)
    const secret = new Hash(c)
    const counter = new BN(d)
    return new AccountEntry(address, publicKey, secret, counter)
  }

  public serialize(): Uint8Array {
    const publicKey = this.publicKey || new Public({ length: Public.SIZE })
    const secret = this.secret || new Hash({ length: Hash.SIZE })
    const counter = this.counter ? new UINT256(this.counter).serialize() : UINT256.fromString('0').serialize()

    return serializeToU8a([
      [this.address.serialize(), Address.SIZE],
      [publicKey, Public.SIZE],
      [secret, Hash.SIZE],
      [counter, UINT256.SIZE]
    ])
  }

  public isInitialized(): boolean {
    return typeof this.publicKey !== 'undefined'
  }
}

export default AccountEntry
