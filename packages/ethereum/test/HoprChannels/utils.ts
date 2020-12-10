import type { AsyncReturnType } from 'type-fest'
import type { HoprChannelsInstance } from '../../types'
import type { Account } from '../types'
import type { Ticket } from './types'
import { prefixMessage, signMessage } from '../utils'
import Web3 from 'web3'
import BN from 'bn.js'
import { constants } from '@openzeppelin/test-helpers'
import { stringToU8a, u8aToHex } from '@hoprnet/hopr-utils'

const { numberToHex, encodePacked, soliditySha3 } = Web3.utils

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
