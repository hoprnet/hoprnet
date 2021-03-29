import type { Account } from './types'
import { publicKeyConvert, publicKeyCreate, ecdsaSign } from 'secp256k1'
import { stringToU8a, u8aToHex, u8aConcat } from '@hoprnet/hopr-utils'
import Web3 from 'web3'

const { soliditySha3, toChecksumAddress, toHex } = Web3.utils

/**
 * Depending on what network tests are run, the error output
 * may vary. This utility prefixes the error to it matches
 * with hardhat's network.
 * @param error
 * @returns error prefixed by network's message
 */
export const vmErrorMessage = (error: string) => {
  return `VM Exception while processing transaction: revert ${error}`
}

/**
 * Given a private key generate necessary data for testing
 * @param privKey
 * @returns Account
 */
export const createAccount = (privKey: string): Account => {
  const pubKey = publicKeyCreate(stringToU8a(privKey), true)
  const uncompressedPubKey = publicKeyConvert(pubKey, false).slice(1)
  const address = toChecksumAddress(
    u8aToHex(
      stringToU8a(
        soliditySha3({
          type: 'bytes',
          value: u8aToHex(uncompressedPubKey)
        })
      ).slice(12)
    )
  )

  return {
    privKey,
    uncompressedPubKey: u8aToHex(uncompressedPubKey),
    pubKey: u8aToHex(pubKey),
    address
  }
}

/**
 * Prefix message with our special message
 * @param message
 * @returns hashed message
 */
export const prefixMessage = (message: string): Uint8Array => {
  const messageWithHOPR = u8aConcat(stringToU8a(toHex('HOPRnet')), stringToU8a(message))
  const messageWithHOPRHex = u8aToHex(messageWithHOPR)

  return stringToU8a(
    soliditySha3(
      {
        type: 'string',
        value: '\x19Ethereum Signed Message:\n'
      },
      {
        type: 'string',
        value: messageWithHOPR.length.toString()
      },
      { type: 'bytes', value: messageWithHOPRHex }
    )
  )
}

/**
 * Sign message using private key provided
 * @param message
 * @param privKey
 * @returns signature properties
 */
export const signMessage = (
  message: string,
  privKey: Uint8Array
): { signature: Uint8Array; r: Uint8Array; s: Uint8Array; v: number } => {
  const { signature, recid } = ecdsaSign(stringToU8a(message), privKey)

  return {
    signature: signature,
    r: signature.slice(0, 32),
    s: signature.slice(32, 64),
    v: recid
  }
}
