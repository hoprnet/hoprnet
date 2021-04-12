import { Networks, networks } from '@hoprnet/hopr-ethereum'
import { PromiEvent, TransactionReceipt } from 'web3-core'
import { BlockTransactionString } from 'web3-eth'
import Web3 from 'web3'
import {
  u8aCompare,
  u8aConcat,
  u8aEquals,
  A_STRICLY_LESS_THAN_B,
  A_EQUALS_B,
  durations,
  u8aToNumber
} from '@hoprnet/hopr-utils'
import { Hash, Signature } from '../types'
import { ContractEventEmitter } from '../tsc/web3/types'
import * as constants from '../constants'
import * as time from './time'
import BN from 'bn.js'

export { time }

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
 * @param error from web3
 * @returns a known error, if we don't know it we return 'UNKNOWN'
 */
export function getWeb3ErrorType(error: string): 'OOF_NATIVE' | 'OOF_HOPR' | 'UNKNOWN' {
  if (error.includes(`enough funds`)) {
    return 'OOF_NATIVE'
  } else if (error.includes(`SafeERC20:`)) {
    return 'OOF_HOPR'
  } else {
    return 'UNKNOWN'
  }
}

/**
 * Wait until transaction has been mined.
 *
 * @typeparam T Our PromiEvent
 * @param event Our event, returned by web3
 * @returns the transaction receipt
 */
export async function waitForConfirmation<T extends PromiEvent<any>>(event: T) {
  return new Promise<TransactionReceipt>((resolve, reject) => {
    return event
      .on('receipt', (receipt) => resolve(receipt))
      .on('error', (error) => {
        const errorType = getWeb3ErrorType(error.message)

        if (errorType === 'OOF_NATIVE') {
          reject(constants.ERRORS.OOF_NATIVE)
        } else if (errorType === 'OOF_HOPR') {
          reject(constants.ERRORS.OOF_HOPR)
        } else {
          reject(error)
        }
      })
  })
}

/**
 * Wait until transaction has been mined.
 *
 * @param txHash transaction hash
 * @returns the transaction receipt
 */
export async function waitForConfirmationUsingHash(web3: Web3, txHash: string, timeout: number = durations.minutes(5)) {
  return new Promise<TransactionReceipt>(async (resolve, reject) => {
    const timer = setTimeout(() => reject('Waited for txHash confirmation too long.'), timeout)

    try {
      let mined = false
      let receipt: TransactionReceipt

      while (!mined) {
        receipt = await web3.eth.getTransactionReceipt(txHash)
        mined = !!receipt?.blockNumber
      }

      return resolve(receipt)
    } catch (err) {
      return reject(err.message)
    } finally {
      clearTimeout(timer)
    }
  })
}

/**
 * An asychronous setTimeout.
 *
 * @param ms milliseconds to wait
 */
export async function wait(ms: number) {
  return new Promise((resolve) => setTimeout(resolve, ms))
}

/**
 * Wait until timestamp is reached onchain.
 *
 * @param ms milliseconds to wait
 */
export async function waitFor({
  web3,
  network,
  getCurrentBlock,
  timestamp
}: {
  web3: Web3
  network: Networks
  getCurrentBlock: () => Promise<BlockTransactionString>
  timestamp?: number
}): Promise<void> {
  const now = await getCurrentBlock().then((block) => Number(block.timestamp) * 1e3)

  if (timestamp < now) {
    return undefined
  }

  const diff = now - timestamp || 60

  if (isGanache(network)) {
    await time.increase(web3, diff)
  } else {
    await wait(diff * 1e3)
  }

  return waitFor({
    web3,
    network,
    getCurrentBlock,
    timestamp: await getCurrentBlock().then((block) => Number(block.timestamp) * 1e3)
  })
}

/**
 * Get chain ID.
 *
 * @param web3 a web3 instance
 * @returns the chain ID
 */
export async function getChainId(web3: Web3): Promise<number> {
  return web3.eth.getChainId()
}

/**
 * Get current network's name.
 *
 * @param web3 a web3 instance
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
 * @param web3 a web3 instance
 * @returns the network's name
 */
export function getNetworkGasPrice(network: Networks): number | undefined {
  const entry = Object.entries(networks).find((entry) => entry[0] === network)

  if (entry && entry[1].gas) return entry[1].gas
  return undefined
}

/**
 * Once function 'fn' resolves, remove all listeners from 'event'.
 *
 * @typeparam E Our contract event emitteer
 * @typeparam R fn's return
 * @param event an event
 * @param fn a function to wait for
 */
export async function cleanupPromiEvent<E extends ContractEventEmitter<any>, R extends Promise<any>>(
  event: E,
  fn: (event: E) => R
): Promise<R> {
  return fn(event).finally(() => event.removeAllListeners())
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

/**
 * @param network
 * @returns true if network is private or ganache
 */
export function isGanache(network?: Networks): boolean {
  return !network || network === 'localhost'
}
