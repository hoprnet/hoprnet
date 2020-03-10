import { Utils, Types } from '@hoprnet/hopr-core-connector-interface'
import assert from 'assert'
import { publicKeyConvert, publicKeyCreate, ecdsaSign, ecdsaVerify } from 'secp256k1'
// @ts-ignore-next-line
import keccak256 from 'keccak256'
import { PromiEvent } from 'web3-core'
import Web3 from 'web3'
import * as constants from '../constants'
import { Signature } from '../types'
import { stringToU8a, u8aToHex } from '../core/u8a'

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

export const getId: Utils['getId'] = function getId(self, counterparty) {
  return hash(Buffer.concat(getParties(self, counterparty), 2 * constants.ADDRESS_LENGTH))
}

export const privKeyToPubKey = async function privKeyToPubKey(privKey: Uint8Array): Promise<Uint8Array> {
  if (privKey.length != constants.PRIVATE_KEY_LENGTH)
    throw Error(
      `Invalid input parameter. Expected a Buffer of size ${constants.PRIVATE_KEY_LENGTH}. Got '${typeof privKey}'${
        privKey.length ? ` of length ${privKey.length}` : ''
      }.`
    )

  return publicKeyCreate(privKey)
}

export const pubKeyToAddress: Utils['pubKeyToAccountId'] = async function pubKeyToAddress(pubKey) {
  if (pubKey.length != constants.COMPRESSED_PUBLIC_KEY_LENGTH)
    throw Error(
      `Invalid input parameter. Expected a Buffer of size ${
        constants.COMPRESSED_PUBLIC_KEY_LENGTH
      }. Got '${typeof pubKey}'${pubKey.length ? ` of length ${pubKey.length}` : ''}.`
    )

  return hash(publicKeyConvert(pubKey, false).slice(1))
    .then(v => u8aToHex(v))
    .then(v => v.replace(/(0x)[0-9a-fA-F]{24}([0-9a-fA-F]{20})/, '$1$2'))
    .then(v => stringToU8a(v))
}
export const pubKeyToAccountId: Utils['pubKeyToAccountId'] = pubKeyToAddress

export const hash: Utils['hash'] = async function hash(msg) {
  return new Uint8Array(keccak256(Buffer.from(msg)))
}

export const sign: Utils['sign'] = async function sign(msg, privKey) {
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

export const waitForConfirmation = async function<T extends PromiEvent<any>>(event: T) {
  return new Promise((resolve, reject) => {
    return event
      .once('confirmation', (confNumber, receipt) => {
        resolve(receipt)
      })
      .once('error', error => {
        reject(error)
      })
  })
}
