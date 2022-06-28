import secp256k1 from 'secp256k1'
import { u8aToHex, u8aEquals, stringToU8a } from '../u8a/index.js'
import type { PeerId } from '@libp2p/interface-peer-id'
import { peerIdFromString } from '@libp2p/peer-id'
import { pubKeyToPeerId } from '../libp2p/index.js'
import { Address, Hash } from './primitives.js'
import { unmarshalPublicKey } from '@libp2p/crypto/keys'

const SIZE_PUBKEY_UNCOMPRESSED = 65
const SIZE_PUBKEY_UNCOMPRESSED_WITHOUT_PREFIX = 64
const SIZE_PUBKEY_COMPRESSED = 33
const SIZE_PRIVKEY = 32

export class PublicKey {
  // Cache expensive computation result
  private _address: Address

  private constructor(private arr: Uint8Array) {}

  static fromPrivKeyString(privKey: string) {
    return PublicKey.fromPrivKey(stringToU8a(privKey))
  }

  static fromPrivKey(privKey: Uint8Array): PublicKey {
    if (privKey.length != SIZE_PRIVKEY) {
      throw new Error('Incorrect size Uint8Array for private key')
    }

    return new PublicKey(secp256k1.publicKeyCreate(privKey, false))
  }

  static deserialize(arr: Uint8Array) {
    switch (arr.length) {
      case SIZE_PUBKEY_UNCOMPRESSED:
        if (arr[0] != 4) {
          throw Error(`Invalid uncompressed public key`)
        }
        return new PublicKey(arr)
      case SIZE_PUBKEY_UNCOMPRESSED_WITHOUT_PREFIX:
        return new PublicKey(Uint8Array.from([4, ...arr]))
      case SIZE_PUBKEY_COMPRESSED:
        if (![2, 3].includes(arr[0])) {
          throw Error(`Invalid compressed public key`)
        }
        return new PublicKey(arr)
      default:
        throw Error(`Invalid length ${arr.length} of public key`)
    }
  }

  static fromPeerId(peerId: PeerId): PublicKey {
    if (peerId.type !== 'secp256k1') {
      throw Error(`PublicKey class only supports secp256k1 public keys`)
    }
    return PublicKey.deserialize(unmarshalPublicKey(peerId.publicKey).marshal())
  }

  static fromPeerIdString(peerIdString: string) {
    return PublicKey.fromPeerId(peerIdFromString(peerIdString))
  }

  static fromSignature(hash: Uint8Array, signature: Uint8Array, v: number): PublicKey {
    return new PublicKey(secp256k1.ecdsaRecover(signature, v, hash, false))
  }

  static fromSignatureString(hash: string, r: string, s: string, v: number): PublicKey {
    return PublicKey.fromSignature(stringToU8a(hash), Uint8Array.from([...stringToU8a(r), ...stringToU8a(s)]), v)
  }

  static fromString(str: string): PublicKey {
    if (!str || str.length == 0) {
      throw new Error('Cannot determine address from empty string')
    }
    return PublicKey.deserialize(stringToU8a(str))
  }

  static get SIZE_COMPRESSED(): number {
    return SIZE_PUBKEY_COMPRESSED
  }

  static get SIZE_UNCOMPRESSED(): number {
    return SIZE_PUBKEY_UNCOMPRESSED
  }

  public get isCompressed(): boolean {
    return [2, 3].includes(this.arr[0])
  }

  public toAddress(): Address {
    if (this._address != undefined) {
      return this._address
    }

    if (this.isCompressed) {
      // Expensive EC-operation, only do if necessary
      this.arr = secp256k1.publicKeyConvert(this.arr, false)
    }

    this._address = new Address(Hash.create(this.arr.slice(1)).serialize().slice(12))

    return this._address
  }

  public toUncompressedPubKeyHex(): string {
    return u8aToHex(this.serializeUncompressed().slice(1))
  }

  public toCompressedPubKeyHex(): string {
    return u8aToHex(this.serializeCompressed())
  }

  public toPeerId(): PeerId {
    return pubKeyToPeerId(this.serializeCompressed())
  }

  public serializeCompressed(): Uint8Array {
    if (this.isCompressed) {
      return this.arr
    } else {
      return secp256k1.publicKeyConvert(this.arr, true)
    }
  }

  public serializeUncompressed() {
    if (this.isCompressed) {
      // Expensive EC-operation, only do if really necessary
      this.arr = secp256k1.publicKeyConvert(this.arr, false)
    }

    return this.arr
  }

  toString(): string {
    return this.toPeerId().toString()
  }

  eq(b: PublicKey) {
    if (this.arr[0] == b.arr[0]) {
      return u8aEquals(this.arr, b.arr)
    } else if (this.isCompressed) {
      return u8aEquals(this.arr, b.serializeCompressed())
    } else {
      return u8aEquals(this.serializeCompressed(), b.arr)
    }
  }

  static createMock(): PublicKey {
    return PublicKey.fromString('0x021464586aeaea0eb5736884ca1bf42d165fc8e2243b1d917130fb9e321d7a93b8')
  }
}
