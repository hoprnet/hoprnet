import { Utils, Types } from '@hoprnet/hopr-core-connector-interface'
import assert from 'assert'
// @ts-ignore-next-line
import { publicKeyConvert, ecdsaSign, ecdsaVerify } from 'secp256k1'
// @ts-ignore-next-line
import keccak256 from 'keccak256'
import { PromiEvent } from 'web3-core'
import Web3 from 'web3'
import { Uint8Array } from 'src/types/extended'
import { COMPRESSED_PUBLIC_KEY_LENGTH, ETHEUREUM_ADDRESS_LENGTH } from 'src/constants'
import { Signature } from 'src/types'

export const isPartyA: Utils['isPartyA'] = function isPartyA(self, counterparty) {
  return Buffer.compare(self, counterparty) < 0
}

export const getParties = function getParties(
  self: Types.AccountId,
  counterparty: Types.AccountId
): [Types.AccountId, Types.AccountId] {
  if (isPartyA(self, counterparty)) {
    return [self, counterparty]
  } else {
    return [counterparty, self]
  }
}

export const getId: Utils['getId'] = function getId(self, counterparty, ...props) {
  return hash(Buffer.concat(getParties(self, counterparty), 2 * ETHEUREUM_ADDRESS_LENGTH))
}

// export const pubKeyToEthereumAddress: Utils['pubKeyToAccountId'] = function pubKeyToEthereumAddress(pubKey: Buffer) {
//   if (pubKey.length != COMPRESSED_PUBLIC_KEY_LENGTH)
//     throw Error(
//       `Invalid input parameter. Expected a Buffer of size ${COMPRESSED_PUBLIC_KEY_LENGTH}. Got '${typeof pubKey}'${
//         pubKey.length ? ` of length ${pubKey.length}` : ''
//       }.`
//     )

//   const hash = sha3(
//     publicKeyConvert(pubKey, false)
//       .slice(1)
//       .toString('hex')
//   )

//   return toChecksumAddress(hash.replace(/(0x)[0-9a-fA-F]{24}([0-9a-fA-F]{20})/, '$1$2'))
// }

export const pubKeyToAccountId: Utils['pubKeyToAccountId'] = async function pubKeyToAccountId(pubKey, ...args) {
  if (pubKey.length != COMPRESSED_PUBLIC_KEY_LENGTH)
    throw Error(
      `Invalid input parameter. Expected a Buffer of size ${COMPRESSED_PUBLIC_KEY_LENGTH}. Got '${typeof pubKey}'${
        pubKey.length ? ` of length ${pubKey.length}` : ''
      }.`
    )

  return publicKeyConvert(pubKey, false).slice(1)
}

export const hash: Utils['hash'] = async function hash(msg) {
  return new Uint8Array(keccak256(Buffer.from(msg)))
}

export const sign: Utils['sign'] = async function sign(msg, privKey, pubKey) {
  const result = ecdsaSign(msg, privKey)

  const response = new Signature(undefined, {
    signature: result.signature,
    recovery: result.recovery
  })

  return response
}

export const verify: Utils['verify'] = async function verify(msg, signature, pubKey) {
  return ecdsaVerify(signature.signature, msg, pubKey)
}

export const convertUnit: Utils['convertUnit'] = function convertUnit(amount, sourceUnit, targetUnit) {
  assert(['eth', 'wei'].includes(sourceUnit), 'not implemented')

  if (sourceUnit === 'eth') {
    return Web3.utils.toWei(amount, targetUnit as any) as any
  } else {
    return Web3.utils.fromWei(amount, targetUnit as any) as any
  }
}

// TODO: check if this causes RAM issues
export const waitForConfirmation = async function<T extends PromiEvent<any>>(event: T) {
  return new Promise((resolve, reject) => {
    return event
      .once('confirmation', (confNumber, recipient) => {
        resolve(recipient)
      })
      .once('error', error => {
        reject(error)
      })
  })
}
