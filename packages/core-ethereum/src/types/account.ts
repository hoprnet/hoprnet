import type AccountId from './accountId'
import BN from 'bn.js'
import {} from '@hoprnet/hopr-utils'
import Public from './public'
import Hash from './hash'
import { UINT256 } from './solidity'
import { pubKeyToAccountId } from '../utils'

class Account {
  constructor(public readonly publicKey?: Public, public readonly secret?: Hash, public readonly counter?: BN) {}

  static deserialize(arr: Uint8Array) {
    const [a, b, c] = u8aSplit(arr, [Public.SIZE, Hash.SIZE, UINT256.SIZE])
    const publicKey = new Public(a)
    const secret = new Hash(b)
    const counter = new BN(c)
    return new Account(publicKey, secret, counter)
  }

  public serialize(): Uint8Array {
    return serializeToU8a([
      [this.publicKey, Public.SIZE],
      [this.secret, Hash.SIZE],
      [this.counter.toBuffer(), 1]
    ])
  }

  public async getAccountId(): Promise<AccountId> {
    return pubKeyToAccountId(this.publicKey)
  }
}

export { Account }
