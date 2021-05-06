import { utils, ethers } from 'ethers'
import BN from 'bn.js'
import { publicKeyConvert, publicKeyCreate, ecdsaSign, ecdsaVerify } from 'secp256k1'
import { moveDecimalPoint } from '../math'
import { u8aToHex, u8aEquals, stringToU8a, u8aConcat, serializeToU8a, u8aToNumber, u8aSplit } from '../u8a'
import { ADDRESS_LENGTH, HASH_LENGTH, SIGNATURE_LENGTH, SIGNATURE_RECOVERY_LENGTH } from '../constants'

export class PublicKey {
  // @TODO use uncompressed public key internally
  constructor(private arr: Uint8Array) {
    if (arr.length !== PublicKey.SIZE) {
      throw new Error('Incorrect size Uint8Array for compressed public key')
    }
  }

  static fromPrivKey(privKey: Uint8Array): PublicKey {
    if (privKey.length !== 32) {
      throw new Error('Incorrect size Uint8Array for private key')
    }

    return new PublicKey(publicKeyCreate(privKey, true))
  }

  static fromUncompressedPubKey(arr: Uint8Array) {
    if (arr.length !== 65) {
      throw new Error('Incorrect size Uint8Array for uncompressed public key')
    }

    return new PublicKey(publicKeyConvert(arr, true))
  }

  toAddress(): Address {
    return new Address(Hash.create(publicKeyConvert(this.arr, false).slice(1)).serialize().slice(12))
  }

  toUncompressedPubKeyHex(): string {
    // Needed in only a few cases for interacting with secp256k1
    return u8aToHex(publicKeyConvert(this.arr, false).slice(1))
  }

  static fromString(str: string): PublicKey {
    return new PublicKey(stringToU8a(str))
  }

  static get SIZE(): number {
    return 33
  }

  serialize() {
    return this.arr
  }

  toHex(): string {
    return u8aToHex(this.arr)
  }

  eq(b: PublicKey) {
    return u8aEquals(this.arr, b.serialize())
  }
}

export class Address {
  constructor(private arr: Uint8Array) {
    if (arr.length !== Address.SIZE) {
      throw new Error('Incorrect size Uint8Array for address')
    } else if (!ethers.utils.isAddress(u8aToHex(arr))) {
      throw new Error('Incorrect Uint8Array for address')
    }
  }

  static get SIZE(): number {
    return ADDRESS_LENGTH
  }

  static fromString(str: string): Address {
    return new Address(stringToU8a(str))
  }

  static deserialize(arr: Uint8Array) {
    return new Address(arr)
  }

  serialize() {
    return this.arr
  }

  toHex(): string {
    return ethers.utils.getAddress(u8aToHex(this.arr, false))
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

  static createMock(): Address {
    return Address.fromString('0xCf7Ed3AccA5a467e9e704C703E8D87F634fB0Fc9')
  }
}

export class Hash {
  constructor(private arr: Uint8Array) {
    if (arr.length !== Hash.SIZE) {
      throw new Error('Incorrect size Uint8Array for hash')
    }
  }

  static SIZE = HASH_LENGTH

  static create(...inputs: Uint8Array[]) {
    return new Hash(utils.arrayify(utils.keccak256(u8aConcat(...inputs))))
  }

  static deserialize(arr: Uint8Array) {
    return new Hash(arr)
  }

  serialize(): Uint8Array {
    return this.arr
  }

  eq(b: Hash) {
    return u8aEquals(this.arr, b.serialize())
  }

  toHex(): string {
    return u8aToHex(this.arr)
  }

  clone(): Hash {
    return new Hash(this.arr.slice())
  }

  hash(): Hash {
    // Sometimes we double hash.
    return Hash.create(this.serialize())
  }
}

export class Signature {
  constructor(readonly signature: Uint8Array, readonly recovery: number) {
    if (signature.length !== SIGNATURE_LENGTH) {
      throw new Error('Incorrect size Uint8Array for signature')
    }
  }

  static deserialize(arr: Uint8Array): Signature {
    const [s, r] = u8aSplit(arr, [SIGNATURE_LENGTH, SIGNATURE_RECOVERY_LENGTH])
    return new Signature(s, u8aToNumber(r) as number)
  }

  static create(msg: Uint8Array, privKey: Uint8Array): Signature {
    const result = ecdsaSign(msg, privKey)
    return new Signature(result.signature, result.recid)
  }

  serialize(): Uint8Array {
    return serializeToU8a([
      [this.signature, SIGNATURE_LENGTH],
      [Uint8Array.of(this.recovery), SIGNATURE_RECOVERY_LENGTH]
    ])
  }

  verify(msg: Uint8Array, pubKey: PublicKey): boolean {
    return ecdsaVerify(this.signature, msg, pubKey.serialize())
  }

  static SIZE = SIGNATURE_LENGTH + SIGNATURE_RECOVERY_LENGTH
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

  static deserialize(arr: Uint8Array) {
    return new Balance(new BN(arr))
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

export class NativeBalance {
  constructor(private bn: BN) {}

  static get SYMBOL(): string {
    return `xDAI`
  }

  static get DECIMALS(): number {
    return 18
  }

  static deserialize(arr: Uint8Array) {
    return new NativeBalance(new BN(arr))
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
