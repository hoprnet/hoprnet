import type { AsyncReturnType } from 'type-fest'
import type { HoprChannelsInstance, ERC777MockContract } from '../../types'
import type { Account, Ticket } from './types'
import Web3 from 'web3'
import BN from 'bn.js'
import { publicKeyConvert, publicKeyCreate, ecdsaSign } from 'secp256k1'
import { constants } from '@openzeppelin/test-helpers'
import { stringToU8a, u8aToHex, u8aConcat } from '@hoprnet/hopr-utils'

const { numberToHex, encodePacked, soliditySha3, toChecksumAddress, toHex } = Web3.utils

/**
 * @param response web3 response
 * @returns formatted response
 */
export const formatAccount = (response: AsyncReturnType<HoprChannelsInstance['accounts']>) => ({
  secret: response[0],
  counter: response[1]
})

/**
 * @param response web3 response
 * @returns formatted response
 */
export const formatChannel = (response: AsyncReturnType<HoprChannelsInstance['channels']>) => ({
  deposit: response[0],
  partyABalance: response[1],
  closureTime: response[2],
  status: response[3],
  closureByPartyA: response[4]
})

/**
 * Create an ERC777 token instance to use in tests
 * @param ERC777 an ERC777Mock contract artifact
 * @param initialHolder ethereum address
 * @param initialBalance
 * @returns A ERC777Mock token instance
 */
export const ERC777Mock = (ERC777: ERC777MockContract, initialHolder: string, initialBalance: string) => {
  return ERC777.new(initialHolder, initialBalance, 'Token', 'TKN', [])
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

/**
 * Given a private key generate necessary data for testing
 * @param privKey
 * @returns Account
 */
export const createAccount = (privKey: string): Account => {
  const pubKey = publicKeyCreate(stringToU8a(privKey), true)
  const pubKeyUncompressed = publicKeyConvert(pubKey, false).slice(1)
  const firstHalf = new BN(pubKeyUncompressed.slice(0, 32))
  const secondHalf = new BN(pubKeyUncompressed.slice(32, 64))
  const address = toChecksumAddress(
    u8aToHex(
      stringToU8a(
        soliditySha3({
          type: 'bytes',
          value: u8aToHex(pubKeyUncompressed)
        })
      ).slice(12)
    )
  )

  return {
    privKey,
    pubKey: u8aToHex(pubKey),
    pubKeyFirstHalf: firstHalf,
    pubKeySecondHalf: secondHalf,
    address
  }
}

/**
 * Upscale a percentage (0-100) to uint256's maximum number
 * @param percent
 */
export const percentToUint256 = (percent: number): string => {
  return numberToHex(new BN(percent).mul(constants.MAX_UINT256).idivn(100).toString())
}

/**
 * Encode ticket data that is used to create a ticket hash
 * @param ticket
 * @return ticket's hash
 */
export const getEncodedTicket = (ticket: Ticket): string => {
  const challenge = soliditySha3({
    type: 'bytes32',
    value: ticket.proofOfRelaySecret
  })

  return encodePacked(
    {
      type: 'address',
      value: ticket.recipient
    },
    {
      type: 'bytes32',
      value: challenge
    },
    {
      type: 'uint256',
      value: ticket.counter
    },
    {
      type: 'uint256',
      value: ticket.amount
    },
    {
      type: 'bytes32', //@TODO: change to uint256?
      value: ticket.winProb
    },
    {
      type: 'uint256',
      value: ticket.iteration
    }
  )
}

/**
 * Get ticket's luck in bytes32
 * @param ticket
 * @param hash ticketHash
 * @param secret recipient's secret
 * @returns ticket's luck
 */
export const getTicketLuck = (ticket: Ticket, hash: string, secret: string): string => {
  const encoded = encodePacked(
    {
      type: 'bytes32',
      value: hash
    },
    {
      type: 'bytes32',
      value: secret
    },
    {
      type: 'bytes32',
      value: ticket.proofOfRelaySecret
    },
    {
      type: 'bytes32', //@TODO: change to uint256?
      value: ticket.winProb
    }
  )

  return soliditySha3({
    type: 'bytes',
    value: encoded
  })
}

/**
 * Given ticket data, generate a ticket for testing
 * @param ticket
 */
export const createTicket = (
  ticket: Ticket,
  account: Account,
  secret: string
): Ticket & {
  counterparty: string
  encoded: string
  hash: string
  luck: string
  signature: string
  r: string
  s: string
  v: number
} => {
  const encoded = getEncodedTicket(ticket)
  const hash = u8aToHex(prefixMessage(encoded))
  const luck = getTicketLuck(ticket, hash, secret)
  const { signature, r, s, v } = signMessage(hash, stringToU8a(account.privKey))

  return {
    ...ticket,
    encoded,
    hash,
    luck,
    r: u8aToHex(r),
    s: u8aToHex(s),
    v: v + 27, // why add 27? https://bitcoin.stackexchange.com/a/38909
    signature: u8aToHex(signature),
    counterparty: account.address
  }
}
