import { Networks, networks } from '@hoprnet/hopr-ethereum'
import { u8aCompare, u8aConcat, u8aEquals, A_STRICLY_LESS_THAN_B, A_EQUALS_B, u8aToNumber } from '@hoprnet/hopr-utils'
import { Hash, Signature } from './types'
import BN from 'bn.js'

/**
 * Decides whether a ticket is a win or not.
 * Note that this mimics the on-chain logic.
 * @dev Purpose of the function is to check the validity of
 * a ticket before we submit it to the blockchain.
 * @param ticketHash hash value of the ticket to check
 * @param challengeResponse response that solves the signed challenge
 * @param preImage preImage of the current onChainSecret
 * @param winProb winning probability of the ticket
 */
export async function isWinningTicket(ticketHash: Hash, challengeResponse: Hash, preImage: Hash, winProb: Hash) {
  return [A_STRICLY_LESS_THAN_B, A_EQUALS_B].includes(
    u8aCompare(
      Hash.create(
        u8aConcat(ticketHash.serialize(), preImage.serialize(), challengeResponse.serialize(), winProb.serialize())
      ).serialize(),
      winProb.serialize()
    )
  )
}

/**
 * Compute the winning probability that is set for a ticket
 * @param prob Desired winning probability of a ticket, e.g. 0.6 resp. 60%
 */
export function computeWinningProbability(prob: number): Hash {
  if (prob == 1) {
    return new Hash(new Uint8Array(Hash.SIZE).fill(0xff))
  }

  if (prob == 0) {
    return new Hash(new Uint8Array(Hash.SIZE).fill(0x00))
  }

  let dividend = new BN(prob.toString(2).slice(2), 2)
  let divisor = new BN(0).bincn(prob.toString(2).slice(2).length)

  return new Hash(new Uint8Array(new BN(0).bincn(256).isubn(1).imul(dividend).div(divisor).toArray('be', Hash.SIZE)))
}

/**
 * Transforms Uint256 encoded probabilities into floats.
 *
 * @notice mostly used to check a ticket's winning probability.
 *
 * @notice the precision is very limited
 *
 * @param winProb Uint256-encoded version of winning probability
 */
export function getWinProbabilityAsFloat(winProb: Hash): number {
  if (u8aEquals(winProb.serialize(), new Uint8Array(Hash.SIZE).fill(0xff))) {
    return 1
  }

  if (u8aEquals(winProb.serialize(), new Uint8Array(Hash.SIZE).fill(0x00))) {
    return 0
  }

  return (
    (u8aToNumber(winProb.serialize().slice(0, 3)) as number) / (u8aToNumber(new Uint8Array(3).fill(0xff)) as number)
  )
}

/**
 * Checks whether the given response solves a given challenge
 * @param challenge challenge for which we search a preImage
 * @param response response to verify
 */
export async function checkChallenge(challenge: Hash, response: Hash) {
  return challenge.eq(response.hash())
}

/**
 * Get current network's name.
 *
 * @param chainId a chain id
 * @returns the network's name
 */
export function getNetworkName(chainId: number): Networks {
  const entry = Object.entries(networks).find(([_, options]) => options.chainId === chainId)

  if (entry) return entry[0] as Networks
  return 'localhost'
}

/**
 * Get current network's name.
 *
 * @param network
 * @returns the network's name
 */
export function getNetworkGasPrice(network: Networks): number | undefined {
  const entry = Object.entries(networks).find((entry) => entry[0] === network)

  if (entry && entry[1].gas) return entry[1].gas
  return undefined
}

/**
 * Get r,s,v values of a signature
 */
export function getSignatureParameters(
  signature: Signature
): {
  r: Hash
  s: Hash
  v: number
} {
  return {
    r: new Hash(signature.signature.slice(0, 32)),
    s: new Hash(signature.signature.slice(32, 64)),
    v: signature.recovery
  }
}
