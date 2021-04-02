import type { Types as Interfaces } from '@hoprnet/hopr-core-connector-interface'
import BN from 'bn.js'
import { u8aSplit, serializeToU8a } from '@hoprnet/hopr-utils'
import { Address } from '.' // TODO: cyclic dep
import { PublicKey } from '..'
import { Hash } from '.'
import { UINT256 } from './solidity'

class AccountEntry implements Interfaces.AccountEntry {
  constructor(
    public readonly address: Address,
    public readonly publicKey?: PublicKey,
    public readonly secret?: Hash,
    public readonly counter?: BN
  ) {}

  static get SIZE(): number {
    return Address.SIZE + PublicKey.SIZE + Hash.SIZE + UINT256.SIZE
  }

  static deserialize(arr: Uint8Array) {
    const [a, b, c, d] = u8aSplit(arr, [Address.SIZE, PublicKey.SIZE, Hash.SIZE, UINT256.SIZE])
    const address = new Address(a)
    const publicKey = new PublicKey(b)
    const secret = new Hash(c)
    const counter = new BN(d)
    return new AccountEntry(address, publicKey, secret, counter)
  }

  public serialize(): Uint8Array {
    const publicKey = this.publicKey || new PublicKey(new Uint8Array({ length: PublicKey.SIZE }))
    const secret = this.secret || new Hash(new Uint8Array(Hash.SIZE))
    const counter = this.counter ? new UINT256(this.counter).serialize() : UINT256.fromString('0').serialize()

    return serializeToU8a([
      [this.address.serialize(), Address.SIZE],
      [publicKey.serialize(), PublicKey.SIZE],
      [secret.serialize(), Hash.SIZE],
      [counter, UINT256.SIZE]
    ])
  }

  public isInitialized(): boolean {
    return typeof this.publicKey !== 'undefined'
  }
}

export default AccountEntry
