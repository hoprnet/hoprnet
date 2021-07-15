import { Multiaddr } from 'multiaddr'
import PeerId from 'peer-id'
import { ethers } from 'ethers'
import { u8aSplit, serializeToU8a, MULTI_ADDR_MAX_LENGTH, u8aEquals } from '..'
import BN from 'bn.js'
import { Address, PublicKey } from './primitives'
import { decode } from 'bs58'
import { publicKeyVerify } from 'secp256k1'

export class AccountEntry {
  constructor(
    public readonly address: Address,
    public readonly multiAddr: Multiaddr | undefined,
    public readonly updatedBlock: BN
  ) {
    // If we have a multiaddr
    if (multiAddr) {
      let encodedPublicKey: string
      try {
        encodedPublicKey = multiAddr.getPeerId()
      } catch {}

      if (encodedPublicKey) {
        // Decode bs58-encoded public key and take last 33 bytes
        // which contain the public key
        const pubKey = decode(encodedPublicKey).slice(-33)

        if (!publicKeyVerify(pubKey)) {
          throw Error(`Multiaddr does not contain a valid public key.`)
        }
      }
    }
  }

  static get SIZE(): number {
    return Address.SIZE + MULTI_ADDR_MAX_LENGTH + 32
  }

  static deserialize(arr: Uint8Array) {
    const [a, b, c] = u8aSplit(arr, [Address.SIZE, MULTI_ADDR_MAX_LENGTH, 32])
    // strip b as it may contain zeros since we don't know
    // the exact multiaddr length
    const strippedB = ethers.utils.stripZeros(b)
    const isBEmpty = u8aEquals(strippedB, new Uint8Array({ length: strippedB.length }))
    const address = new Address(a)
    const blockNumber = new BN(c)
    return new AccountEntry(address, isBEmpty ? undefined : new Multiaddr(strippedB), blockNumber)
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

  public getPeerId(): PeerId {
    return PeerId.createFromB58String(this.multiAddr.getPeerId())
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
