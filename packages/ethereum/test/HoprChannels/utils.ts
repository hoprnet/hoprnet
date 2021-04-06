import { ethers } from 'ethers'
import { prefixMessageWithHOPR, signMessage } from '../utils'

export type Ticket = {
  recipient: string
  proofOfRelaySecret: string
  counter: string
  amount: string
  winProb: string
  iteration: string
}

const { solidityPack, solidityKeccak256 } = ethers.utils

/**
 * Upscale a percentage (0-100) to uint256's maximum number
 * @param percent
 */
export const percentToUint256 = (percent: number): string => {
  return ethers.constants.MaxUint256.mul(percent).div(100).toHexString()
}

/**
 * Encode ticket data that is used to create a ticket hash
 * @param ticket
 * @return ticket's hash
 */
export const getEncodedTicket = (ticket: Ticket): string => {
  const challenge = solidityKeccak256(['bytes32'], [ticket.proofOfRelaySecret])

  return solidityPack(
    ['address', 'bytes32', 'uint256', 'uint256', 'bytes32', 'uint256'],
    [ticket.recipient, challenge, ticket.counter, ticket.amount, ticket.winProb, ticket.iteration]
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
  const encoded = solidityPack(
    ['bytes32', 'bytes32', 'bytes32', 'bytes32'],
    [hash, secret, ticket.proofOfRelaySecret, ticket.winProb]
  )

  return solidityKeccak256(['bytes'], [encoded])
}

/**
 * Given ticket data, generate a ticket for testing
 * @param ticket
 */
export const createTicket = async (
  ticket: Ticket,
  account: {
    privateKey: string
    address: string
  },
  secret: string
): Promise<
  Ticket & {
    secret: string
    counterparty: string
    encoded: string
    hash: string
    luck: string
    signature: string
    r: string
    s: string
    v: number
  }
> => {
  const encoded = getEncodedTicket(ticket)
  const hash = prefixMessageWithHOPR(encoded)
  const luck = getTicketLuck(ticket, hash, secret)
  const { signature, r, s, v } = await signMessage(hash, account.privateKey)

  return {
    ...ticket,
    secret,
    encoded,
    hash,
    luck,
    r: r,
    s: s,
    v: v + 27, // why add 27? https://bitcoin.stackexchange.com/a/38909
    signature: signature,
    counterparty: account.address
  }
}
