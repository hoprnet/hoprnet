import secp256k1 from 'secp256k1'
import { randomBytes } from 'crypto'
import { u8aXOR, u8aConcat, PRG } from '@hoprnet/hopr-utils'
import { MAX_HOPS } from '../../../constants'

import {
  Header,
  BETA_LENGTH,
  deriveBlinding,
  derivePRGParameters,
  deriveTicketKey,
  deriveTicketKeyBlinding,
  deriveTicketLastKey,
  createMAC
} from './index'

import type { Types } from '@hoprnet/hopr-core-connector-interface'
import PeerId from 'peer-id'
import Debug from 'debug'
const log = Debug('hopr-core:packet:header')

import {
  PRIVATE_KEY_LENGTH,
  PER_HOP_SIZE,
  DESINATION_SIZE,
  IDENTIFIER_SIZE,
  ADDRESS_SIZE,
  MAC_SIZE,
  LAST_HOP_SIZE,
  KEY_LENGTH
} from './parameters'

export async function createHeader(hash: (msg: Uint8Array) => Promise<Types.Hash>, header: Header, peerIds: PeerId[]) {
  function checkPeerIds() {
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

  function generateKeyShares(): Uint8Array[] {
    let done = false
    let secrets: Uint8Array[]
    let privKey: Uint8Array

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

      header.alpha.set(secp256k1.publicKeyCreate(privKey), 0)

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

    return secrets
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

  async function createBetaAndGamma(secrets: Uint8Array[], filler: Uint8Array, identifier: Uint8Array) {
    const tmp = new Uint8Array(BETA_LENGTH - PER_HOP_SIZE)

    for (let i = secrets.length; i > 0; i--) {
      const { key, iv } = derivePRGParameters(secrets[i - 1])

      let paddingLength = (MAX_HOPS - secrets.length) * PER_HOP_SIZE

      if (i == secrets.length) {
        header.beta.set(peerIds[i - 1].pubKey.marshal(), 0)
        header.beta.set(identifier, DESINATION_SIZE)

        // @TODO filling the array might not be necessary
        if (paddingLength > 0) {
          header.beta.set(randomBytes(paddingLength), LAST_HOP_SIZE)
        }

        u8aXOR(
          true,
          header.beta.subarray(0, LAST_HOP_SIZE + paddingLength),
          PRG.createPRG(key, iv).digest(0, LAST_HOP_SIZE + paddingLength)
        )

        header.beta.set(filler, LAST_HOP_SIZE + paddingLength)
      } else {
        tmp.set(header.beta.subarray(0, BETA_LENGTH - PER_HOP_SIZE), 0)

        header.beta.set(peerIds[i].pubKey.marshal(), 0)
        header.beta.set(header.gamma, ADDRESS_SIZE)

        // Used for the challenge that is created for the next node
        header.beta.set(await hash(deriveTicketKeyBlinding(secrets[i])), ADDRESS_SIZE + MAC_SIZE)
        header.beta.set(tmp, PER_HOP_SIZE)

        if (i < secrets.length - 1) {
          /**
           * Tells the relay node which challenge it should for the issued ticket.
           * The challenge should be done in a way such that:
           *   - the relay node does not know how to solve it
           *   - having one secret share is not sufficient to reconstruct
           *     the secret
           *   - the relay node can verify the key derivation path
           */
          header.beta.set(
            await hash(
              await hash(u8aConcat(deriveTicketKey(secrets[i]), await hash(deriveTicketKeyBlinding(secrets[i + 1]))))
            ),
            ADDRESS_SIZE + MAC_SIZE + KEY_LENGTH
          )
        } else if (i == secrets.length - 1) {
          header.beta.set(await hash(deriveTicketLastKey(secrets[i])), ADDRESS_SIZE + MAC_SIZE + KEY_LENGTH)
        }

        u8aXOR(true, header.beta, PRG.createPRG(key, iv).digest(0, BETA_LENGTH))
      }

      header.gamma.set(createMAC(secrets[i - 1], header.beta), 0)
    }
  }

  checkPeerIds()
  const secrets = generateKeyShares()
  const identifier = randomBytes(IDENTIFIER_SIZE)
  const filler = generateFiller(secrets)
  await createBetaAndGamma(secrets, filler, identifier)

  // printValues(header, secrets)

  return {
    header: header,
    secrets: secrets,
    identifier: identifier
  }
}
