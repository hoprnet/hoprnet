import Multiaddr from 'multiaddr'
import PeerId from 'peer-id'
import { ethers } from 'ethers'
import { u8aSplit, serializeToU8a, MULTI_ADDR_MAX_LENGTH, u8aEquals } from '..'
import BN from 'bn.js'
import { Address, PublicKey, Hash } from '.' // TODO: cyclic dep

export class AccountEntry {
  constructor(
    public readonly address: Address,
    public readonly multiAddr: Multiaddr,
    public readonly updatedBlock: BN
  ) {}

  static get SIZE(): number {
    return Address.SIZE + MULTI_ADDR_MAX_LENGTH
  }

  static deserialize(arr: Uint8Array) {
    const [a, b, c] = u8aSplit(arr, [Address.SIZE, MULTI_ADDR_MAX_LENGTH, 32])
    // strip b as it may contain zeros since we don't know
    // the exact multiaddr length
    const strippedB = ethers.utils.stripZeros(b)
    const isBEmpty = u8aEquals(strippedB, new Uint8Array({ length: strippedB.length }))
    const address = new Address(a)
    const blockNumber = new BN(c)
    return new AccountEntry(address, isBEmpty ? undefined : Multiaddr(strippedB), blockNumber)
  }

  // TODO: kill
  public get secret() {
    return new Hash(new Uint8Array({ length: Hash.SIZE }))
  }
  // TODO: kill
  public get counter() {
    return new BN(0)
  }

  public serialize(): Uint8Array {
    const multiaddr = ethers.utils.zeroPad(
      this.multiAddr ? this.multiAddr.bytes : new Uint8Array(),
      MULTI_ADDR_MAX_LENGTH
    )

    return serializeToU8a([
      [this.address.serialize(), Address.SIZE],
      [multiaddr, MULTI_ADDR_MAX_LENGTH],
      [this.updatedBlock.toBuffer('be', 32), 32]
    ])
  }

  public getPeerId() {
    return this.multiAddr.getPeerId()
  }

  public getPublicKey(): PublicKey {
    return new PublicKey(PeerId.createFromB58String(this.multiAddr.getPeerId()).pubKey.marshal())
  }

  public containsRouting(): boolean {
    const protos = this.multiAddr.protoNames()
    return protos.includes('ip4') && protos.includes('tcp')
  }

  public hasAnnounced(): boolean {
    return typeof this.multiAddr !== 'undefined'
  }
}
