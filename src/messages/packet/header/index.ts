import secp256k1 from 'secp256k1'
import hkdf from 'futoin-hkdf'
import crypto from 'crypto'
import bs58 from 'bs58'

import { createHeader as createHeaderHelper } from './createHeader'

import { PRP, PRG, u8aXOR, u8aConcat } from '../../../utils'
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

const TAG_SIZE = 16

import { HoprCoreConnectorInstance } from '@hoprnet/hopr-core-connector-interface'

export type CipherParameters = {
  key: Uint8Array
  iv: Uint8Array
}

export type PRGParameters = {
  key: Uint8Array
  iv: Uint8Array
}

export class Header<Chain extends HoprCoreConnectorInstance> extends Uint8Array {
  tmpData?: Uint8Array
  derivedSecretLastNode?: Uint8Array

  constructor(arr: Uint8Array) {
    super(arr)

    if (arr.length != Header.SIZE) throw Error(`Wrong input. Please provide a Buffer of size ${Header.SIZE}.`)
  }

  subarray(begin?: number, end?: number): Uint8Array {
    return new Uint8Array(this.buffer, begin, end != null ? end - begin : undefined)
  }

  get alpha(): Uint8Array {
    return new Uint8Array(this.buffer, 0, COMPRESSED_PUBLIC_KEY_LENGTH)
  }

  get beta(): Uint8Array {
    return new Uint8Array(this.buffer, COMPRESSED_PUBLIC_KEY_LENGTH, BETA_LENGTH)
  }

  get gamma(): Uint8Array {
    return new Uint8Array(this.buffer, COMPRESSED_PUBLIC_KEY_LENGTH + BETA_LENGTH, MAC_SIZE)
  }

  get address(): this['tmpData'] {
    return this.tmpData != null ? this.tmpData.subarray(0, ADDRESS_SIZE) : undefined
  }

  get identifier(): this['tmpData'] {
    return this.tmpData != null ? this.tmpData.subarray(ADDRESS_SIZE, ADDRESS_SIZE + IDENTIFIER_SIZE) : undefined
  }

  get hashedKeyHalf(): this['tmpData'] {
    return this.tmpData != null ? this.tmpData.subarray(ADDRESS_SIZE, ADDRESS_SIZE + COMPRESSED_PUBLIC_KEY_LENGTH) : undefined
  }

  get encryptionKey(): this['tmpData'] {
    return this.tmpData != null ? this.tmpData.subarray(ADDRESS_SIZE + COMPRESSED_PUBLIC_KEY_LENGTH, ADDRESS_SIZE + PROVING_VALUES_SIZE) : undefined
  }

  get derivedSecret(): this['tmpData'] {
    return this.tmpData != null
      ? this.derivedSecretLastNode != null
        ? this.derivedSecretLastNode
        : this.tmpData.subarray(ADDRESS_SIZE + PROVING_VALUES_SIZE, ADDRESS_SIZE + PROVING_VALUES_SIZE + COMPRESSED_PUBLIC_KEY_LENGTH)
      : undefined
  }

  deriveSecret(secretKey: Uint8Array, lastNode: boolean = false): void {
    if (!secp256k1.privateKeyVerify(Buffer.from(secretKey))) {
      throw Error(`Invalid private key.`)
    }

    if (lastNode) {
      this.tmpData = this.beta.subarray(0, LAST_HOP_SIZE)
      this.derivedSecretLastNode = new Uint8Array(COMPRESSED_PUBLIC_KEY_LENGTH)
    } else {
      this.tmpData = new Uint8Array(ADDRESS_SIZE + PROVING_VALUES_SIZE + COMPRESSED_PUBLIC_KEY_LENGTH)
    }

    this.derivedSecret.set(new Uint8Array(secp256k1.publicKeyTweakMul(Buffer.from(this.alpha), Buffer.from(secretKey))), 0)
  }

  verify(): boolean {
    return createMAC(this.derivedSecret, this.beta).every((value: number, index: number) => value == this.gamma[index])
  }

  extractHeaderInformation(lastNode: boolean = false): void {
    const { key, iv } = derivePRGParameters(this.derivedSecret)

    if (lastNode) {
      const { key, iv } = derivePRGParameters(this.derivedSecret)

      this.tmpData.set(
        u8aXOR(false, this.beta.subarray(0, DESINATION_SIZE + IDENTIFIER_SIZE), PRG.createPRG(key, iv).digest(0, DESINATION_SIZE + IDENTIFIER_SIZE)),
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

    this.alpha.set(secp256k1.publicKeyTweakMul(Buffer.from(this.alpha), Buffer.from(deriveBlinding(this.alpha, this.derivedSecret))), 0)
  }

  toString(): string {
    return (
      'Header:\n' +
      '|-> Alpha:\n' +
      '|---> ' +
      bs58.encode(Buffer.from(this.alpha)) +
      '\n' +
      '|-> Beta:\n' +
      '|---> ' +
      bs58.encode(Buffer.from(this.beta)) +
      '\n' +
      '|-> Gamma:\n' +
      '|---> ' +
      bs58.encode(Buffer.from(this.gamma)) +
      '\n'
    )
  }

  static get SIZE(): number {
    return COMPRESSED_PUBLIC_KEY_LENGTH + BETA_LENGTH + MAC_SIZE
  }

  static create<Chain extends HoprCoreConnectorInstance>(
    peerIds: PeerId[]
  ): {
    header: Header<Chain>
    secrets: Uint8Array[]
    identifier: Uint8Array
  } {
    const header = new Header<Chain>(new Uint8Array(Header.SIZE))
    header.tmpData = header.beta.subarray(ADDRESS_SIZE + MAC_SIZE, PER_HOP_SIZE)

    return createHeaderHelper(header, peerIds)
  }
}

export const BETA_LENGTH = PER_HOP_SIZE * (MAX_HOPS - 1) + LAST_HOP_SIZE

export function deriveTagParameters(secret: Uint8Array): Uint8Array {
  if (!secp256k1.publicKeyVerify(Buffer.from(secret))) {
    throw Error('General error.')
  }

  return hkdf(Buffer.from(secret), TAG_SIZE, { salt: HASH_KEY_TAGGING })
}

export function deriveCipherParameters(secret: Uint8Array): CipherParameters {
  if (!secp256k1.publicKeyVerify(Buffer.from(secret))) {
    throw Error('General error.')
  }

  const keyAndIV = hkdf(Buffer.from(secret), PRP.KEY_LENGTH + PRP.IV_LENGTH, { salt: HASH_KEY_PRP })

  const key = keyAndIV.subarray(0, PRP.KEY_LENGTH)
  const iv = keyAndIV.subarray(PRP.KEY_LENGTH)

  return { key, iv }
}

export function derivePRGParameters(secret: Uint8Array): PRGParameters {
  if (!secp256k1.publicKeyVerify(Buffer.from(secret))) {
    throw Error('General error.')
  }

  const keyAndIV = hkdf(Buffer.from(secret), PRG.KEY_LENGTH + PRG.IV_LENGTH, { salt: HASH_KEY_PRG })

  const key = keyAndIV.subarray(0, PRG.KEY_LENGTH)
  const iv = keyAndIV.subarray(PRG.KEY_LENGTH, PRG.KEY_LENGTH + PRG.IV_LENGTH)

  return { key, iv }
}

export function deriveBlinding(alpha: Uint8Array, secret: Uint8Array): Uint8Array {
  if (!secp256k1.publicKeyVerify(Buffer.from(secret))) {
    throw Error('General error.')
  }

  if (!secp256k1.publicKeyVerify(Buffer.from(alpha))) {
    throw Error('General error.')
  }

  return hkdf(Buffer.from(u8aConcat(alpha, secret)), PRIVATE_KEY_LENGTH, { salt: HASH_KEY_BLINDINGS })
}

export function deriveTicketKey(secret: Uint8Array): Uint8Array {
  if (!secp256k1.publicKeyVerify(Buffer.from(secret))) {
    throw Error('General error')
  }

  return hkdf(Buffer.from(secret), KEY_LENGTH, { salt: HASH_KEY_TX })
}

export function deriveTransactionKeyBlinding(secret: Uint8Array) {
  if (!secp256k1.publicKeyVerify(Buffer.from(secret))) {
    throw Error('General error')
  }

  return hkdf(Buffer.from(secret), KEY_LENGTH, { salt: HASH_KEY_TX_BLINDED })
}

export function createMAC(secret: Uint8Array, msg: Uint8Array): Uint8Array {
  if (!secp256k1.publicKeyVerify(Buffer.from(secret))) {
    throw Error('General error')
  }

  const key = hkdf(Buffer.from(secret), MAC_KEY_LENGTH, { salt: HASH_KEY_HMAC })

  return crypto
    .createHmac('sha256', key)
    .update(msg)
    .digest()
}
