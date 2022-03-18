import { utils } from 'ethers'
import BN from 'bn.js'
import { ecdsaSign, ecdsaVerify } from 'secp256k1'
import { moveDecimalPoint } from '../math'
import { u8aToHex, u8aEquals, stringToU8a, u8aConcat } from '../u8a'
import { ADDRESS_LENGTH, HASH_LENGTH, SIGNATURE_LENGTH } from '../constants'
import type { PublicKey } from './publicKey'

export class Address {
  constructor(private arr: Uint8Array) {
    if (arr.length != Address.SIZE) {
      throw new Error('Incorrect size Uint8Array for address')
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
    return utils.getAddress(u8aToHex(this.arr, false))
  }

  toString(): string {
    return this.toHex()
  }

  toBytes32(): Uint8Array {
    return Uint8Array.from([...new Uint8Array(12).fill(0), ...this.arr])
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
    if (typeof Buffer !== 'undefined' && Buffer.isBuffer(arr)) {
      throw Error(`Expected a Uint8Array but got a Buffer`)
    }

    if (arr.length != Hash.SIZE) {
      throw new Error('Incorrect size Uint8Array for hash')
    }
  }

  static SIZE = HASH_LENGTH

  static create(...inputs: Uint8Array[]) {
    return new Hash(stringToU8a(utils.keccak256(u8aConcat(...inputs))))
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

/**
 * Class used to represent an ECDSA signature.
 *
 * The methods serialize()/deserialize() are used to convert the signature
 * to/from 64-byte compressed representation as given by EIP-2098 (https://eips.ethereum.org/EIPS/eip-2098).
 * This compressed signature format is supported by OpenZeppelin.
 *
 * Internally this class still maintains representation using `(r,s)` tuple and `v` parity component separate
 * as this makes interop with the underlying ECDSA library simpler.
 */
export class Signature {
  constructor(readonly signature: Uint8Array, readonly recovery: number) {
    if (signature.length !== SIGNATURE_LENGTH) {
      throw new Error('Incorrect size Uint8Array for signature')
    }
    if (![0, 1].includes(recovery)) {
      throw new Error(`Recovery must be either 1 or 0, got ${recovery}`)
    }
  }

  static deserialize(arr: Uint8Array): Signature {
    if (arr.length !== SIGNATURE_LENGTH) {
      throw new Error('Incorrect size Uint8Array for signature')
    }

    const arrCopy = new Uint8Array(arr)

    // Read & clear the top-most bit in S
    const recovery = (arrCopy[SIGNATURE_LENGTH / 2] & 0x80) != 0 ? 1 : 0
    arrCopy[SIGNATURE_LENGTH / 2] &= 0x7f

    return new Signature(arrCopy, recovery)
  }

  static create(msg: Uint8Array, privKey: Uint8Array): Signature {
    const result = ecdsaSign(msg, privKey)
    return new Signature(result.signature, result.recid)
  }

  serialize(): Uint8Array {
    const compressedSig = new Uint8Array(this.signature)
    compressedSig[SIGNATURE_LENGTH / 2] &= 0x7f
    compressedSig[SIGNATURE_LENGTH / 2] |= this.recovery << 7
    return compressedSig
  }

  verify(msg: Uint8Array, pubKey: PublicKey): boolean {
    return ecdsaVerify(this.signature, msg, pubKey.serializeUncompressed())
  }

  toHex(): string {
    return u8aToHex(this.serialize())
  }

  static SIZE = SIGNATURE_LENGTH
}

abstract class BalanceBase {
  //Uint256
  static readonly SIZE: number = 32
  static readonly DECIMALS: number = 18
  abstract readonly symbol: string

  constructor(protected bn: BN) {}

  abstract add(b: BalanceBase): BalanceBase
  abstract sub(b: BalanceBase): BalanceBase

  public eq(b: Balance): boolean {
    return this.bn.eq(b.bn)
  }

  public toBN(): BN {
    return this.bn
  }

  public toHex(): string {
    return `0x${this.bn.toString('hex', 2 * BalanceBase.SIZE)}`
  }

  public lt(b: BalanceBase): boolean {
    return this.bn.lt(b.bn)
  }

  public gt(b: BalanceBase): boolean {
    return this.bn.gt(b.bn)
  }

  public gte(b: BalanceBase): boolean {
    return this.bn.gte(b.bn)
  }

  public lte(b: BalanceBase): boolean {
    return this.bn.lte(b.bn)
  }

  public serialize(): Uint8Array {
    return new Uint8Array(this.bn.toBuffer('be', BalanceBase.SIZE))
  }

  public toString(): string {
    return this.bn.toString()
  }

  public toFormattedString(): string {
    const str = moveDecimalPoint(this.toString(), BalanceBase.DECIMALS * -1)
    return `${str} ${this.symbol}`
  }
}

export class Balance extends BalanceBase {
  static SYMBOL: string = 'wxHOPR'
  readonly symbol: string = Balance.SYMBOL

  public add(b: Balance): Balance {
    return new Balance(this.bn.add(b.bn))
  }

  public sub(b: Balance): Balance {
    return new Balance(this.bn.sub(b.bn))
  }

  static deserialize(arr: Uint8Array): Balance {
    return new Balance(new BN(arr))
  }

  static get ZERO(): Balance {
    return new Balance(new BN('0'))
  }
}

export class NativeBalance extends BalanceBase {
  static SYMBOL: string = 'xDAI'
  readonly symbol: string = NativeBalance.SYMBOL

  public add(b: NativeBalance): NativeBalance {
    return new NativeBalance(this.bn.add(b.bn))
  }

  public sub(b: NativeBalance): NativeBalance {
    return new NativeBalance(this.bn.sub(b.bn))
  }

  static deserialize(arr: Uint8Array): NativeBalance {
    return new NativeBalance(new BN(arr))
  }
  static get ZERO(): NativeBalance {
    return new NativeBalance(new BN('0'))
  }
}
