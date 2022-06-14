import { findPath } from '@hoprnet/hopr-core'
import BN from 'bn.js'
import { BigNumber } from 'bignumber.js'
import { type PublicKey, type ChannelEntry, randomFloat } from '@hoprnet/hopr-utils'
import type { State, PersistedState } from './state'
import { CT_PATH_RANDOMNESS, CT_INTERMEDIATE_HOPS } from './constants'
import { debug } from '@hoprnet/hopr-utils'

const log = debug('hopr:cover-traffic')

/**
 * Interval of the unreleased token schedule. Unrecordeed intervals has 0 unreleased token allocation.
 * @param lowerBlock The lower bound of the time interval during which the unreleased token amount is defined
 * @param unreleased The unreleased token amount in 18 decimals
 */
export type UnreleasedSchedule = {
  lowerBlock: number
  upperBlock: number
  unreleased: string
}

export type UnreleasedTokens = {
  // hopr id to node Ethereum addresses
  link: {
    [index: string]: string[]
  }
  // Node Ethereum address to the array of unreleased token schedule
  allocation: {
    [index: string]: UnreleasedSchedule[]
  }
}

// @TODO inefficient & does not support runtime updates
const unreleasedTokens: UnreleasedTokens = require('../unreleasedTokens.json')

/**
 * Gets the integer part of the sqaure root of a an integer
 * @dev squaring the result in general does not lead to the input value
 * @param int the integer to compute the square root
 * @returns the rounded square root
 */
export function sqrtBN(int: BN): BN {
  return new BN(new BigNumber(int.toString()).squareRoot().integerValue().toFixed(), 10)
}

/**
 * Get channels opened from a node with a given public key in the state.
 * @param p Public key of the `source` node
 * @returns a list of channel entries where the `source` is the given public key
 */
export function findChannelsFrom(p: PublicKey, state: State): ChannelEntry[] {
  let result: ChannelEntry[] = []
  for (const channelId in state.channels) {
    const channel = state.channels[channelId].channel
    if (channel.source.eq(p)) {
      result.push(channel)
    }
  }
  return result
}

/**
 * Get the total outgoing channel balance of a node, given the network state.
 * totalChannelBalance(n) = balance of each channel from node n
 * @param p Public key of the node
 * @param state State of the network
 * @returns Total channel balance in big number
 */
export function totalChannelBalanceFor(p: PublicKey, state: State): BN {
  const result = new BN(0)
  for (const channel of findChannelsFrom(p, state)) {
    result.iadd(channel.balance.toBN())
  }
  return result
}

/**
 * Get the stake of a node which consists of the total channel balance
 * and the unreleased token amount in big number
 *
 * stake(n) = unreleasedTokens(n) + totalChannelBalance(n)
 *
 * @param p Public key of the node
 * @param state State of the network
 * @returns Stake of a node in big number
 */
export function stakeFor(p: PublicKey, state: State): BN {
  let linkedAccounts: UnreleasedTokens['link'][string] = null

  const b58String = p.toB58String()
  for (const id in unreleasedTokens.link) {
    // Base58 encoding is case-sensitive
    if (id === b58String) {
      linkedAccounts = unreleasedTokens.link[id]
    }
  }

  if (!linkedAccounts) {
    return totalChannelBalanceFor(p, state)
  }

  const currentBlockNumber = state.block.toNumber()

  const result = totalChannelBalanceFor(p, state)
  for (const linkedAccount of linkedAccounts) {
    let accountSchedule: UnreleasedSchedule = null
    for (const schedule of unreleasedTokens.allocation[linkedAccount]) {
      if (schedule.lowerBlock <= currentBlockNumber && currentBlockNumber < schedule.upperBlock) {
        accountSchedule = schedule
      }
    }

    if (accountSchedule) {
      result.iadd(new BN(accountSchedule.unreleased))
    }
  }

  return result
}

/**
 * Get the importance score of a node, given the network state.
 * Sum of the square root of all the outgoing channels
 * importance(node) = sum(squareRoot((balance(node) * stake(node) * totalStake(node)))
 * where stake(n) = unreleasedTokens(n) + totalChannelBalance(n)
 * @param p Public key of the node
 * @param state State of the network
 * @returns Total channel balance in big number
 */
export function importance(p: PublicKey, state: State): BN {
  const result = new BN(0)
  for (const channel of findChannelsFrom(p, state)) {
    result.iadd(sqrtBN(stakeFor(p, state).imul(channel.balance.toBN()).imul(stakeFor(channel.destination, state))))
  }
  return result
}

/**
 * Return the randomnized importance score of a node, given the network state.
 * @param p Public key of the node
 * @param state State of the network
 * @returns the randmonized importance score
 */
export function randomWeightedImportance(p: PublicKey, state: State): BN {
  const randomComponent = 1 + randomFloat() * CT_PATH_RANDOMNESS
  return importance(p, state).muln(randomComponent)
}

/**
 * Find the channel entry that is between the provided source and destination,
 * given the network state.
 * @param src Public key of the `source` of the channel
 * @param dest Public key of the `destination` of the channel
 * @returns ChannelEntry between `source` and `destination`, undefined otherwise.
 */
export function findChannel(src: PublicKey, dest: PublicKey, state: State): ChannelEntry | null {
  for (const channelId in state.channels) {
    const channel = state.channels[channelId].channel
    if (channel.source.eq(src) && channel.destination.eq(dest)) {
      return channel
    }
  }
  return null
}

/*
 * Find the timestamp at which a CT channel is opened.
 * Returns current time if channel does not exist.
 * @param dest Public key of the `destination` of the channel
 * @param state State of the network
 */
export function findCtChannelOpenTime(dest: PublicKey, state: State): number {
  const ctChannel = state.ctChannels.find((ctChannel) => ctChannel.destination.eq(dest))
  return !ctChannel ? Date.now() : ctChannel.openFrom ?? Date.now()
}

/**
 *
 * @param startNode Public key of the first-hop node
 * @param selfPub Public key of the cover-traffic node, which is also the recipient of the message.
 * @param sendMessage Method to send messages
 * @param data Persisted state of the network.
 * @returns false if no path is found.
 */
export const sendCTMessage = async (
  startNode: PublicKey,
  selfPub: PublicKey,
  sendMessage: (message: Uint8Array, path: PublicKey[]) => Promise<void>,
  data: PersistedState
): Promise<boolean> => {
  let path: PublicKey[]

  // build CT message,
  const counter = data.messageTotalSuccess()
  const message = new TextEncoder().encode(`CT_${counter.toString}`)

  try {
    path = await findPath(
      startNode,
      selfPub,
      CT_INTERMEDIATE_HOPS - 1, // As us to start is first intermediate
      (_p: PublicKey): number => 1, // TODO: network quality?
      (p: PublicKey) => Promise.resolve(data.findChannelsFrom(p)),
      // get the randomized weighted importance score of the destination of a given channel.
      (edge: ChannelEntry): Promise<BN> => Promise.resolve(randomWeightedImportance(edge.destination, data.get()))
    )

    // update counters in the state
    path.forEach((p, i, a) => {
      // increase counter for non-1st hop nodes
      if (i < CT_INTERMEDIATE_HOPS - 2) {
        data.incrementForwards(p, a[i + 1])
      } else {
        data.incrementForwards(p, selfPub)
      }
    })
    data.incrementSent(startNode, path[0]) // increase counter for 1st hop node

    // build the complete path
    path.unshift(startNode) // Path doesn't normally include this

    log(`SEND ${path.map((pub) => pub.toB58String()).join(',')}`)
  } catch (e) {
    return false
  }
  try {
    await sendMessage(message, path)
    log(`success sending ${path.map((pub) => pub.toB58String()).join(',')} message ${message}`)
    return true
  } catch (e) {
    log(`error ${e} sending to ${startNode.toB58String()}`)
    return false
  }
}
