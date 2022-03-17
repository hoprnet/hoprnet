import { Multiaddr, protocols } from 'multiaddr'
import PeerId from 'peer-id'
import { u8aSplit, serializeToU8a, MULTI_ADDR_MAX_LENGTH, toU8a, u8aToNumber } from '..'
import BN from 'bn.js'
import { PublicKey } from './publicKey'
import type { Address } from './primitives'

const LENGTH_PREFIX_LENGTH = 4
const BLOCK_NUMBER_LENGTH = 4

const CODE_IP4 = protocols('ip4').code
const CODE_IP6 = protocols('ip6').code
const CODE_TCP = protocols('tcp').code

export class AccountEntry {
  constructor(
    public readonly publicKey: PublicKey,
    public readonly multiAddr: Multiaddr | undefined,
    public readonly updatedBlock: BN
  ) {}

  static get SIZE(): number {
    return PublicKey.SIZE_UNCOMPRESSED + LENGTH_PREFIX_LENGTH + MULTI_ADDR_MAX_LENGTH + BLOCK_NUMBER_LENGTH
  }

  static deserialize(arr: Uint8Array) {
    const [preAddress, preMaLength, preMultiAddr, preUpdatedBlock] = u8aSplit(arr, [
      PublicKey.SIZE_UNCOMPRESSED,
      LENGTH_PREFIX_LENGTH,
      MULTI_ADDR_MAX_LENGTH,
      BLOCK_NUMBER_LENGTH
    ])

    const maLength = u8aToNumber(preMaLength)
    const pubKey = PublicKey.deserialize(preAddress)
    const blockNumber = new BN(preUpdatedBlock)

    return new AccountEntry(
      pubKey,
      maLength == 0 ? undefined : new Multiaddr(preMultiAddr.slice(MULTI_ADDR_MAX_LENGTH - maLength)),
      blockNumber
    )
  }

  public serialize(): Uint8Array {
    let serializedMultiaddr: Uint8Array

    if (this.multiAddr) {
      if (this.multiAddr.bytes.length > MULTI_ADDR_MAX_LENGTH) {
        throw Error(
          `Multiaddr is ${this.multiAddr.bytes.length - MULTI_ADDR_MAX_LENGTH} bytes longer than maximum length.`
        )
      }

      serializedMultiaddr = Uint8Array.from([
        ...toU8a(this.multiAddr.bytes.length, LENGTH_PREFIX_LENGTH),
        ...new Uint8Array(MULTI_ADDR_MAX_LENGTH - this.multiAddr.bytes.length).fill(0x00),
        ...this.multiAddr.bytes
      ])
    } else {
      serializedMultiaddr = Uint8Array.from([
        ...new Uint8Array(LENGTH_PREFIX_LENGTH + MULTI_ADDR_MAX_LENGTH).fill(0x00)
      ])
    }

    return serializeToU8a([
      [this.publicKey.serializeUncompressed(), PublicKey.SIZE_UNCOMPRESSED],
      [serializedMultiaddr, LENGTH_PREFIX_LENGTH + MULTI_ADDR_MAX_LENGTH],
      [this.updatedBlock.toBuffer('be', 4), BLOCK_NUMBER_LENGTH]
    ])
  }

  public getPeerId(): PeerId {
    return this.publicKey.toPeerId()
  }

  public getAddress(): Address {
    return this.publicKey.toAddress()
  }

  public get containsRouting(): boolean {
    if (!this.multiAddr) {
      return false
    }

    const protos = this.multiAddr.protoCodes()
    return (protos.includes(CODE_IP4) || protos.includes(CODE_IP6)) && protos.includes(CODE_TCP)
  }

  public get hasAnnounced(): boolean {
    return !!this.multiAddr
  }

  public toString(): string {
    return (
      // prettier-ignore
      `AccountEntry: ${this.publicKey.toB58String()}\n` +
      `  PublicKey: ${this.publicKey.toCompressedPubKeyHex()}\n` +
      `  Multiaddr: ${this.multiAddr ? this.multiAddr.toString() : 'not announced'}\n` +
      `  updatedAt: ${this.updatedBlock ? `Block ${this.updatedBlock.toString(10)}` : `not annonced`}\n` +
      `  routableAddress: ${this.hasAnnounced ? this.containsRouting : 'not announced'}\n`
    )
  }
}
