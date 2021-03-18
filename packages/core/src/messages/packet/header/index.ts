import secp256k1 from 'secp256k1'
import hkdf from 'futoin-hkdf'
import crypto from 'crypto'
import { randomBytes } from 'crypto'

import { u8aXOR, u8aConcat, u8aEquals, u8aToHex, PRP, PRG, serializeToU8a } from '@hoprnet/hopr-utils'
import type { Types } from '@hoprnet/hopr-core-connector-interface'

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
  DESTINATION_SIZE
} from './parameters'
import PeerId from 'peer-id'
import Debug from 'debug'

const log = Debug('hopr-core:packet:header')

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

export type CipherParameters = {
  key: Uint8Array
  iv: Uint8Array
}

export type PRGParameters = {
  key: Uint8Array
  iv: Uint8Array
}

function checkPeerIds(peerIds: PeerId[]) {
  if (peerIds.length > MAX_HOPS) {
    log('Exceeded max hops')
    throw Error(`Expected at most ${MAX_HOPS} but got ${peerIds.length}`)
  }
  peerIds.forEach((peerId, index) => {
    if (peerId.pubKey == null) {
      throw Error(`Invalid peerId at index ${index}.`)
    }
  })
}

function generateKeyShares(peerIds: PeerId[]): { secrets: Uint8Array[], alpha: Uint8Array } {
  let done = false
  let secrets: Uint8Array[]
  let privKey: Uint8Array
  let alpha

  // Generate the Diffie-Hellman key shares and
  // the respective blinding factors for the
  // relays.
  // There exists a negligible, but NON-ZERO,
  // probability that the key share is chosen
  // such that it yields non-group elements.
  do {
    // initialize values
    let mul = new Uint8Array(PRIVATE_KEY_LENGTH)

    mul[PRIVATE_KEY_LENGTH - 1] = 1
    const G = secp256k1.publicKeyCreate(mul)

    secrets = []

    do {
      privKey = randomBytes(PRIVATE_KEY_LENGTH)
    } while (!secp256k1.privateKeyVerify(privKey))

    alpha = secp256k1.publicKeyCreate(privKey)

    mul.set(privKey, 0)

    peerIds.forEach((peerId: PeerId, index: number) => {
      // parallel
      // thread 1
      const alpha = secp256k1.publicKeyTweakMul(G, mul)
      // secp256k1.publicKeyVerify(alpha)

      // thread 2
      const secret = secp256k1.publicKeyTweakMul(peerId.pubKey.marshal(), mul)
      // secp256k1.publicKeyVerify(secret)
      // end parallel

      if (!secp256k1.publicKeyVerify(alpha) || !secp256k1.publicKeyVerify(secret)) {
        return
      }

      mul = secp256k1.privateKeyTweakMul(mul, deriveBlinding(alpha, secret))

      if (!secp256k1.privateKeyVerify(mul)) {
        return
      }

      secrets.push(secret)

      if (index == peerIds.length - 1) {
        done = true
      }
    })
  } while (!done)

  return { secrets, alpha }
}

function generateFiller(secrets: Uint8Array[]) {
  const filler = new Uint8Array(PER_HOP_SIZE * (secrets.length - 1))

  let length: number = 0
  let start: number = LAST_HOP_SIZE + MAX_HOPS * PER_HOP_SIZE
  let end: number = LAST_HOP_SIZE + MAX_HOPS * PER_HOP_SIZE

  for (let index = 0; index < secrets.length - 1; index++) {
    let { key, iv } = derivePRGParameters(secrets[index])

    start -= PER_HOP_SIZE
    length += PER_HOP_SIZE

    u8aXOR(true, filler.subarray(0, length), PRG.createPRG(key, iv).digest(start, end))
  }

  return filler
}

async function createBetaAndGamma(
  hash: (msg: Uint8Array) => Promise<Types.Hash>,
  peerIds: PeerId[],
  secrets: Uint8Array[],
  filler: Uint8Array,
  identifier: Uint8Array
): Promise<{ beta: Uint8Array, gamma: Uint8Array }> {
  const tmp = new Uint8Array(BETA_LENGTH - PER_HOP_SIZE)
  let beta = new Uint8Array(BETA_LENGTH)
  let gamma = new Uint8Array(MAC_SIZE)

  for (let i = secrets.length; i > 0; i--) {
    const { key, iv } = derivePRGParameters(secrets[i - 1])

    let paddingLength = (MAX_HOPS - secrets.length) * PER_HOP_SIZE

    if (i == secrets.length) {
      beta.set(peerIds[i - 1].pubKey.marshal(), 0)
      beta.set(identifier, DESTINATION_SIZE)

      // @TODO filling the array might not be necessary
      if (paddingLength > 0) {
        beta.set(randomBytes(paddingLength), LAST_HOP_SIZE)
      }

      u8aXOR(
        true,
        beta.subarray(0, LAST_HOP_SIZE + paddingLength),
        PRG.createPRG(key, iv).digest(0, LAST_HOP_SIZE + paddingLength)
      )

      beta.set(filler, LAST_HOP_SIZE + paddingLength)
    } else {
      tmp.set(beta.subarray(0, BETA_LENGTH - PER_HOP_SIZE), 0)

      beta.set(peerIds[i].pubKey.marshal(), 0)
      beta.set(gamma, ADDRESS_SIZE)

      // Used for the challenge that is created for the next node
      beta.set(
        await hash(deriveTicketKeyBlinding(secrets[i])),
        ADDRESS_SIZE + MAC_SIZE
      )
      beta.set(tmp, PER_HOP_SIZE)

      if (i < secrets.length - 1) {
        /**
         * Tells the relay node which challenge it should for the issued ticket.
         * The challenge should be done in a way such that:
         *   - the relay node does not know how to solve it
         *   - having one secret share is not sufficient to reconstruct
         *     the secret
         *   - the relay node can verify the key derivation path
         */
        beta.set(
          await hash(
            await hash(
              u8aConcat(
                deriveTicketKey(secrets[i]),
                await hash(deriveTicketKeyBlinding(secrets[i + 1]))
              )
            )
          ),
          ADDRESS_SIZE + MAC_SIZE + KEY_LENGTH
        )
      } else if (i == secrets.length - 1) {
        beta.set(
          await hash(deriveTicketLastKey(secrets[i])),
          ADDRESS_SIZE + MAC_SIZE + KEY_LENGTH
        )
      }

      u8aXOR(true, beta, PRG.createPRG(key, iv).digest(0, BETA_LENGTH))
    }

    gamma.set(createMAC(secrets[i - 1], beta), 0)
  }
  return { beta, gamma }
}

export class Header {
  tmpData?: Uint8Array
  derivedSecretLastNode?: Uint8Array

  constructor(
    readonly alpha: Uint8Array,
    readonly beta: Uint8Array,
    readonly gamma: Uint8Array) {
  }

  serialize(): Uint8Array {
    return serializeToU8a([
      [this.alpha, COMPRESSED_PUBLIC_KEY_LENGTH],
      [this.beta, BETA_LENGTH],
      [this.gamma, MAC_SIZE]
    ])
  }

  static deserialize(arr: Uint8Array): Header {

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
          this.beta.subarray(0, DESTINATION_SIZE + IDENTIFIER_SIZE),
          PRG.createPRG(key, iv).digest(0, DESTINATION_SIZE + IDENTIFIER_SIZE)
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

  static async create(
    hash: (msg: Uint8Array) => Promise<Types.Hash>,
    peerIds: PeerId[],
  ): Promise<{
    header: Header
    secrets: Uint8Array[]
    identifier: Uint8Array
  }> {
    //header.tmpData = header.beta.subarray(ADDRESS_SIZE + MAC_SIZE, PER_HOP_SIZE)
    checkPeerIds(peerIds)
    const { secrets, alpha } = generateKeyShares(peerIds)
    const identifier = randomBytes(IDENTIFIER_SIZE)
    const filler = generateFiller(secrets)
    const { beta, gamma } = await createBetaAndGamma(hash, peerIds, secrets, filler, identifier)
    return {
      header: new Header(alpha, beta, gamma),
      secrets,
      identifier
    }
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
