import assert from 'assert'
import { Utils, Types } from '@hoprnet/hopr-core-connector-interface'
// @ts-ignore-next-line
import keccak256 from 'keccak256'
// @ts-ignore-next-line
import { publicKeyConvert, ecdsaSign, ecdsaVerify } from 'secp256k1'
import Web3 from 'web3'
import Constants from '../constants'
import { Signature } from '../srml_types'
import BN from 'bn.js'
import { PromiEvent } from 'web3-core'

const constants = new Constants()
const { COMPRESSED_PUBLIC_KEY_LENGTH, ETHEUREUM_ADDRESS_LENGTH } = constants

export const isPartyA: Utils['isPartyA'] = (self, counterparty) => {
  return Buffer.compare(self, counterparty) < 0
}

export const getParties = (
  self: Types.AccountId,
  counterparty: Types.AccountId
): [Types.AccountId, Types.AccountId] => {
  if (isPartyA(self, counterparty)) {
    return [self, counterparty]
  } else {
    return [counterparty, self]
  }
}

export const getId: Utils['getId'] = (self, counterparty, ...props) => {
  return hash(Buffer.concat(getParties(self, counterparty), 2 * ETHEUREUM_ADDRESS_LENGTH))
}

export const pubKeyToAccountId: Utils['pubKeyToAccountId'] = async (pubKey, ...args) => {
  if (pubKey.length != COMPRESSED_PUBLIC_KEY_LENGTH)
    throw Error(
      `Invalid input parameter. Expected a Buffer of size ${COMPRESSED_PUBLIC_KEY_LENGTH}. Got '${typeof pubKey}'${
        pubKey.length ? ` of length ${pubKey.length}` : ''
      }.`
    )

  return publicKeyConvert(pubKey, false).slice(1)
}

export const hash: Utils['hash'] = async msg => {
  return new Uint8Array(keccak256(Buffer.from(msg)))
}

export const sign: Utils['sign'] = async (msg, privKey, pubKey) => {
  const result = ecdsaSign(msg, privKey)

  const response: Types.Signature = new Signature(undefined, {
    onChainSignature: result.signature,
    recovery: result.recovery
  })

  return response
}

export const verify: Utils['verify'] = async (msg, signature, pubKey) => {
  return ecdsaVerify(signature.signature, msg, pubKey)
}

export const convertUnit: Utils['convertUnit'] = (amount, sourceUnit, targetUnit): BN => {
  assert(['eth', 'wei'].includes(sourceUnit), 'not implemented')

  if (sourceUnit === 'eth') {
    return Web3.utils.toWei(amount, targetUnit as any) as any
  } else {
    return Web3.utils.fromWei(amount, targetUnit as any) as any
  }
}

// TODO: check if this causes RAM issues
export const waitForConfirmation = async <T extends PromiEvent<any>>(event: T) => {
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
