import createKeccakHash from 'keccak'
import { ADDRESS_LENGTH, HASH_LENGTH } from '../constants'
import { u8aToHex, u8aEquals, stringToU8a, moveDecimalPoint, u8aConcat } from '@hoprnet/hopr-utils'
import type { Types as Interfaces } from '@hoprnet/hopr-core-connector-interface'
import Web3 from 'web3'
import BN from 'bn.js'
import { publicKeyConvert, publicKeyCreate } from 'secp256k1'

export class Address implements Interfaces.Address {
  constructor(private arr: Uint8Array) {}

  static get SIZE(): number {
    return ADDRESS_LENGTH
  }

  static fromString(str: string): Address {
    if (!Web3.utils.isAddress(str)) throw Error(`String ${str} is not an address`)
    return new Address(stringToU8a(str))
  }

  serialize() {
    return this.arr
  }

  toHex(): string {
    return Web3.utils.toChecksumAddress(u8aToHex(this.arr, false))
  }

  eq(b: Address) {
    return u8aEquals(this.arr, b.serialize())
  }
}

export class Balance implements Interfaces.Balance {
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

export class Hash implements Interfaces.Hash {
  constructor(private arr: Uint8Array) {}

  static SIZE = HASH_LENGTH

  static create(msg: Uint8Array) {
    return new Hash(createKeccakHash('keccak256').update(Buffer.from(msg)).digest())
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

  get length() {
    return this.arr.length
  }
}

export class NativeBalance implements Interfaces.Balance {
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

export class PublicKey implements Interfaces.PublicKey {
  constructor(private arr: Uint8Array) {
    if (arr.length !== PublicKey.SIZE) {
      throw new Error('Incorrect size Uint8Array for compressed public key')
    }
    // TODO check length
  }

  static fromPrivKey(privKey: Uint8Array): PublicKey {
    if (privKey.length !== 32) {
      throw new Error('Incorrect size Uint8Array for private key')
    }
    let arr = publicKeyCreate(privKey, true)
    return new PublicKey(arr)
  }

  static fromUncompressedPubKey(pubkey: Uint8Array) {
    const uncompressedPubKey = u8aConcat(new Uint8Array([4]), pubkey)
    return new PublicKey(publicKeyConvert(uncompressedPubKey, true))
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
