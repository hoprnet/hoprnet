import secp256k1 from 'secp256k1'
import crypto from 'crypto'
import bs58 from 'bs58'
import forEachRight from 'lodash.foreachright'

import { PRG, u8aXOR } from '../../../utils'
import { MAX_HOPS } from '../../../constants'

import { Header, BETA_LENGTH, deriveBlinding, derivePRGParameters, deriveTransactionKey, createMAC } from './index'

import { HoprCoreConnectorClass } from '@hoprnet/hopr-core-connector-interface'

import PeerId from 'peer-id'

import { PRIVATE_KEY_LENGTH, PER_HOP_SIZE, DESINATION_SIZE, IDENTIFIER_SIZE, ADDRESS_SIZE, MAC_SIZE, COMPRESSED_PUBLIC_KEY_LENGTH, LAST_HOP_SIZE } from './parameters'

export function createHeader<Chain extends HoprCoreConnectorClass>(header: Header<Chain>, peerIds: PeerId[]) {
  function checkPeerIds() {
    if (peerIds.length > MAX_HOPS) {
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
      const G = secp256k1.publicKeyCreate(Buffer.from(mul))

      secrets = []

      do {
        privKey = crypto.randomBytes(PRIVATE_KEY_LENGTH)
      } while (!secp256k1.privateKeyVerify(Buffer.from(privKey)))

      header.alpha.set(secp256k1.publicKeyCreate(Buffer.from(privKey)), 0)

      mul.set(privKey, 0)

      peerIds.forEach((peerId: PeerId, index: number) => {
        // parallel
        // thread 1
        const alpha = secp256k1.publicKeyTweakMul(G, Buffer.from(mul))
        // secp256k1.publicKeyVerify(alpha)

        // thread 2
        const secret = secp256k1.publicKeyTweakMul(peerId.pubKey.marshal(), Buffer.from(mul))
        // secp256k1.publicKeyVerify(secret)
        // end parallel

        if (!secp256k1.publicKeyVerify(alpha) || !secp256k1.publicKeyVerify(secret)) {
          return
        }

        mul = secp256k1.privateKeyTweakMul(Buffer.from(mul), Buffer.from(deriveBlinding(alpha, secret)))

        if (!secp256k1.privateKeyVerify(Buffer.from(mul))) {
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
    const filler = new Uint8Array(PER_HOP_SIZE * (MAX_HOPS - 1))

    let length: number, start: number, end: number

    for (let index = 0; index < MAX_HOPS - 1; index++) {
      let { key, iv } = derivePRGParameters(secrets[index])

      start = LAST_HOP_SIZE + (MAX_HOPS - 1 - index) * PER_HOP_SIZE
      end = LAST_HOP_SIZE + MAX_HOPS * PER_HOP_SIZE

      length = (index + 1) * PER_HOP_SIZE

      u8aXOR(true, filler.subarray(0, length), PRG.createPRG(key, iv).digest(start, end))
    }

    return filler
  }

  function createBetaAndGamma(secrets: Uint8Array[], filler: Uint8Array, identifier: Uint8Array) {
    const tmp = new Uint8Array(BETA_LENGTH - PER_HOP_SIZE)

    forEachRight(secrets, (secret: Uint8Array, index: number) => {
      const { key, iv } = derivePRGParameters(secret)

      let paddingLength = (MAX_HOPS - secrets.length) * PER_HOP_SIZE

      if (index == secrets.length - 1) {
        header.beta.set(peerIds[index].pubKey.marshal(), 0)
        header.beta.set(identifier, DESINATION_SIZE)

        if (paddingLength > 0) {
          header.beta.fill(0, LAST_HOP_SIZE, paddingLength)
        }

        u8aXOR(true, header.beta.subarray(0, LAST_HOP_SIZE), PRG.createPRG(key, iv).digest(0, LAST_HOP_SIZE))
        header.beta.set(filler, LAST_HOP_SIZE + paddingLength)
      } else {
        tmp.set(header.beta.subarray(0, BETA_LENGTH - PER_HOP_SIZE), 0)

        header.beta.set(peerIds[index + 1].pubKey.marshal(), 0)
        header.beta.set(header.gamma, ADDRESS_SIZE)
        header.beta.set(secp256k1.publicKeyCreate(Buffer.from(deriveTransactionKey(secrets[index + 1]))), ADDRESS_SIZE + MAC_SIZE)
        header.beta.set(tmp, PER_HOP_SIZE)

        if (secrets.length > 2) {
          let key: Uint8Array
          if (index < secrets.length - 2) {
            key = secp256k1.privateKeyTweakAdd(Buffer.from(deriveTransactionKey(secrets[index + 1])), Buffer.from(deriveTransactionKey(secrets[index + 2])))
          } else if (index == secrets.length - 2) {
            // console.log(`created key half ${Header.deriveTransactionKey(secrets[index + 1]).toString('hex')}`)
            key = deriveTransactionKey(secrets[index + 1])
          }
          header.beta.set(key, ADDRESS_SIZE + MAC_SIZE + COMPRESSED_PUBLIC_KEY_LENGTH)
        }

        u8aXOR(true, header.beta, PRG.createPRG(key, iv).digest(0, BETA_LENGTH))
      }


      header.gamma.set(createMAC(secret, header.beta), 0)
    })
  }

  function deriveKey(a: Uint8Array, b: Uint8Array) {
    return secp256k1.privateKeyTweakAdd(Buffer.from(deriveTransactionKey(a)), Buffer.from(deriveTransactionKey(b)))
  }

  function printValues(header: Header<Chain>, secrets: Uint8Array[]) {
    console.log(
      peerIds.reduce((str, peerId, index) => {
        str =
          str +
          '\nsecret[' +
          index +
          ']: ' +
          bs58.encode(Buffer.from(secrets[index])) +
          '\n' +
          'peerId[' +
          index +
          ']: ' +
          peerId.toB58String() +
          '\n' +
          'peerId[' +
          index +
          '] pubkey ' +
          bs58.encode(peerId.pubKey.marshal())

        return str
      }, header.toString())
    )
  }

  checkPeerIds()
  const secrets = generateKeyShares()
  const identifier = crypto.randomBytes(IDENTIFIER_SIZE)
  const filler = generateFiller(secrets)
  createBetaAndGamma(secrets, filler, identifier)

  // printValues(header, secrets)

  return {
    header: header,
    secrets: secrets,
    identifier: identifier
  }
}
