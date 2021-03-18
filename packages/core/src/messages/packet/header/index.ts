import secp256k1 from 'secp256k1'
import hkdf from 'futoin-hkdf'
import crypto from 'crypto'

import { createHeader as createHeaderHelper } from './createHeader'
import Hopr from '../../..'
import { u8aXOR, u8aConcat, u8aEquals, u8aToHex, PRP, PRG } from '@hoprnet/hopr-utils'

import { MAX_HOPS } from '../../../constants'

import {
  COMPRESSED_PUBLIC_KEY_LENGTH,
  ADDRESS_SIZE,
  MAC_SIZE,
  PROVING_VALUES_SIZE,
  LAST_HOP_SIZE,
  PER_HOP_SIZE,
  PRIVATE_KEY_LENGTH,
  KEY_LENGTH,
  IDENTIFIER_SIZE,
  DESINATION_SIZE
} from './parameters'
import PeerId from 'peer-id'

const MAC_KEY_LENGTH = 16
const HASH_KEY_PRG = 'P'
const HASH_KEY_PRP = 'W'
const HASH_KEY_BLINDINGS = 'B'
const HASH_KEY_HMAC = 'H'
const HASH_KEY_TAGGING = 'T'
const HASH_KEY_TX = 'Tx'
const HASH_KEY_TX_BLINDED = 'Tx_'
const HASH_KEY_TX_LAST = 'Tx_Last'
const HASH_KEY_TX_LAST_BLINDED = 'Tx_Last_'

const TAG_SIZE = 16

import HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'

export type CipherParameters = {
  key: Uint8Array
  iv: Uint8Array
}

export type PRGParameters = {
  key: Uint8Array
  iv: Uint8Array
}

export class Header extends Uint8Array {
  tmpData?: Uint8Array
  derivedSecretLastNode?: Uint8Array

  constructor(arr: { bytes: ArrayBuffer; offset: number }) {
    super(arr.bytes, arr.offset, Header.SIZE)
  }

  slice(begin: number = 0, end: number = Header.SIZE) {
    return this.subarray(begin, end)
  }

  subarray(begin: number = 0, end: number = Header.SIZE): Uint8Array {
    return new Uint8Array(this.buffer, begin + this.byteOffset, end - begin)
  }

  get alpha(): Uint8Array {
    return this.subarray(0, COMPRESSED_PUBLIC_KEY_LENGTH)
  }

  get beta(): Uint8Array {
    return this.subarray(COMPRESSED_PUBLIC_KEY_LENGTH, COMPRESSED_PUBLIC_KEY_LENGTH + BETA_LENGTH)
  }

  get gamma(): Uint8Array {
    return this.subarray(
      COMPRESSED_PUBLIC_KEY_LENGTH + BETA_LENGTH,
      COMPRESSED_PUBLIC_KEY_LENGTH + BETA_LENGTH + MAC_SIZE
    )
  }

  get address(): this['tmpData'] {
    return this.tmpData != null ? this.tmpData.subarray(0, ADDRESS_SIZE) : undefined
  }

  get identifier(): this['tmpData'] {
    return this.tmpData != null ? this.tmpData.subarray(ADDRESS_SIZE, ADDRESS_SIZE + IDENTIFIER_SIZE) : undefined
  }

  get hashedKeyHalf(): this['tmpData'] {
    return this.tmpData != null ? this.tmpData.subarray(ADDRESS_SIZE, ADDRESS_SIZE + KEY_LENGTH) : undefined
  }

  get encryptionKey(): this['tmpData'] {
    return this.tmpData != null
      ? this.tmpData.subarray(ADDRESS_SIZE + KEY_LENGTH, ADDRESS_SIZE + PROVING_VALUES_SIZE)
      : undefined
  }

  get derivedSecret(): this['tmpData'] {
    return this.tmpData != null
      ? this.derivedSecretLastNode != null
        ? this.derivedSecretLastNode
        : this.tmpData.subarray(
            ADDRESS_SIZE + PROVING_VALUES_SIZE,
            ADDRESS_SIZE + PROVING_VALUES_SIZE + COMPRESSED_PUBLIC_KEY_LENGTH
          )
      : undefined
  }

  deriveSecret(secretKey: Uint8Array, lastNode: boolean = false): void {
    if (!secp256k1.privateKeyVerify(secretKey)) {
      throw Error(`Invalid private key.`)
    }

    if (lastNode) {
      this.tmpData = this.beta.subarray(0, LAST_HOP_SIZE)
      this.derivedSecretLastNode = new Uint8Array(COMPRESSED_PUBLIC_KEY_LENGTH)
    } else {
      this.tmpData = new Uint8Array(ADDRESS_SIZE + PROVING_VALUES_SIZE + COMPRESSED_PUBLIC_KEY_LENGTH)
    }

    this.derivedSecret.set(secp256k1.publicKeyTweakMul(this.alpha, secretKey), 0)
  }

  verify(): boolean {
    return u8aEquals(createMAC(this.derivedSecret, this.beta), this.gamma)
  }

  extractHeaderInformation(lastNode: boolean = false): void {
    const { key, iv } = derivePRGParameters(this.derivedSecret)

    if (lastNode) {
      const { key, iv } = derivePRGParameters(this.derivedSecret)

      this.tmpData.set(
        u8aXOR(
          false,
          this.beta.subarray(0, DESINATION_SIZE + IDENTIFIER_SIZE),
          PRG.createPRG(key, iv).digest(0, DESINATION_SIZE + IDENTIFIER_SIZE)
        ),
        0
      )
    } else {
      const tmp = new Uint8Array(BETA_LENGTH + PER_HOP_SIZE)

      tmp.set(this.beta, 0)
      tmp.fill(0, BETA_LENGTH, BETA_LENGTH + PER_HOP_SIZE)

      u8aXOR(true, tmp, PRG.createPRG(key, iv).digest(0, BETA_LENGTH + PER_HOP_SIZE))

      // this.tmpData = this.tmpData || Buffer.alloc(ADDRESS_SIZE + PROVING_VALUES_SIZE + COMPRESSED_PUBLIC_KEY_LENGTH)

      this.tmpData.set(tmp.subarray(0, ADDRESS_SIZE), 0)

      this.tmpData.set(tmp.subarray(ADDRESS_SIZE + MAC_SIZE, PER_HOP_SIZE), ADDRESS_SIZE)

      this.gamma.set(tmp.subarray(ADDRESS_SIZE, ADDRESS_SIZE + MAC_SIZE), 0)

      this.beta.set(tmp.subarray(PER_HOP_SIZE, PER_HOP_SIZE + BETA_LENGTH), 0)
    }
  }

  transformForNextNode(): void {
    if (this.tmpData == null) {
      throw Error(`Cannot read from 'this.data'. Please call 'deriveSecret()' first.`)
    }

    this.alpha.set(secp256k1.publicKeyTweakMul(this.alpha, deriveBlinding(this.alpha, this.derivedSecret)), 0)
  }

  toString(): string {
    return (
      'Header:\n' +
      '|-> Alpha:\n' +
      '|---> ' +
      u8aToHex(this.alpha) +
      '\n' +
      '|-> Beta:\n' +
      '|---> ' +
      u8aToHex(this.beta) +
      '\n' +
      '|-> Gamma:\n' +
      '|---> ' +
      u8aToHex(this.gamma) +
      '\n'
    )
  }

  static get SIZE(): number {
    return COMPRESSED_PUBLIC_KEY_LENGTH + BETA_LENGTH + MAC_SIZE
  }

  static async create<Chain extends HoprCoreConnector>(
    node: Hopr<Chain>,
    peerIds: PeerId[],
    arr?: {
      bytes: ArrayBuffer
      offset: number
    }
  ): Promise<{
    header: Header
    secrets: Uint8Array[]
    identifier: Uint8Array
  }> {
    if (arr == null) {
      let tmpArray = new Uint8Array(Header.SIZE)
      arr = {
        bytes: tmpArray.buffer,
        offset: tmpArray.byteOffset
      }
    }

    const header = new Header(arr)
    header.tmpData = header.beta.subarray(ADDRESS_SIZE + MAC_SIZE, PER_HOP_SIZE)

    return createHeaderHelper(node.paymentChannels.utils.hash, header, peerIds)
  }
}

export const BETA_LENGTH = PER_HOP_SIZE * (MAX_HOPS - 1) + LAST_HOP_SIZE

export function deriveTagParameters(secret: Uint8Array): Uint8Array {
  if (secret.length != COMPRESSED_PUBLIC_KEY_LENGTH || (secret[0] != 0x02 && secret[0] != 0x03)) {
    throw Error('Secret must be a public key.')
  }

  return hkdf(Buffer.from(secret), TAG_SIZE, { salt: HASH_KEY_TAGGING })
}

export function deriveCipherParameters(secret: Uint8Array): CipherParameters {
  if (secret.length != COMPRESSED_PUBLIC_KEY_LENGTH || (secret[0] != 0x02 && secret[0] != 0x03)) {
    throw Error('Secret must be a public key')
  }

  const keyAndIV = hkdf(Buffer.from(secret), PRP.KEY_LENGTH + PRP.IV_LENGTH, { salt: HASH_KEY_PRP })

  const key = keyAndIV.subarray(0, PRP.KEY_LENGTH)
  const iv = keyAndIV.subarray(PRP.KEY_LENGTH)

  return { key, iv }
}

export function derivePRGParameters(secret: Uint8Array): PRGParameters {
  if (secret.length != COMPRESSED_PUBLIC_KEY_LENGTH || (secret[0] != 0x02 && secret[0] != 0x03)) {
    throw Error('Secret must be a public key')
  }

  const keyAndIV = hkdf(Buffer.from(secret), PRG.KEY_LENGTH + PRG.IV_LENGTH, { salt: HASH_KEY_PRG })

  const key = keyAndIV.subarray(0, PRG.KEY_LENGTH)
  const iv = keyAndIV.subarray(PRG.KEY_LENGTH, PRG.KEY_LENGTH + PRG.IV_LENGTH)

  return { key, iv }
}

export function deriveBlinding(alpha: Uint8Array, secret: Uint8Array): Uint8Array {
  if (secret.length != COMPRESSED_PUBLIC_KEY_LENGTH || (secret[0] != 0x02 && secret[0] != 0x03)) {
    throw Error('Secret must be a public key')
  }

  if (alpha.length != COMPRESSED_PUBLIC_KEY_LENGTH || (alpha[0] != 0x02 && alpha[0] != 0x03)) {
    throw Error('Alpha must be a public key')
  }

  return hkdf(Buffer.from(u8aConcat(alpha, secret)), PRIVATE_KEY_LENGTH, { salt: HASH_KEY_BLINDINGS })
}

function derivationHelper(secret: Uint8Array, salt: string) {
  if (secret.length != COMPRESSED_PUBLIC_KEY_LENGTH || (secret[0] != 0x02 && secret[0] != 0x03)) {
    throw Error('Secret must be a public key')
  }

  return hkdf(Buffer.from(secret), KEY_LENGTH, { salt })
}

export function deriveTicketKey(secret: Uint8Array): Uint8Array {
  return derivationHelper(secret, HASH_KEY_TX)
}

export function deriveTicketKeyBlinding(secret: Uint8Array): Uint8Array {
  return derivationHelper(secret, HASH_KEY_TX_BLINDED)
}

export function deriveTicketLastKey(secret: Uint8Array): Uint8Array {
  return derivationHelper(secret, HASH_KEY_TX_LAST)
}

export function deriveTicketLastKeyBlinding(secret: Uint8Array) {
  return derivationHelper(secret, HASH_KEY_TX_LAST_BLINDED)
}

export function createMAC(secret: Uint8Array, msg: Uint8Array): Uint8Array {
  if (secret.length != COMPRESSED_PUBLIC_KEY_LENGTH || (secret[0] != 0x02 && secret[0] != 0x03)) {
    throw Error('Secret must be a public key')
  }

  const key = hkdf(Buffer.from(secret), MAC_KEY_LENGTH, { salt: HASH_KEY_HMAC })

  return crypto.createHmac('sha256', key).update(msg).digest()
}
