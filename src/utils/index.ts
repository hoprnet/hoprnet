import assert from 'assert'
import { publicKeyConvert, publicKeyCreate, ecdsaSign, ecdsaVerify } from 'secp256k1'
// @ts-ignore-next-line
import keccak256 from 'keccak256'
import { PromiEvent, TransactionReceipt } from 'web3-core'
import Web3 from 'web3'
import BN from "bn.js"
import type { Types } from '@hoprnet/hopr-core-connector-interface'
import { AccountId, Signature, Hash } from "../types"
import * as constants from '../constants'

export function isPartyA(self: Types.AccountId, counterparty:Types.AccountId): boolean {
  return Buffer.compare(self, counterparty) < 0
}

export function getParties(
  self: Types.AccountId,
  counterparty: Types.AccountId
): [Types.AccountId, Types.AccountId] {
  if (isPartyA(self, counterparty)) {
    return [self, counterparty]
  } else {
    return [counterparty, self]
  }
}

export function getId(self: Types.AccountId, counterparty: Types.AccountId) {
  return hash(Buffer.concat(getParties(self, counterparty), 2 * constants.ADDRESS_LENGTH))
}

export async function privKeyToPubKey(privKey: Uint8Array): Promise<Uint8Array> {
  if (privKey.length != constants.PRIVATE_KEY_LENGTH)
    throw Error(
      `Invalid input parameter. Expected a Buffer of size ${constants.PRIVATE_KEY_LENGTH}. Got '${typeof privKey}'${
        privKey.length ? ` of length ${privKey.length}` : ''
      }.`
    )

  return publicKeyCreate(privKey)
}

export async function pubKeyToAccountId(pubKey: Uint8Array): Promise<Types.AccountId> {
  if (pubKey.length != constants.COMPRESSED_PUBLIC_KEY_LENGTH)
    throw Error(
      `Invalid input parameter. Expected a Buffer of size ${
        constants.COMPRESSED_PUBLIC_KEY_LENGTH
      }. Got '${typeof pubKey}'${pubKey.length ? ` of length ${pubKey.length}` : ''}.`
    )

  return new AccountId(publicKeyConvert(pubKey, false).slice(1))
}

export function hashSync(msg: Uint8Array): Types.Hash {
  return new Hash(new Uint8Array(keccak256(Buffer.from(msg))))
}

export async function hash(msg: Uint8Array): Promise<Types.Hash> {
  return hashSync(msg)
}

export async function sign(msg: Uint8Array, privKey: Uint8Array): Promise<Types.Signature> {
  const result = ecdsaSign(msg, privKey)

  const response = new Signature(undefined, {
    signature: result.signature,
    recovery: result.recovery
  })

  return response
}

export async function verify(msg: Uint8Array, signature: Types.Signature, pubKey: Uint8Array): Promise<boolean> {
  return ecdsaVerify(signature.signature, msg, pubKey)
}

export function convertUnit(amount: BN, sourceUnit: string, targetUnit: string): BN {
  assert(['eth', 'wei'].includes(sourceUnit), 'not implemented')

  if (sourceUnit === 'eth') {
    return Web3.utils.toWei(amount, targetUnit as any) as any
  } else {
    return Web3.utils.fromWei(amount, targetUnit as any) as any
  }
}

export async function waitForConfirmation<T extends PromiEvent<any>>(event: T) {
  return new Promise<TransactionReceipt>((resolve, reject) => {
    return event
      .once('confirmation', (confNumber, receipt) => {
        resolve(receipt)
      })
      .once('error', error => {
        reject(error)
      })
  })
}
