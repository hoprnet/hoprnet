import type { Types } from '@hoprnet/hopr-core-connector-interface'
import BN from 'bn.js'
import { u8aSplit, serializeToU8a } from '@hoprnet/hopr-utils'
import Address from './accountId'
import Public from './public'
import Hash from './hash'
import { UINT256 } from './solidity'

class Account implements Types.Account {
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
    return new Account(address, publicKey, secret, counter)
  }

  public serialize(): Uint8Array {
    return serializeToU8a([
      [this.publicKey, Public.SIZE],
      [this.secret, Hash.SIZE],
      [this.counter.toBuffer(), 1]
    ])
  }

  public isInitialized(): boolean {
    return typeof this.publicKey !== 'undefined'
  }
}

export { Account }
