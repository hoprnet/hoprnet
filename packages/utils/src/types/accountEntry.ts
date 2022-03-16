import { Multiaddr, protocols } from 'multiaddr'
import PeerId from 'peer-id'
import { u8aSplit, serializeToU8a, MULTI_ADDR_MAX_LENGTH, toU8a, u8aToNumber } from '..'
import BN from 'bn.js'
import { Address, PublicKey } from './primitives'
import { decode } from 'bs58'
import { publicKeyVerify } from 'secp256k1'

const LENGTH_PREFIX_LENGTH = 4
const BLOCK_NUMBER_LENGTH = 4

const CODE_IP4 = protocols('ip4').code
const CODE_IP6 = protocols('ip6').code
const CODE_TCP = protocols('tcp').code
const CODE_P2P = protocols('p2p').code

export class AccountEntry {
  constructor(
    public readonly address: Address,
    public readonly multiAddr: Multiaddr | undefined,
    public readonly updatedBlock: BN
  ) {
    if (multiAddr.protoCodes().includes(CODE_P2P))
      if (multiAddr) {
        // If we have a multiaddr
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
    return Address.SIZE + LENGTH_PREFIX_LENGTH + MULTI_ADDR_MAX_LENGTH + BLOCK_NUMBER_LENGTH
  }

  static deserialize(arr: Uint8Array) {
    const [preAddress, preMaLength, preMultiAddr, preUpdatedBlock] = u8aSplit(arr, [
      Address.SIZE,
      LENGTH_PREFIX_LENGTH,
      MULTI_ADDR_MAX_LENGTH,
      BLOCK_NUMBER_LENGTH
    ])

    const maLength = u8aToNumber(preMaLength)
    const address = new Address(preAddress)
    const blockNumber = new BN(preUpdatedBlock)

    return new AccountEntry(
      address,
      maLength == 0 ? undefined : new Multiaddr(preMultiAddr.slice(maLength)),
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
      [this.address.serialize(), Address.SIZE],
      [serializedMultiaddr, LENGTH_PREFIX_LENGTH + MULTI_ADDR_MAX_LENGTH],
      [this.updatedBlock.toBuffer('be', 32), BLOCK_NUMBER_LENGTH]
    ])
  }

  public getPeerId(): PeerId {
    return PeerId.createFromB58String(this.multiAddr.getPeerId())
  }

  public getPublicKey(): PublicKey {
    return new PublicKey(PeerId.createFromB58String(this.multiAddr.getPeerId()).pubKey.marshal())
  }

  public containsRouting(): boolean {
    const protos = this.multiAddr.protoCodes()
    return (protos.includes(CODE_IP4) || protos.includes(CODE_IP6)) && protos.includes(CODE_TCP)
  }

  public hasAnnounced(): boolean {
    return typeof this.multiAddr !== 'undefined'
  }
}
