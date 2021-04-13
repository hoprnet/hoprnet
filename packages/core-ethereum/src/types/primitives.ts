import { ADDRESS_LENGTH, HASH_LENGTH, SIGNATURE_LENGTH, SIGNATURE_RECOVERY_LENGTH } from '../constants'
import { u8aEquals, moveDecimalPoint } from '@hoprnet/hopr-utils'
import { ethers } from 'ethers'
import BN from 'bn.js'
import { serializeToU8a, u8aSplit, u8aToNumber } from '@hoprnet/hopr-utils'

const { utils } = ethers

export class Address {
  constructor(private arr: Uint8Array) {}

  static get SIZE(): number {
    return ADDRESS_LENGTH
  }

  static fromString(str: string): Address {
    if (!utils.isAddress(str)) throw Error(`String ${str} is not an address`)
    return new Address(utils.arrayify(str))
  }

  serialize() {
    return this.arr
  }

  toHex(): string {
    return utils.getAddress(utils.hexlify(this.arr))
  }

  eq(b: Address) {
    return u8aEquals(this.arr, b.serialize())
  }

  compare(b: Address): number {
    return Buffer.compare(this.serialize(), b.serialize())
  }

  lt(b: Address): boolean {
    return this.compare(b) < 0
  }

  sortPair(b: Address): [Address, Address] {
    return this.lt(b) ? [this, b] : [b, this]
  }
}

export class Balance {
  constructor(private bn: BN) {}

  static get SYMBOL(): string {
    return `HOPR`
  }

  static get DECIMALS(): number {
    return 18
  }

  public toBN(): BN {
    return this.bn
  }

  public serialize(): Uint8Array {
    return new Uint8Array(this.bn.toBuffer('be', Balance.SIZE))
  }

  public toFormattedString(): string {
    return moveDecimalPoint(this.bn.toString(), Balance.DECIMALS * -1) + ' ' + Balance.SYMBOL
  }

  static get SIZE(): number {
    // Uint256
    return 32
  }
}

export class Hash {
  constructor(private arr: Uint8Array) {}

  static SIZE = HASH_LENGTH

  static create(msg: Uint8Array) {
    return new Hash(utils.arrayify(utils.keccak256(msg)))
  }

  static createChallenge(secretA: Uint8Array, secretB: Uint8Array): Hash {
    return Hash.create(utils.concat([secretA, secretB])).hash()
  }

  serialize(): Uint8Array {
    return this.arr
  }

  eq(b: Hash) {
    return u8aEquals(this.arr, b.serialize())
  }

  toHex(): string {
    return utils.hexlify(this.arr)
  }

  clone(): Hash {
    return new Hash(this.arr.slice())
  }

  hash(): Hash {
    // Sometimes we double hash.
    return Hash.create(this.serialize())
  }

  get length() {
    return this.arr.length
  }
}

export class NativeBalance {
  constructor(private bn: BN) {}

  static get SYMBOL(): string {
    return `xDAI`
  }

  static get DECIMALS(): number {
    return 18
  }

  public toBN(): BN {
    return this.bn
  }

  public serialize(): Uint8Array {
    return new Uint8Array(this.bn.toBuffer('be', NativeBalance.SIZE))
  }

  public toFormattedString(): string {
    return moveDecimalPoint(this.bn.toString(), NativeBalance.DECIMALS * -1) + ' ' + NativeBalance.SYMBOL
  }

  static get SIZE(): number {
    // Uint256
    return 32
  }
}

export class PublicKey {
  constructor(private arr: Uint8Array) {
    if (arr.length !== PublicKey.SIZE) {
      throw new Error('Incorrect size Uint8Array for compressed public key')
    }
  }

  static fromPrivKey(privKey: Uint8Array): PublicKey {
    // removes identifier
    return PublicKey.fromString(utils.computePublicKey(privKey, true))
  }

  // Needed when interacting with HoprChannels
  static fromUncompressedPubKey(pubkey: Uint8Array): PublicKey {
    // adds identifer
    const withIdentifier = ethers.utils.concat([new Uint8Array([4]), pubkey])
    return PublicKey.fromString(utils.computePublicKey(withIdentifier, true))
  }

  toAddress(): Address {
    return Address.fromString(utils.computeAddress(this.serialize()))
  }

  // Needed when interacting with HoprChannels
  toUncompressedPubKeyHex(): string {
    return utils.hexDataSlice(utils.computePublicKey(this.serialize(), false), 1)
  }

  static fromString(str: string): PublicKey {
    return new PublicKey(utils.arrayify(str))
  }

  static get SIZE(): number {
    return 33
  }

  serialize() {
    return this.arr
  }

  toHex(): string {
    return utils.hexlify(this.arr)
  }

  eq(b: PublicKey) {
    return u8aEquals(this.arr, b.serialize())
  }
}

export class Signature {
  constructor(readonly signature: Uint8Array, readonly recovery: number) {}

  static deserialize(arr: Uint8Array): Signature {
    const [s, r] = u8aSplit(arr, [SIGNATURE_LENGTH, SIGNATURE_RECOVERY_LENGTH])
    return new Signature(s, u8aToNumber(r) as number)
  }

  static create(msg: Uint8Array, privKey: Uint8Array): Signature {
    const signingKey = new ethers.utils.SigningKey(privKey)
    const result = signingKey.signDigest(msg)
    // get flattened signature
    const signatue = ethers.utils.joinSignature(result)
    return new Signature(utils.arrayify(signatue), result.recoveryParam)
  }

  serialize(): Uint8Array {
    return serializeToU8a([
      [this.signature, SIGNATURE_LENGTH],
      [Uint8Array.of(this.recovery), SIGNATURE_RECOVERY_LENGTH]
    ])
  }

  verify(msg: Uint8Array, pubKey: PublicKey): boolean {
    return u8aEquals(utils.arrayify(ethers.utils.recoverPublicKey(msg, this.signature)), pubKey.serialize())
  }

  static SIZE = SIGNATURE_LENGTH + SIGNATURE_RECOVERY_LENGTH
}
